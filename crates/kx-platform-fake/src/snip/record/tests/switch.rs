//! The microphone and sound switches: logged in step with the picture by the clock (edge 12), refused for a pass of
//! another seal or a device that cannot record (edge 11) with no fault told as the recording goes on, and failed once
//! as set.

use super::*;

/// A second.
const SECOND: Duration = Duration::from_secs(1);

/// What a recording hears from `seconds` in.
fn heard(seconds: u32, mic: bool, sound: bool) -> Heard {
    let ms = u32::try_from(SECOND.as_millis()).unwrap() * seconds;
    Heard { ms, mic, sound }
}

#[test]
fn switches_are_logged_in_step_with_the_picture() {
    let rig = Rig::new();
    let mut run = rig.start(ask(Some(rig.seal.pass()), false)).unwrap();
    rig.clock.advance(SECOND);
    assert_eq!(run.mic(None), Ok(()));
    rig.clock.advance(SECOND);
    assert_eq!(run.sound(true), Ok(()));
    rig.clock.advance(SECOND);
    assert_eq!(run.mic(Some(rig.seal.pass())), Ok(()));
    let want = [
        heard(0, true, false),
        heard(1, false, false),
        heard(2, false, true),
        heard(3, true, true),
    ];
    assert_eq!(rig.recorder.runs()[0].heard, want);
}

#[test]
fn a_pass_from_another_seal_opens_no_microphone() {
    let rig = Rig::new();
    let other = MicSeal::new().pass();
    let started = rig.start(ask(Some(other.clone()), false));
    assert_eq!(refusal(started), Some(RecordError::BadPass));
    let mut run = rig.start(ask(None, false)).unwrap();
    assert_eq!(run.mic(Some(other)), Err(RecordError::BadPass));
    assert_eq!(
        run.mic(Some(rig.seal.clone().pass())),
        Ok(()),
        "a clone's pass"
    );
    let runs = rig.recorder.runs();
    assert_eq!(runs.len(), 1, "the refused start made none");
    assert_eq!(
        runs[0].heard,
        [heard(0, false, false), heard(0, true, false)]
    );
}

#[test]
fn a_device_that_cannot_record_refuses_its_switch_on_but_not_off() {
    let rig = Rig::new();
    let mut run = rig.start(ask(Some(rig.seal.pass()), true)).unwrap();
    let lacking = RecordFacts {
        video: true,
        mic: Gear::Refused,
        sound: Gear::Missing,
    };
    rig.recorder.set_facts(lacking);
    assert_eq!(run.mic(Some(rig.seal.pass())), Err(RecordError::MicRefused));
    assert_eq!(run.sound(true), Err(RecordError::NoSound));
    assert_eq!(run.mic(None), Ok(()));
    assert_eq!(run.sound(false), Ok(()));
    let unplugged = RecordFacts {
        mic: Gear::Missing,
        ..lacking
    };
    rig.recorder.set_facts(unplugged);
    assert_eq!(run.mic(Some(rig.seal.pass())), Err(RecordError::NoMic));
    assert_eq!(rig.recorder.runs()[0].heard.len(), 3);
}

#[test]
fn a_refused_switch_tells_no_fault_and_the_recording_goes_on() {
    let rig = Rig::new();
    let mut run = rig.start(ask(None, false)).unwrap();
    let lacking = RecordFacts {
        video: true,
        mic: Gear::Missing,
        sound: Gear::Missing,
    };
    rig.recorder.set_facts(lacking);
    assert_eq!(run.mic(Some(rig.seal.pass())), Err(RecordError::NoMic));
    assert_eq!(run.sound(true), Err(RecordError::NoSound));
    assert!(rig.notes().is_empty(), "a switch's fault is its answer");
    assert!(rig.recorder.runs()[0].live);
    rig.clock.advance(SECOND);
    assert_eq!(run.stop(), Ok(()));
    assert!(matches!(rig.notes()[..], [RecordNote::Done(_)]));
    let length_ms = u32::try_from(SECOND.as_millis()).unwrap();
    let kept = Stored::Video { length_ms };
    assert_eq!(rig.folder.stored(run.file()), Some(kept));
}

#[test]
fn a_failed_switch_comes_once_and_logs_nothing() {
    let rig = Rig::new();
    let mut run = rig.start(ask(None, false)).unwrap();
    rig.recorder.fail(RecordCall::Mic, RecordError::Gone);
    rig.recorder.fail(RecordCall::Sound, RecordError::Failed);
    assert_eq!(run.mic(None), Err(RecordError::Gone));
    assert_eq!(run.sound(true), Err(RecordError::Failed));
    assert_eq!(rig.recorder.runs()[0].heard.len(), 1);
    assert_eq!((run.mic(None), run.sound(true)), (Ok(()), Ok(())), "once");
}

#[test]
fn a_lost_sound_is_told_heard_off_from_then_and_the_recording_goes_on() {
    let rig = Rig::new();
    let _run = rig.start(ask(Some(rig.seal.pass()), true)).unwrap();
    rig.clock.advance(SECOND);
    assert!(rig.recorder.tell(RecordNote::Lost(Source::Mic)));
    rig.clock.advance(SECOND);
    assert!(rig.recorder.tell(RecordNote::Lost(Source::Sound)));
    let want = [
        heard(0, true, true),
        heard(1, false, true),
        heard(2, false, false),
    ];
    let runs = rig.recorder.runs();
    assert_eq!(runs[0].heard, want);
    assert!(runs[0].live, "the run goes on");
    let lost = [
        RecordNote::Lost(Source::Mic),
        RecordNote::Lost(Source::Sound),
    ];
    assert_eq!(rig.notes(), lost);
    assert!(rig.recorder.time(), "it still hears its notes");
}
