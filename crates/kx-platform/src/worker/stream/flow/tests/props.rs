//! Any run of fresh asks, asks on runs, notes, bad frames, quits, refusals, stops and clock moves: one worker at most,
//! no note told after its worker was lost or on an id it was not asked, no start while resting, and none left running.

use super::*;
use crate::worker::Frames;
use crate::worker::fake::voice::failed;
use proptest::prelude::*;

/// An id no run of a short test reaches.
const STRAY: u32 = u32::MAX;
/// Milliseconds in a second.
const MS_PER_S: u32 = 1_000;
/// The longest run.
const STEPS: usize = 60;
/// The most notes a scripted worker sends for one fresh ask.
const SAID: usize = 3;

/// A note for an id.
type Note = fn(u32) -> VoiceNote;

/// One thing that happens to the stream.
#[derive(Clone, Debug)]
enum Step {
    /// A fresh ask, which the worker meets with these notes, watched as this says.
    Begin(Vec<Note>, Wait),
    /// An ask on the last run begun.
    Ask(Wait),
    /// The worker sends a note on its own: on the last run, or on a stray id when true.
    Say(Note, bool),
    /// The worker sends a frame its job cannot read.
    Bad(IpcError),
    /// The worker quits.
    Quit,
    /// The next start is refused.
    Refuse,
    /// The reader hands the flow every frame so far.
    Feed,
    /// The owner stops the worker.
    Stop,
    /// The clock moves this many ms, and the flow checks its watches.
    Wait(u32),
}

fn note() -> impl Strategy<Value = Note> {
    let notes: Vec<Note> = vec![open, level, piece, done, devices, card_failed, failed];
    prop::sample::select(notes)
}

fn wait() -> impl Strategy<Value = Wait> {
    prop_oneof![
        Just(Wait::Off),
        Just(Wait::Once(WAIT)),
        Just(Wait::Each(WAIT))
    ]
}

fn step() -> impl Strategy<Value = Step> {
    let bad = vec![IpcError::BadKind, IpcError::TooLong, IpcError::Short];
    let begin = (prop::collection::vec(note(), 0..=SAID), wait());
    prop_oneof![
        2 => begin.prop_map(|(notes, wait)| Step::Begin(notes, wait)),
        1 => wait().prop_map(Step::Ask),
        2 => (note(), any::<bool>()).prop_map(|(note, stray)| Step::Say(note, stray)),
        1 => prop::sample::select(bad).prop_map(Step::Bad),
        1 => Just(Step::Quit),
        1 => Just(Step::Refuse),
        3 => Just(Step::Feed),
        1 => Just(Step::Stop),
        2 => (0..=2 * TIMES.restart_max_ms).prop_map(Step::Wait),
        1 => Just(Step::Wait(TIMES.idle_s * MS_PER_S)),
    ]
}

/// What the run saw so far.
struct Seen {
    /// The ids begun on the live worker; empty without one.
    owned: Vec<u32>,
    /// The last id begun.
    last: u32,
    /// How long the backoff still runs, as the last refusal told.
    rest: Duration,
}

/// How many workers were ended so far.
fn ends(script: &Script<VoiceFrames>) -> usize {
    script.calls().iter().filter(|c| **c == Call::End).count()
}

/// The asks sent so far.
fn asks(script: &Script<VoiceFrames>) -> usize {
    let ask = |call: &Call| matches!(call, Call::Voice(_));
    script.calls().iter().filter(|c| ask(c)).count()
}

