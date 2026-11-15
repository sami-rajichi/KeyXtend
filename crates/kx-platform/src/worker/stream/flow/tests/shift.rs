//! A shifting wait that takes its longer time once the microphone closed by itself, a relaunch that keeps the backoff,
//! and a run's end stamped with its job's peak memory.

use super::*;
use crate::worker::fake::voice::limit;

/// The longer wait a shifting watch takes after its shift.
const THEN: Duration = Duration::from_secs(5);
/// A test job's peak memory, in MB.
const PEAK_MB: u32 = 321;
/// A relaunched job's memory cap, in MB.
const BIGGER_MB: u32 = 128;

/// Begins a start watched short until the microphone closes, then long.
fn shifting(rig: &mut Rig) -> u32 {
    let wait = Wait::Shift {
        each: WAIT,
        then: THEN,
    };
    rig.flow.begin(start, wait).unwrap()
}

/// `done` with the job's memory set to `mb`.
fn with_memory(note: VoiceNote, mb: u32) -> VoiceNote {
    match note {
        VoiceNote::Done {
            id,
            pieces,
            sound_ms,
            wait_ms,
            load_ms,
            card,
            ..
        } => VoiceNote::Done {
            id,
            pieces,
            sound_ms,
            wait_ms,
            load_ms,
            memory_mb: mb,
            card,
        },
        other => other,
    }
}

#[test]
fn a_shifting_wait_takes_its_longer_time_after_the_note_that_shifts_it() {
    let mut rig = rig();
    rig.script.act(VoiceAct::Say(vec![open]));
    let id = shifting(&mut rig);
    rig.heard();
    assert_eq!(rig.after(WAIT.saturating_sub(TICK)), [], "not late yet");
    rig.script.say(Ok(limit(id)));
    assert_eq!(rig.heard(), notes(&[limit(id)]));
    assert_eq!(
        rig.after(THEN.saturating_sub(TICK)),
        [],
        "the long wait now"
    );
    rig.script.say(Ok(piece(id)));
    rig.heard();
    assert_eq!(
        rig.after(THEN.saturating_sub(TICK)),
        [],
        "kept by each note"
    );
    assert_eq!(rig.after(TICK), [Told::Lost(Lost::Late)]);
}

#[test]
fn before_its_shift_a_shifting_wait_keeps_its_short_time() {
    let mut rig = rig();
    rig.script.act(VoiceAct::Say(vec![open, level]));
    shifting(&mut rig);
    rig.heard();
    assert_eq!(rig.after(WAIT), [Told::Lost(Lost::Late)]);
}

#[test]
fn a_relaunch_says_goodbye_tells_nothing_and_keeps_the_backoff() {
    let mut rig = rig();
    rig.begin(start).unwrap();
    rig.heard();
    assert_eq!(rig.after(WAIT), [Told::Lost(Lost::Late)]);
    let bigger = Launch {
        args: vec![VOICE_JOB.to_owned()],
        memory_mb: BIGGER_MB,
    };
    rig.flow.relaunch(bigger.clone(), TIMES);
    assert_eq!(
        rig.begin(start),
        down(ms(TIMES.restart_ms)),
        "still held back"
    );
    rig.clock.advance(ms(TIMES.restart_ms));
    let id = rig.flow.begin(list, Wait::Off).unwrap();
    assert_eq!(rig.script.launches().last(), Some(&bigger));
    rig.flow.relaunch(bigger, TIMES);
    let within = ms(TIMES.deadline_ms);
    let bye = [Call::Bye, Call::Code { within }, Call::End];
    assert!(rig.script.calls().ends_with(&bye), "the live worker left");
    assert_eq!(rig.heard(), [], "a relaunch tells nothing");
    let gone = Err(Unsent::Lost(Lost::Gone));
    assert_eq!(rig.flow.ask(close(id), Wait::Off), gone, "its runs went");
}

#[test]
fn a_runs_end_carries_its_jobs_peak_memory() {
    let mut rig = rig();
    rig.script.peak(Some(PEAK_MB));
    rig.script.act(VoiceAct::Say(vec![open, level, done]));
    let id = rig.begin(start).unwrap();
    let ended = with_memory(done(id), PEAK_MB);
    assert_eq!(rig.heard(), notes(&[open(id), level(id), ended]));
}

#[test]
fn a_job_that_tells_no_peak_leaves_the_end_as_it_came() {
    let mut rig = rig();
    rig.script.act(VoiceAct::Say(vec![done]));
    let id = rig.begin(start).unwrap();
    assert_eq!(rig.heard(), notes(&[done(id)]));
}
