//! Any run of asks, sound, a lost source and late events over the fake engine: every note passes its checks, one run
//! listens or writes at a time, the source is open only while a run listens, a Stop or Cancel always rests that run,
//! no piece, end or level comes after a run's end, Close or Cancel, and a recording is handed over only once it stopped.

use super::*;
use kx_ipc::{RECORD_MAX_S, SILENCE_STOP_S};
use proptest::collection::vec;
use proptest::prelude::*;
use std::collections::BTreeSet;

/// The most samples one step of sound gives.
const MAX_SOUND: usize = 2 * SEC;
/// The most samples a recording handed in holds.
const MAX_HAND_IN: usize = 2 * SEC;
/// The most steps in one case.
const MAX_STEPS: usize = 60;
/// Random cases; each records many seconds of sound.
const CASES: u32 = 64;
/// How often a Stop, Cancel, Keep or Close names the run before the last.
const BACK: f64 = 0.2;

/// One thing the job meets.
#[derive(Clone, Debug)]
enum Step {
    /// Start a fresh run, with a model when true, stopping on silence when true.
    Start(bool, bool),
    /// Write the held recording again as a fresh run.
    Again,
    /// Hand in a recording of this many samples as a fresh run.
    HandIn(usize),
    /// Stop, Cancel, Keep or Close the last run, or the one before when true.
    Stop(bool),
    Cancel(bool),
    Keep(bool),
    Close(bool),
    /// The source gives this many samples, loud when true, queued.
    Speak(bool, usize),
    /// The source stops by itself, queued.
    Lost,
    /// The job takes every event queued.
    Pump,
}

fn step() -> impl Strategy<Value = Step> {
    let back = || prop::bool::weighted(BACK);
    prop_oneof![
        3 => (any::<bool>(), any::<bool>()).prop_map(|(m, s)| Step::Start(m, s)),
        1 => Just(Step::Again),
        1 => (1..=MAX_HAND_IN).prop_map(Step::HandIn),
        2 => back().prop_map(Step::Stop),
        1 => back().prop_map(Step::Cancel),
        1 => back().prop_map(Step::Keep),
        1 => back().prop_map(Step::Close),
        6 => (any::<bool>(), 1..=MAX_SOUND).prop_map(|(l, n)| Step::Speak(l, n)),
        1 => Just(Step::Lost),
        4 => Just(Step::Pump),
    ]
}

/// A setup as the host sends it, with the shortest limit so runs reach it, and the shortest silence stop when `quiet`.
fn limited(model: bool, quiet: bool) -> Setup<'static> {
    let mut s = setup(model);
    s.record_max_s = *RECORD_MAX_S.start();
    s.silence_stop_s = quiet.then_some(*SILENCE_STOP_S.start());
    s
}

/// Queues `n` samples of sound on the source's last opening, loud when `speech`.
fn speak(rig: &Rig, speech: bool, n: usize) {
    let Some(feed) = rig.mic.borrow().feed.clone() else {
        return;
    };
    let x = if speech { loud(n) } else { silent(n) };
    for c in x.chunks(CHUNK) {
        feed.sound(Buf::collect(c.iter().copied()).unwrap());
    }
}

/// What the case saw so far.
#[derive(Default)]
struct Seen {
    /// The last run's id.
    last: u32,
    /// The runs that ended, or that a Close or Cancel was taken for.
    ended: BTreeSet<u32>,
}

/// Queues sound or a lost source, or takes every event queued; false for an ask.
fn queue(rig: &mut Rig, step: &Step) -> bool {
    match *step {
        Step::Speak(speech, n) => speak(rig, speech, n),
        Step::Lost => {
            let (epoch, fault) = (rig.job.epoch, VoiceFault::Unplugged);
            drop(rig.tx.send(Event::Lost { epoch, fault }));
        }
        Step::Pump => rig.pump(),
        _ => return false,
    }
    true
}

