//! A stream's rules over a scripted voice worker, its frames fed in as the reader thread feeds them: notes told in
//! order, watches kept by each note or met by one and judged by when a frame was taken, a silent worker ended and held
//! back by a doubling backoff, and a frame the job cannot read ending the worker; nothing sleeps.

use super::*;
use crate::worker::VoiceFrames;
use crate::worker::fake::voice::{
    bad_start, cancel, card_failed, close, devices, done, level, list, open, piece, shut, start,
    stop,
};
use crate::worker::fake::{Call, Frame, Hands, Script, VoiceAct};
use kx_ipc::{VOICE_JOB, VoiceAsk, VoiceNote};
use std::sync::mpsc::{self, TryRecvError};

mod life;
mod props;

/// Test values; the real ones come from each port's config. The cap is four first waits, so the waits double twice.
const TIMES: Times = Times {
    deadline_ms: 2_000,
    idle_s: 30,
    restart_ms: 1_000,
    restart_max_ms: 4_000,
};
/// How long a test watch waits for a note.
const WAIT: Duration = Duration::from_millis(500);
/// The smallest step of the clock.
const TICK: Duration = Duration::from_micros(1);
/// The exit code of a worker that died.
const DIED: u32 = 3;

type Flows = Flow<Script<VoiceFrames>, VoiceFrames>;
type Notes = Receiver<Frame<VoiceFrames>>;

/// A flow, its script and clock, what it told, and each worker's notes as the reader thread would get them.
struct Rig {
    flow: Flows,
    script: Script<VoiceFrames>,
    clock: Hands,
    told: Receiver<Told<VoiceNote>>,
    pumps: Receiver<(u64, Notes)>,
    live: Vec<(u64, Notes)>,
}

/// A rig whose reader threads start.
fn rig() -> Rig {
    rig_with(Ok(()))
}

/// A rig whose reader threads start as `started` says.
fn rig_with(started: Result<(), SpawnError>) -> Rig {
    let (script, clock) = (Script::default(), Hands::default());
    let (said, told) = mpsc::channel();
    let (handed, pumps) = mpsc::channel();
    let tell: Tell<VoiceNote> = Box::new(move |t| drop(said.send(t)));
    let pump: Pump<VoiceNote> = Box::new(move |nth, notes| {
        let _ = handed.send((nth, notes));
        started.clone()
    });
    let launch = Launch {
        args: vec![VOICE_JOB.to_owned()],
        memory_mb: 64,
    };
    let (spawner, now) = (Arc::new(script.clone()), Arc::new(clock.clone()));
    Rig {
        flow: Flow::new(spawner, now, TIMES, launch, tell, pump),
        script,
        clock,
        told,
        pumps,
        live: Vec::new(),
    }
}

impl Rig {
    /// Feeds the flow each frame the workers sent, as the reader thread does, each taken now.
    fn feed(&mut self) {
        self.live.extend(self.pumps.try_iter());
        let (flow, at) = (&mut self.flow, self.clock.now());
        self.live.retain(|(nth, notes)| feed(flow, *nth, notes, at));
    }

    /// Feeds the flow each frame the workers sent and returns what it told.
    fn heard(&mut self) -> Vec<Told<VoiceNote>> {
        self.feed();
        self.told.try_iter().collect()
    }

    /// Moves the clock on by `by`, lets the flow check its watches and returns what it told.
    fn after(&mut self, by: Duration) -> Vec<Told<VoiceNote>> {
        self.clock.advance(by);
        self.flow.tick();
        self.heard()
    }

    /// A fresh ask `make` builds, watched for `WAIT`.
    fn begin(&mut self, make: fn(u32) -> VoiceAsk<'static>) -> Result<u32, Unsent> {
        self.flow.begin(make, Wait::Each(WAIT))
    }
}

/// Hands `notes`' frames to `flow`, taken `at`, until none is left; false once they closed or one failed, as the
/// reader then ends.
fn feed(flow: &mut Flows, nth: u64, notes: &Notes, at: Mono) -> bool {
    loop {
        let frame = match notes.try_recv() {
            Ok(frame) => frame,
            Err(TryRecvError::Empty) => return true,
            Err(TryRecvError::Disconnected) => Err(IpcError::Closed),
        };
        let last = frame.is_err();
        flow.heard(nth, frame, at);
        if last {
            return false;
        }
    }
}

/// Each note as told.
fn notes(list: &[VoiceNote]) -> Vec<Told<VoiceNote>> {
    list.iter().cloned().map(Told::Note).collect()
}

fn ms(n: u32) -> Duration {
    Duration::from_millis(u64::from(n))
}

/// How long the worker may sit idle with `TIMES`.
fn idle() -> Duration {
    Duration::from_secs(u64::from(TIMES.idle_s))
}

/// A fresh ask refused while the backoff runs for `retry_in`.
fn down(retry_in: Duration) -> Result<u32, Unsent> {
    Err(Unsent::Lost(Lost::Down { retry_in }))
}

