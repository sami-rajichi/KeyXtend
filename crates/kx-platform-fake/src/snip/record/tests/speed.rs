//! Re-timed copies: a new video of the unsaved part whose length divides by the speed, rounded up so a very short
//! video still gives one (edge 20), refused as the port says, every ask kept.

use super::*;
use crate::snip::tests::{SNIPS, file};
use kx_module_api::ports::{FileKind, Speed, SpeedAsk};

/// A recording's length, odd so a division rounds.
const LONG_MS: u32 = 2_501;
/// The shortest video there is.
const SHORT_MS: u32 = 1;

/// A copy of `from` at `times` with the least memory cap.
fn speed_ask(from: SnipFile, times: u8) -> SpeedAsk {
    SpeedAsk {
        from,
        speed: Speed::new(times).unwrap(),
        min_free_mb: *RecordAsk::MIN_FREE_MB.start(),
        job_mb: *RecordAsk::JOB_MB.start(),
        waits: waits(),
    }
}

/// A recording `length_ms` long, named `name`, held by `rig`'s folder in `path`.
fn recorded(rig: &Rig, path: &str, name: &str, length_ms: u32) -> SnipFile {
    let video = file(path, name, FileKind::Mp4);
    rig.folder.put(video.clone(), Stored::Video { length_ms });
    video
}

/// The fastest speed.
fn fastest() -> u8 {
    *SpeedAsk::SPEEDS.iter().max().unwrap()
}

#[test]
fn a_copy_is_a_new_video_of_the_unsaved_part_its_length_divided() {
    let rig = Rig::new();
    let long = recorded(&rig, UNSAVED, "a.mp4", LONG_MS);
    for times in SpeedAsk::SPEEDS {
        let copy = rig.recorder.speed(&speed_ask(long.clone(), times)).unwrap();
        assert!(copy.lies_in(&dir(UNSAVED)) && copy != long);
        let length_ms = LONG_MS.div_ceil(u32::from(times));
        assert_eq!(rig.folder.stored(&copy), Some(Stored::Video { length_ms }));
    }
    let short = recorded(&rig, UNSAVED, "b.mp4", SHORT_MS);
    let copy = rig.recorder.speed(&speed_ask(short, fastest())).unwrap();
    let still = Stored::Video {
        length_ms: SHORT_MS,
    };
    assert_eq!(
        rig.folder.stored(&copy),
        Some(still),
        "a very short one still"
    );
    let kept = Stored::Video { length_ms: LONG_MS };
    assert_eq!(rig.folder.stored(&long), Some(kept), "the source stays");
    assert_eq!(rig.recorder.speeds().len(), SpeedAsk::SPEEDS.len() + 1);
}

#[test]
fn a_speed_ask_is_refused_as_the_port_says_and_kept() {
    let rig = Rig::new();
    let long = recorded(&rig, UNSAVED, "a.mp4", LONG_MS);
    let elsewhere = recorded(&rig, SNIPS, "b.mp4", LONG_MS);
    let gone = file(UNSAVED, "c.mp4", FileKind::Mp4);
    let picture = file(UNSAVED, "d.png", FileKind::Png);
    let big = SpeedAsk {
        job_mb: *RecordAsk::JOB_MB.end() + 1,
        ..speed_ask(long.clone(), fastest())
    };
    let refusals = [
        (speed_ask(picture, fastest()), RecordError::BadPath),
        (big, RecordError::Range("job_mb")),
        (speed_ask(elsewhere, fastest()), RecordError::BadPath),
        (speed_ask(gone, fastest()), RecordError::BadPath),
    ];
    for (asked, error) in &refusals {
        assert_eq!(rig.recorder.speed(asked), Err(*error));
    }
    let one = speed_ask(long, fastest());
    rig.recorder.fail(RecordCall::Speed, RecordError::Full);
    assert_eq!(rig.recorder.speed(&one), Err(RecordError::Full));
    rig.recorder.set_facts(RecordFacts {
        video: false,
        mic: Gear::Ready,
        sound: Gear::Ready,
    });
    assert_eq!(rig.recorder.speed(&one), Err(RecordError::NoVideo));
    assert_eq!(rig.folder.files().len(), 2, "no copy made");
    assert_eq!(rig.recorder.speeds().len(), refusals.len() + 2);
}

#[test]
fn a_copy_under_its_floor_is_full_and_one_at_it_is_made() {
    let rig = Rig::new();
    let long = recorded(&rig, UNSAVED, "a.mp4", LONG_MS);
    let asked = speed_ask(long, fastest());
    let floor = u64::from(asked.min_free_mb) * MB;
    rig.folder.set_free(Ok(floor - 1));
    assert_eq!(rig.recorder.speed(&asked), Err(RecordError::Full));
    assert_eq!(rig.folder.files().len(), 1, "no copy made");
    let facts = rig.recorder.facts();
    rig.recorder.set_facts(RecordFacts {
        video: false,
        ..facts
    });
    let first = rig.recorder.speed(&asked);
    assert_eq!(first, Err(RecordError::Full), "room first, as the worker's");
    rig.recorder.set_facts(facts);
    rig.folder.set_free(Ok(floor));
    assert!(rig.recorder.speed(&asked).is_ok());
}

#[test]
fn a_test_waits_for_a_copy_made_on_another_thread() {
    let rig = Rig::new();
    let long = recorded(&rig, UNSAVED, "a.mp4", LONG_MS);
    assert!(!rig.recorder.wait_speeds(1, Duration::ZERO), "none yet");
    let port: Arc<dyn Recorder> = Arc::new(rig.recorder.clone());
    let asked = speed_ask(long, fastest());
    let module = thread::spawn(move || port.speed(&asked).is_ok());
    assert!(rig.recorder.wait_speeds(1, WITHIN));
    assert!(module.join().unwrap());
}

#[test]
fn a_hidden_run_copies_nothing() {
    let clock = FakeClock::new(Mono::default());
    let hidden = FakeSnipFolder::hidden(Arc::new(clock));
    let recorder = FakeRecorder::new(MicSeal::new(), &hidden);
    let video = file(UNSAVED, "a.mp4", FileKind::Mp4);
    let asked = speed_ask(video, fastest());
    assert_eq!(recorder.speed(&asked), Err(RecordError::NoFolder));
}