/// A fresh ask met with `notes`; within the backoff it is refused with the time left, and starts and sends nothing.
fn begin(
    rig: &mut Rig,
    seen: &mut Seen,
    notes: Vec<Note>,
    wait: Wait,
) -> Result<(), TestCaseError> {
    let (tries, sent, spawns) = (rig.script.tries(), asks(&rig.script), rig.script.spawns());
    rig.script.act(VoiceAct::Say(notes));
    let begun = rig.flow.begin(start, wait);
    told(rig, seen)?;
    if !seen.rest.is_zero() {
        prop_assert_eq!(&begun, &down(seen.rest), "still down, for what is left");
        prop_assert_eq!(rig.script.tries(), tries, "no start within the backoff");
    }
    seen.rest = Duration::ZERO;
    match begun {
        Ok(id) => {
            prop_assert!(id > seen.last, "a fresh id");
            let last = rig.script.calls().pop();
            prop_assert_eq!(last, Some(Call::Voice(start(id))), "its own ask");
            if rig.script.spawns() > spawns {
                seen.owned.clear();
            }
            seen.last = id;
            seen.owned.push(id);
        }
        Err(Unsent::Lost(Lost::Down { retry_in })) => {
            prop_assert!(!retry_in.is_zero() && retry_in <= ms(TIMES.restart_max_ms));
            prop_assert_eq!(asks(&rig.script), sent, "a resting stream sends nothing");
            seen.rest = retry_in;
        }
        Err(other) => {
            let why = format!("a fresh ask is down or begun, not {other:?}");
            return Err(TestCaseError::fail(why));
        }
    }
    Ok(())
}

/// An ask on the last run: sent only to the worker that took it, and never a start.
fn ask(rig: &mut Rig, seen: &mut Seen, wait: Wait) -> Result<(), TestCaseError> {
    let tries = rig.script.tries();
    let asked = rig.flow.ask(stop(seen.last), wait);
    told(rig, seen)?;
    prop_assert_eq!(
        asked.is_ok(),
        seen.owned.contains(&seen.last),
        "its own worker"
    );
    prop_assert_eq!(rig.script.tries(), tries, "an ask starts nothing");
    Ok(())
}

/// Checks what the flow told since the last step: a note only from a live worker on an id it took.
fn told(rig: &mut Rig, seen: &mut Seen) -> Result<(), TestCaseError> {
    for told in rig.told.try_iter() {
        match told {
            Told::Note(note) => {
                let id = VoiceFrames::id(&note);
                prop_assert!(seen.owned.contains(&id), "note on {} told", id);
            }
            Told::Lost(_) | Told::Idle => seen.owned.clear(),
        }
    }
    Ok(())
}

/// Runs `step` and checks what it told.
fn run(rig: &mut Rig, seen: &mut Seen, step: Step) -> Result<(), TestCaseError> {
    match step {
        Step::Begin(notes, wait) => return begin(rig, seen, notes, wait),
        Step::Ask(wait) => return ask(rig, seen, wait),
        Step::Say(note, stray) => {
            let id = if stray { STRAY } else { seen.last };
            rig.script.say(Ok(note(id)));
        }
        Step::Bad(e) => rig.script.say(Err(e)),
        Step::Quit => rig.script.quit(DIED),
        Step::Refuse => rig.script.refuse(1),
        Step::Feed => rig.feed(),
        Step::Stop => {
            rig.flow.stop();
            seen.owned.clear();
        }
        Step::Wait(n) => {
            rig.clock.advance(ms(n));
            rig.flow.tick();
            rig.feed();
            seen.rest = seen.rest.saturating_sub(ms(n));
        }
    }
    told(rig, seen)
}

proptest! {
    #[test]
    fn any_run_keeps_one_worker_and_tells_no_note_of_a_lost_one(
        steps in prop::collection::vec(step(), 0..STEPS),
    ) {
        let mut rig = rig();
        let mut seen = Seen { owned: Vec::new(), last: 0, rest: Duration::ZERO };
        for step in steps {
            run(&mut rig, &mut seen, step)?;
            let (spawns, ends) = (rig.script.spawns(), ends(&rig.script));
            prop_assert!(ends <= spawns && spawns - ends <= 1, "one worker at most");
        }
        let Rig { flow, script, .. } = rig;
        drop(flow);
        prop_assert_eq!(ends(&script), script.spawns(), "none left running");
    }
}