/// Runs `step`; the run a Stop, Cancel, Keep or Close named, and whether that run may tell nothing more after it.
fn run(rig: &mut Rig, seen: &mut Seen, step: &Step) -> Option<(u32, bool)> {
    if queue(rig, step) {
        return None;
    }
    let next = seen.last + 1;
    let fresh = match *step {
        Step::Start(model, quiet) => Some(VoiceAsk::Start {
            id: next,
            setup: limited(model, quiet),
        }),
        Step::Again => Some(VoiceAsk::Again {
            id: next,
            setup: limited(true, false),
        }),
        Step::HandIn(n) => Some(hand_in(next, 0, true, &ramp16(n))),
        _ => None,
    };
    if let Some(ask) = fresh {
        seen.last = next;
        rig.only(ask);
        return None;
    }
    let named = |back| seen.last.saturating_sub(u32::from(back));
    let (ask, ends) = match *step {
        Step::Stop(back) => (VoiceAsk::Stop { id: named(back) }, false),
        Step::Keep(back) => (VoiceAsk::Keep { id: named(back) }, false),
        Step::Cancel(back) => (VoiceAsk::Cancel { id: named(back) }, true),
        Step::Close(back) => (VoiceAsk::Close { id: named(back) }, true),
        _ => return None,
    };
    let id = ask.id();
    rig.only(ask);
    Some((id, ends))
}

/// Checks the notes one step told, in order: nothing after a run's end, and no hand-over while its run listens.
fn notes(rig: &Rig, seen: &mut Seen, told: &[VoiceNote]) -> Result<(), TestCaseError> {
    let listening = rig.job.run.as_ref().map(|r| r.id);
    for note in told {
        let id = note.id();
        let live = matches!(
            note,
            VoiceNote::Piece { .. } | VoiceNote::Done { .. } | VoiceNote::Level { .. }
        );
        prop_assert!(
            !live || !seen.ended.contains(&id),
            "{note:?} after run {id} ended"
        );
        if let VoiceNote::Sound { .. } | VoiceNote::Speech { .. } = note {
            prop_assert!(
                listening != Some(id),
                "run {id} handed over while it listens"
            );
        }
        if let VoiceNote::Done { .. } = note {
            seen.ended.insert(id);
        }
    }
    Ok(())
}

/// Checks the job after `step` on run `named`: the source open only while a run listens, and a Stop or Cancel rests it.
fn state(rig: &Rig, step: &Step, named: Option<(u32, bool)>) -> Result<(), TestCaseError> {
    let listening = rig.job.run.as_ref().map(|r| r.id);
    prop_assert_eq!(
        rig.open(),
        listening.is_some(),
        "the source and the run disagree"
    );
    let Some((id, _)) = named else {
        return Ok(());
    };
    if matches!(step, Step::Stop(_) | Step::Cancel(_)) {
        prop_assert!(
            listening != Some(id),
            "run {} still listens after {:?}",
            id,
            step
        );
    }
    if matches!(step, Step::Cancel(_)) {
        prop_assert!(
            rig.job.busy() != Some(id),
            "run {} still writes after Cancel",
            id
        );
    }
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn one_run_at_a_time_and_nothing_after_its_end(
        detector in any::<bool>(),
        steps in vec(step(), 1..MAX_STEPS),
    ) {
        let mut rig = Rig::new(RATE, 1, detector);
        let mut seen = Seen::default();
        for step in &steps {
            let busy = rig.job.busy();
            let named = run(&mut rig, &mut seen, step);
            let told = rig.told();
            if let Step::Start(..) = step {
                let opened = told.contains(&stage(seen.last, Stage::Open));
                prop_assert!(!opened || busy.is_none(), "run {} opened beside {:?}", seen.last, busy);
            }
            notes(&rig, &mut seen, &told)?;
            state(&rig, step, named)?;
            seen.ended.extend(named.filter(|n| n.1).map(|n| n.0));
        }
    }
}
