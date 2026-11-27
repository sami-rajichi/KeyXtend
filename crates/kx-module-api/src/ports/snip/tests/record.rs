//! What the recorder is asked: the frames' own ranges and unit, each value refused past its ends, a listed speed of a
//! video only, a microphone pass only its own seal opens, and errors that say what happened.

use crate::ports::snip::{
    FileKind, MicPasses, MicSeal, Part, RecordAsk, RecordDone, RecordEnd, RecordError, RecordNote,
    RecordWaits, SnipFile, Source, Speed, SpeedAsk,
};
use crate::ports::{MB, Rect};
use kx_test_support::paths::native_path;
use std::collections::HashSet;

/// A plain ask: a 640 by 360 part at the first rate, both sounds off, every value at the start of its range.
fn ask() -> RecordAsk {
    let part = Part::new(Rect {
        x: 96,
        y: 130,
        w: 640,
        h: 360,
    });
    RecordAsk {
        part: part.unwrap(),
        fps: RecordAsk::FPS[0],
        mic: None,
        sound: false,
        max_min: *RecordAsk::MAX_MIN.start(),
        min_free_mb: *RecordAsk::MIN_FREE_MB.start(),
        job_mb: *RecordAsk::JOB_MB.start(),
        waits: waits(),
    }
}

/// Each wait at the start of its range.
pub(super) const fn waits() -> RecordWaits {
    RecordWaits {
        start_ms: *RecordWaits::START_MS.start(),
        tick_ms: *RecordWaits::TICK_MS.start(),
        end_ms: *RecordWaits::END_MS.start(),
        switch_ms: *RecordWaits::SWITCH_MS.start(),
        speed_ms: *RecordWaits::SPEED_MS.start(),
        speed_mb_ms: *RecordWaits::SPEED_MB_MS.start(),
    }
}

/// A video at `name` in the Snips folder, or a picture when `kind` says so.
fn file(name: &str, kind: FileKind) -> SnipFile {
    SnipFile::new(native_path(&format!(r"C:\Snips\unsaved\{name}")), kind).unwrap()
}

/// A 2x copy of one recording into a new video of the unsaved part.
fn speed() -> SpeedAsk {
    SpeedAsk {
        from: file("a.mp4", FileKind::Mp4),
        speed: Speed::new(2).unwrap(),
        min_free_mb: *RecordAsk::MIN_FREE_MB.start(),
        job_mb: *RecordAsk::JOB_MB.start(),
        waits: waits(),
    }
}

#[test]
fn the_ranges_and_their_unit_are_the_frames_own() {
    assert_eq!(RecordAsk::FPS, kx_ipc::SNIP_FPS, "one source of truth");
    assert_eq!(RecordAsk::MAX_MIN, kx_ipc::SNIP_MAX_MIN);
    assert_eq!(RecordAsk::MIN_FREE_MB, kx_ipc::SNIP_MIN_FREE_MB);
    assert_eq!(RecordAsk::JOB_MB, kx_ipc::SNIP_RECORD_MB);
    assert_eq!(SpeedAsk::SPEEDS, kx_ipc::SNIP_SPEEDS);
    let mib = u64::try_from(kx_ipc::MIB).unwrap();
    assert_eq!(
        MB, mib,
        "one MB for `min_free_mb`, `record_mb` and `shot_cap_mb`"
    );
}

/// `ask()` with `change` made.
fn changed(change: impl FnOnce(&mut RecordAsk)) -> RecordAsk {
    let mut ask = ask();
    change(&mut ask);
    ask
}

/// `speed()` with `change` made, then checked.
fn sped(change: impl FnOnce(&mut SpeedAsk)) -> Result<(), RecordError> {
    let mut ask = speed();
    change(&mut ask);
    ask.check()
}

#[test]
fn a_record_ask_refuses_each_value_outside_its_range() {
    assert_eq!(ask().check(), Ok(()));
    let unlisted = (0..=u8::MAX).find(|f| !RecordAsk::FPS.contains(f)).unwrap();
    let (min, free, job) = (
        RecordAsk::MAX_MIN,
        RecordAsk::MIN_FREE_MB,
        RecordAsk::JOB_MB,
    );
    let bad = [
        (changed(|a| a.fps = unlisted), "fps"),
        (changed(|a| a.max_min = min.start() - 1), "max_min"),
        (changed(|a| a.max_min = min.end() + 1), "max_min"),
        (changed(|a| a.min_free_mb = free.start() - 1), "min_free_mb"),
        (changed(|a| a.job_mb = job.end() + 1), "job_mb"),
    ];
    for (ask, name) in bad {
        assert_eq!(ask.check(), Err(RecordError::Range(name)));
    }
    let last = RecordAsk::FPS[RecordAsk::FPS.len() - 1];
    let ends = changed(|a| {
        (a.fps, a.max_min) = (last, *min.end());
        (a.min_free_mb, a.job_mb) = (*free.end(), *job.end());
    });
    assert_eq!(ends.check(), Ok(()), "each end is in");
}