#[test]
fn a_fresh_ask_starts_the_worker_and_its_notes_are_told_in_order() {
    let mut rig = rig();
    rig.script.act(VoiceAct::Say(vec![open, level, piece]));
    assert_eq!(rig.begin(start), Ok(1), "ids start at 1");
    assert_eq!(rig.heard(), notes(&[open(1), level(1), piece(1)]));
    rig.script.say(Ok(level(1)));
    assert_eq!(rig.heard(), notes(&[level(1)]), "a note on its own");
    assert_eq!(rig.script.spawns(), 1);
    assert_eq!(rig.script.calls(), [Call::Voice(start(1))]);
}

#[test]
fn each_note_keeps_its_watch_and_the_last_ends_it() {
    let mut rig = rig();
    rig.script.act(VoiceAct::Say(vec![open]));
    rig.begin(start).unwrap();
    rig.heard();
    for note in [level, card_failed, piece] {
        assert_eq!(rig.after(WAIT.saturating_sub(TICK)), [], "not late yet");
        rig.script.say(Ok(note(1)));
        assert_eq!(rig.heard(), notes(&[note(1)]), "and kept again");
    }
    rig.script.say(Ok(done(1)));
    rig.heard();
    assert_eq!(rig.after(WAIT * 2), [], "nothing is watched");
    assert_eq!(
        rig.flow.wake_in(),
        Some(idle().saturating_sub(WAIT * 2)),
        "the idle wait"
    );
}

#[test]
fn a_once_wait_is_met_by_one_note_and_the_id_may_then_stay_silent() {
    let mut rig = rig();
    rig.script.act(VoiceAct::Say(vec![open]));
    rig.begin(start).unwrap();
    rig.heard();
    rig.script.act(VoiceAct::Say(vec![shut]));
    rig.flow.ask(cancel(1), Wait::Once(WAIT)).unwrap();
    assert_eq!(rig.heard(), notes(&[shut(1)]));
    assert_eq!(rig.after(WAIT * 2), [], "met by its answer");
    rig.flow.ask(stop(1), Wait::Once(WAIT)).unwrap();
    assert_eq!(rig.after(WAIT), [Told::Lost(Lost::Late)], "but never met");
}

#[test]
fn a_frame_is_judged_by_when_its_reader_took_it() {
    let mut rig = rig();
    rig.begin(start).unwrap();
    rig.heard();
    let (nth, taken) = (rig.live[0].0, rig.clock.now());
    rig.clock.advance(WAIT * 2);
    rig.flow.heard(nth, Ok(open(1)), taken);
    assert_eq!(rig.heard(), notes(&[open(1)]), "taken in time, told late");
    rig.clock.advance(WAIT);
    rig.flow.heard(nth, Ok(level(1)), rig.clock.now());
    assert_eq!(rig.heard(), [Told::Lost(Lost::Late)], "taken past its wait");
}

#[test]
fn a_worker_silent_past_its_wait_is_ended_and_held_back_by_a_doubling_backoff() {
    let mut rig = rig();
    let (first, cap) = (ms(TIMES.restart_ms), ms(TIMES.restart_max_ms));
    for (n, wait) in [first, first * 2, cap, cap].into_iter().enumerate() {
        assert!(rig.begin(list).is_ok(), "a start after the backoff");
        assert_eq!(rig.after(WAIT.saturating_sub(TICK)), [], "not late yet");
        assert_eq!(rig.after(TICK), [Told::Lost(Lost::Late)]);
        assert_eq!(rig.script.calls().last(), Some(&Call::End));
        assert_eq!(rig.begin(list), down(wait), "failure {n} in a row");
        rig.clock.advance(wait.saturating_sub(TICK));
        assert_eq!(rig.begin(list), down(TICK), "still resting");
        rig.clock.advance(TICK);
    }
    assert_eq!(rig.script.spawns(), 4);
}

#[test]
fn a_note_on_a_watched_id_resets_the_backoff() {
    let mut rig = rig();
    let first = ms(TIMES.restart_ms);
    rig.begin(list).unwrap();
    rig.after(WAIT);
    rig.clock.advance(first);
    rig.script.act(VoiceAct::Say(vec![devices]));
    rig.begin(list).unwrap();
    rig.heard();
    rig.begin(list).unwrap();
    assert_eq!(rig.after(WAIT), [Told::Lost(Lost::Late)]);
    assert_eq!(rig.begin(list), down(first), "back to the first wait");
}

#[test]
fn a_note_of_an_unknown_kind_or_past_its_size_ends_the_worker() {
    for bad in [IpcError::BadKind, IpcError::TooLong] {
        let mut rig = rig();
        rig.script.act(VoiceAct::Say(vec![open]));
        rig.script.act(VoiceAct::Broken(bad));
        rig.begin(start).unwrap();
        rig.flow.ask(stop(1), Wait::Each(WAIT)).unwrap();
        rig.script.say(Ok(level(1)));
        let lost = Told::Lost(Lost::Gone);
        assert_eq!(rig.heard(), [Told::Note(open(1)), lost], "nothing after it");
        assert_eq!(rig.script.calls().last(), Some(&Call::End));
        assert_eq!(rig.begin(start), down(ms(TIMES.restart_ms)));
    }
}
