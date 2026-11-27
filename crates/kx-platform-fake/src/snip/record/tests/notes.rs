//! A recording's notes: the time and the end by the clock to the newest live one, a death that deletes the broken
//! video (edge 13), Stop, Cancel (edge 10) and a drop, each as the port says.

use super::*;

/// A second.
const SECOND: Duration = Duration::from_secs(1);
/// A longer step.
const STEP: Duration = Duration::from_millis(2_500);
/// A time told by hand.
const TOLD_MS: u32 = 7;

/// `d` in whole ms.
fn ms(d: Duration) -> u32 {
    u32::try_from(d.as_millis()).unwrap()
}

/// A video `d` long.
fn video(d: Duration) -> Stored {
    Stored::Video { length_ms: ms(d) }
}

#[test]
fn the_time_and_the_end_come_by_the_clock_and_keep_the_video() {
    let rig = Rig::new();
    let run = rig.start(ask(None, false)).unwrap();
    rig.clock.advance(SECOND);
    assert!(rig.recorder.time());
    rig.clock.advance(STEP);
    assert!(rig.recorder.end(RecordEnd::Longest));
    let done = RecordDone {
        length_ms: ms(SECOND + STEP),
        end: RecordEnd::Longest,
    };
    let time = RecordNote::Time { ms: ms(SECOND) };
    assert_eq!(rig.notes(), [time, RecordNote::Done(done)]);
    assert_eq!(rig.folder.stored(run.file()), Some(video(SECOND + STEP)));
    assert!(!rig.recorder.time(), "none hears after its end");
    let log = &rig.recorder.runs()[0];
    assert_eq!((log.live, log.told.len()), (false, 2));
}

#[test]
fn notes_go_to_the_newest_live_recording_only() {
    let rig = Rig::new();
    assert!(!rig.recorder.time(), "none yet");
    let _first = rig.start(ask(None, false)).unwrap();
    let _second = rig.start(ask(None, false)).unwrap();
    let told = RecordNote::Time { ms: TOLD_MS };
    assert!(rig.recorder.tell(told));
    assert!(rig.recorder.end(RecordEnd::Full));
    assert!(rig.recorder.tell(told));
    let runs = rig.recorder.runs();
    assert_eq!(runs[0].told, [told]);
    assert_eq!(runs[1].told.len(), 2);
}

#[test]
fn a_dead_worker_tells_a_fault_and_deletes_the_broken_video() {
    let rig = Rig::new();
    let mut run = rig.start(ask(None, false)).unwrap();
    assert!(rig.recorder.die());
    assert_eq!(rig.notes(), [RecordNote::Fault(RecordError::Gone)]);
    assert_eq!(rig.folder.stored(run.file()), None);
    assert_eq!(run.stop(), Err(RecordError::Gone));
    assert_eq!(run.sound(true), Err(RecordError::Gone));
    assert!(!rig.recorder.die(), "no one left to hear");
    let _next = rig.start(ask(None, false)).unwrap();
    assert!(rig.recorder.tell(RecordNote::Fault(RecordError::Memory)));
    assert!(rig.folder.files().is_empty(), "any fault deletes its video");
}

#[test]
fn stop_tells_the_end_with_the_length_and_keeps_the_video() {
    let rig = Rig::new();
    let mut run = rig.start(ask(None, false)).unwrap();
    rig.clock.advance(STEP);
    assert_eq!(run.stop(), Ok(()));
    let done = RecordDone {
        length_ms: ms(STEP),
        end: RecordEnd::Stopped,
    };
    assert_eq!(rig.notes(), [RecordNote::Done(done)]);
    assert_eq!(rig.folder.stored(run.file()), Some(video(STEP)));
    assert_eq!(run.stop(), Err(RecordError::Gone), "it ended");
    let log = &rig.recorder.runs()[0];
    assert_eq!((log.stops, log.live), (1, false));
}

#[test]
fn a_failed_stop_comes_once_and_changes_nothing_else() {
    let rig = Rig::new();
    let mut run = rig.start(ask(None, false)).unwrap();
    rig.recorder.fail(RecordCall::Stop, RecordError::Gone);
    assert_eq!(run.stop(), Err(RecordError::Gone));
    assert!(rig.notes().is_empty());
    assert!(rig.recorder.runs()[0].live);
    assert_eq!(run.stop(), Ok(()), "once");
}

#[test]
fn cancel_deletes_the_video_and_tells_nothing_more() {
    let rig = Rig::new();
    let mut run = rig.start(ask(None, false)).unwrap();
    rig.clock.advance(STEP);
    run.cancel();
    assert_eq!(rig.folder.stored(run.file()), None);
    assert!(!rig.recorder.time());
    assert!(rig.notes().is_empty());
    let log = &rig.recorder.runs()[0];
    assert_eq!((log.cancels, log.live), (1, false));
}

#[test]
fn cancel_after_the_end_leaves_the_kept_video_alone() {
    let rig = Rig::new();
    let mut run = rig.start(ask(None, false)).unwrap();
    rig.clock.advance(STEP);
    run.stop().unwrap();
    run.cancel();
    assert_eq!(rig.folder.stored(run.file()), Some(video(STEP)), "kept");
    assert_eq!(rig.notes().len(), 1, "only the end");
    assert_eq!(rig.recorder.runs()[0].cancels, 1, "the ask counts");
}

#[test]
fn a_dropped_recording_keeps_its_video_so_far_for_the_sweep() {
    let rig = Rig::new();
    let run = rig.start(ask(None, false)).unwrap();
    let file = run.file().clone();
    rig.clock.advance(STEP);
    drop(run);
    assert_eq!(rig.folder.stored(&file), Some(video(STEP)));
    assert_eq!(rig.folder.unsaved().unwrap().len(), 1);
    let log = &rig.recorder.runs()[0];
    assert_eq!((log.dropped, log.live), (true, false));
    assert!(rig.notes().is_empty());
}

#[test]
fn a_drop_after_the_end_keeps_the_ends_length() {
    let rig = Rig::new();
    let mut run = rig.start(ask(None, false)).unwrap();
    rig.clock.advance(STEP);
    run.stop().unwrap();
    let file = run.file().clone();
    rig.clock.advance(SECOND);
    drop(run);
    assert_eq!(rig.folder.stored(&file), Some(video(STEP)));
    assert!(rig.recorder.runs()[0].dropped);
}
