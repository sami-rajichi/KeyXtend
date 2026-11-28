//! Switches mid-run: each waits for the job's answer and the run goes on, a pass of another seal never reaches the
//! job, a switch the job never answers is gone once its time passed by the clock, and one still waiting when the run
//! ends is settled by the run's last note (edges 11 and 12).

use super::*;
use std::thread;

/// The microphone switched on or off.
const fn mic_on(id: u32) -> Note {
    let (switch, on) = (Switch::Mic, true);
    Note::Switched { id, switch, on }
}

/// The computer's sound switched off.
const fn sound_off(id: u32) -> Note {
    let (switch, on) = (Switch::Sound, false);
    Note::Switched { id, switch, on }
}

#[test]
fn a_switch_waits_for_its_answer_tells_nothing_and_the_run_goes_on() {
    let rig = Rig::new();
    let (mut run, heard) = rig.live();
    let answers: [(Says, Result<(), RecordError>); 3] = [
        (mic_on, Ok(())),
        (|id| fault(id, RecordFault::NoMic), Err(RecordError::NoMic)),
        (
            |id| fault(id, RecordFault::MicRefused),
            Err(RecordError::MicRefused),
        ),
    ];
    for (answer, want) in answers {
        rig.script.act(RecordAct::Say(vec![answer]));
        assert_eq!(run.mic(Some(rig.seal.pass())), want);
    }
    rig.script.act(RecordAct::Say(vec![sound_off]));
    assert_eq!(run.sound(false), Ok(()));
    let no_sound = |id| fault(id, RecordFault::NoSound);
    rig.script.act(RecordAct::Say(vec![no_sound]));
    assert_eq!(run.sound(true), Err(RecordError::NoSound));
    assert!(heard.try_recv().is_err(), "a switch's answer is its own");
    rig.script.act(RecordAct::Say(vec![stopped]));
    assert_eq!(run.stop(), Ok(()), "the run went on");
    let asks = rig.asks();
    assert_eq!(asks[1], Ask::Mic { id: 1, on: true });
    assert_eq!(asks[4], Ask::Sound { id: 1, on: false });
    assert_eq!(asks[5], Ask::Sound { id: 1, on: true });
}

#[test]
fn a_pass_of_another_seal_never_reaches_the_job() {
    let rig = Rig::new();
    let (mut run, _heard) = rig.live();
    let other = MicSeal::new().pass();
    assert_eq!(run.mic(Some(other)), Err(RecordError::BadPass));
    assert_eq!(rig.asks().len(), 1, "the start alone");
}

#[test]
fn a_switch_the_job_never_answers_is_gone_once_its_time_passed_by_the_clock() {
    let rig = Rig::new();
    let (mut run, heard) = rig.live();
    let (calls, asked) = mpsc::channel();
    rig.script.tell(calls);
    let pass = rig.seal.pass();
    // The thread hands the run back, so a note told after the gone switch still has a live run to tell it.
    let switching = thread::spawn(move || (run.mic(Some(pass)), run));
    assert!(matches!(sent(&asked), Ask::Mic { .. }));
    let half = ms(ask().waits.switch_ms) / 2;
    rig.clock.advance(half);
    rig.script.say(Ok(second(1)));
    assert_eq!(next(&heard), RecordNote::Time { ms: 1_000 });
    assert!(!switching.is_finished(), "still within its time");
    rig.clock.advance(half);
    rig.script.say(Ok(second(1)));
    let (switched, _run) = switching.join().unwrap();
    assert_eq!(switched, Err(RecordError::Gone));
    assert_eq!(
        next(&heard),
        RecordNote::Time { ms: 1_000 },
        "the run goes on"
    );
}

#[test]
fn a_switch_waiting_when_the_run_ends_is_settled_gone_by_the_runs_last_note() {
    let rig = Rig::new();
    let (mut run, heard) = rig.live();
    rig.script.act(RecordAct::Say(vec![longest]));
    assert_eq!(run.sound(false), Err(RecordError::Gone));
    let done = RecordDone {
        length_ms: LENGTH_MS,
        end: RecordEnd::Longest,
    };
    assert_eq!(
        next(&heard),
        RecordNote::Done(done),
        "edge 9: kept, and said"
    );
    assert_eq!(run.mic(None), Err(RecordError::Gone), "no run to switch");
    assert!(rig.folder.removes().is_empty());
}

#[test]
fn a_switch_its_run_ended_under_says_goodbye_so_no_watch_outlives_the_run() {
    let rig = Rig::new();
    let (mut run, _heard) = rig.live();
    let (calls, asked) = mpsc::channel();
    rig.script.tell(calls);
    rig.script.act(RecordAct::Say(vec![longest]));
    assert_eq!(run.sound(false), Err(RecordError::Gone));
    while asked.recv_timeout(LONG).expect("goodbye before the drop") != Call::Bye {}
    drop(run);
}