#[test]
fn a_speed_is_only_one_the_frames_list() {
    let made: Vec<_> = (0..=5).filter_map(Speed::new).map(Speed::times).collect();
    assert_eq!(made, SpeedAsk::SPEEDS);
    assert_eq!(Speed::new(u8::MAX), None);
}

#[test]
fn the_plain_speed_is_the_first_listed_which_plays_as_recorded() {
    assert_eq!(Speed::new(SpeedAsk::SPEEDS[0]), Some(Speed::PLAIN));
    assert_eq!(Speed::PLAIN.times(), 1, "1 plays it as it is");
}

#[test]
fn each_next_speed_steps_through_the_list_then_back_to_as_recorded() {
    let mut at = Speed::PLAIN;
    for &times in SpeedAsk::SPEEDS[1..].iter().chain(&SpeedAsk::SPEEDS[..1]) {
        at = at.next();
        assert_eq!(at.times(), times);
    }
    assert_eq!(at, Speed::PLAIN, "a full round plays it as recorded");
}

#[test]
fn a_speed_ask_takes_only_a_video() {
    assert_eq!(speed().check(), Ok(()));
    let free = RecordAsk::MIN_FREE_MB;
    let bad = [
        sped(|a| a.job_mb = RecordAsk::JOB_MB.start() - 1),
        sped(|a| a.min_free_mb = free.start() - 1),
        sped(|a| a.min_free_mb = free.end() + 1),
        sped(|a| a.from = file("a.png", FileKind::Png)),
    ];
    let want = [
        Err(RecordError::Range("job_mb")),
        Err(RecordError::Range("min_free_mb")),
        Err(RecordError::Range("min_free_mb")),
        Err(RecordError::BadPath),
    ];
    assert_eq!(bad, want);
}

#[test]
fn a_lost_sound_names_which_one_and_never_ends_the_run() {
    let (mic, sound) = (
        RecordNote::Lost(Source::Mic),
        RecordNote::Lost(Source::Sound),
    );
    assert_ne!(mic, sound);
    assert!(!mic.ends() && !sound.ends(), "the run goes on with it off");
    let done = RecordDone {
        length_ms: 1,
        end: RecordEnd::Stopped,
    };
    assert!(RecordNote::Done(done).ends());
    assert!(RecordNote::Fault(RecordError::Gone).ends());
    assert!(!RecordNote::Time { ms: 0 }.ends());
}

#[test]
fn a_seal_opens_only_the_passes_it_made() {
    let (app, own) = (MicSeal::new(), MicSeal::new());
    let gate: &dyn MicPasses = &app;
    let pass = gate.pass();
    assert!(
        app.opens(&pass),
        "the gate's pass opens the recorder's seal"
    );
    assert!(
        !own.opens(&pass) && !app.opens(&own.pass()),
        "a module's own seal opens nothing"
    );
    assert_eq!(pass, app.clone().pass(), "one seal's passes are alike");
    assert_ne!(pass, own.pass());
    let heard = changed(|a| a.mic = Some(pass));
    assert_eq!(heard.check(), Ok(()));
}

#[test]
fn record_errors_each_say_something_else_and_name_no_path() {
    let errors = [
        RecordError::Range("fps"),
        RecordError::BadPath,
        RecordError::NoFolder,
        RecordError::NoVideo,
        RecordError::NoMic,
        RecordError::MicRefused,
        RecordError::BadPass,
        RecordError::NoSound,
        RecordError::Full,
        RecordError::Memory,
        RecordError::Failed,
        RecordError::Gone,
    ];
    let texts: HashSet<_> = errors.iter().map(ToString::to_string).collect();
    assert_eq!(texts.len(), errors.len(), "{texts:?}");
    assert!(texts.iter().all(|t| !t.contains('\\')), "{texts:?}");
    assert!(RecordError::Range("fps").to_string().contains("fps"));
}
