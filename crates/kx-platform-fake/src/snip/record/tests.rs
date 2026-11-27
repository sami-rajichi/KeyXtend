//! The fake recorder, held to the port: each ask checked in the port's order, a new video of the unsaved part per
//! start, the facts as set, a failure once, and a test that waits on another thread. Notes, switches and speed copies
//! are in their own files.

use super::*;
use crate::FakeClock;
use crate::snip::tests::{UNSAVED, WITHIN, dir, file, rig};
use kx_module_api::Mono;
use kx_module_api::ports::{
    FileKind, Gear, MB, MicPass, MicPasses, Part, RecordFacts, RecordWaits, Rect, SaveError,
    SnipFolder, Source, Speed, UnsavedFile,
};
use std::sync::Mutex;
use std::thread;

mod notes;
mod speed;
mod switch;

/// Leftover videos already in the unsaved part, named as the fake names its own.
const LEFTOVERS: u32 = 3;
/// Their length.
const OLD_MS: u32 = 9;
/// How long ago they were written.
const AGE: Duration = Duration::from_hours(1);

/// The part filmed.
const PART: Rect = Rect {
    x: 10,
    y: 20,
    w: 640,
    h: 360,
};

/// The notes the module heard, shared with the test.
type Notes = Arc<Mutex<Vec<RecordNote>>>;

/// A recorder over the shared folder, its seal, the clock, and the notes heard.
struct Rig {
    clock: FakeClock,
    folder: FakeSnipFolder,
    seal: MicSeal,
    recorder: FakeRecorder,
    notes: Notes,
}

impl Rig {
    /// A folder at the clock's start and a recorder holding a new seal.
    fn new() -> Self {
        let (clock, folder) = rig();
        let seal = MicSeal::new();
        let recorder = FakeRecorder::new(seal.clone(), &folder);
        let notes = Notes::default();
        Self {
            clock,
            folder,
            seal,
            recorder,
            notes,
        }
    }

    /// A callback that keeps each note heard.
    fn tell(&self) -> Box<dyn Fn(RecordNote) + Send> {
        let notes = Arc::clone(&self.notes);
        Box::new(move |note| crate::lock(&notes).push(note))
    }

    /// The notes heard so far.
    fn notes(&self) -> Vec<RecordNote> {
        crate::lock(&self.notes).clone()
    }

    /// Starts `ask`.
    fn start(&self, ask: RecordAsk) -> Result<Box<dyn RecordRun>, RecordError> {
        self.recorder.start(ask, self.tell())
    }
}

/// The least of every range, the microphone on with `mic` and the computer's sound as `sound`.
fn ask(mic: Option<MicPass>, sound: bool) -> RecordAsk {
    RecordAsk {
        part: Part::new(PART).unwrap(),
        fps: RecordAsk::FPS[0],
        mic,
        sound,
        max_min: *RecordAsk::MAX_MIN.start(),
        min_free_mb: *RecordAsk::MIN_FREE_MB.start(),
        job_mb: *RecordAsk::JOB_MB.start(),
        waits: waits(),
    }
}

/// Each job wait at the start of its range.
const fn waits() -> RecordWaits {
    RecordWaits {
        start_ms: *RecordWaits::START_MS.start(),
        tick_ms: *RecordWaits::TICK_MS.start(),
        end_ms: *RecordWaits::END_MS.start(),
        switch_ms: *RecordWaits::SWITCH_MS.start(),
        speed_ms: *RecordWaits::SPEED_MS.start(),
        speed_mb_ms: *RecordWaits::SPEED_MB_MS.start(),
    }
}

/// The error a start gave, or `None` for a recording.
fn refusal(started: Result<Box<dyn RecordRun>, RecordError>) -> Option<RecordError> {
    started.err()
}

#[test]
fn a_start_writes_a_new_video_of_the_unsaved_part_and_logs_its_ask() {
    let rig = Rig::new();
    let asked = ask(Some(rig.seal.pass()), false);
    let first = rig.start(asked.clone()).unwrap();
    let second = rig.start(ask(None, true)).unwrap();
    assert!(first.file().lies_in(&dir(UNSAVED)));
    assert_ne!(first.file(), second.file(), "each its own video");
    let empty = Stored::Video { length_ms: 0 };
    assert_eq!(rig.folder.stored(first.file()), Some(empty));
    let runs = rig.recorder.runs();
    assert_eq!(
        (runs.len(), &runs[0].ask, &runs[0].file),
        (2, &asked, first.file())
    );
    let at_start = Heard {
        ms: 0,
        mic: true,
        sound: false,
    };
    assert_eq!(runs[0].heard, [at_start]);
    assert!(runs.iter().all(|r| r.live && r.told.is_empty()));
}

#[test]
fn each_lack_refuses_a_start_in_the_ports_order_and_makes_no_video() {
    let rig = Rig::new();
    let pass = Some(rig.seal.pass());
    let bad_fps = RecordAsk {
        fps: 0,
        ..ask(None, false)
    };
    assert_eq!(refusal(rig.start(bad_fps)), Some(RecordError::Range("fps")));
    let lacks = [
        (true, Gear::Missing, Gear::Ready, RecordError::NoMic),
        (true, Gear::Refused, Gear::Missing, RecordError::MicRefused),
        (true, Gear::Ready, Gear::Missing, RecordError::NoSound),
        (false, Gear::Missing, Gear::Missing, RecordError::NoVideo),
    ];
    for (video, mic, sound, error) in lacks {
        rig.recorder.set_facts(RecordFacts { video, mic, sound });
        assert_eq!(refusal(rig.start(ask(pass.clone(), true))), Some(error));
    }
    let off = RecordFacts {
        video: true,
        mic: Gear::Missing,
        sound: Gear::Refused,
    };
    rig.recorder.set_facts(off);
    assert!(rig.start(ask(None, false)).is_ok(), "devices left off");
    assert_eq!(rig.recorder.runs().len(), 1);
    assert_eq!(rig.folder.files().len(), 1);
}

#[test]
fn a_start_needs_its_room_and_a_folder() {
    let rig = Rig::new();
    let least = u64::from(*RecordAsk::MIN_FREE_MB.start()) * MB;
    rig.folder.set_free(Ok(least - 1));
    assert_eq!(
        refusal(rig.start(ask(None, false))),
        Some(RecordError::Full)
    );
    let facts = rig.recorder.facts();
    rig.recorder.set_facts(RecordFacts {
        video: false,
        ..facts
    });
    let first = refusal(rig.start(ask(None, false)));
    assert_eq!(
        first,
        Some(RecordError::Full),
        "room first, as the worker's"
    );
    rig.recorder.set_facts(facts);
    rig.folder.set_free(Err(SaveError::Io));
    let unread = refusal(rig.start(ask(None, false)));
    assert_eq!(unread, Some(RecordError::Failed), "room unread");
    rig.folder.set_free(Ok(least));
    assert!(rig.start(ask(None, false)).is_ok(), "exactly the floor");
    let clock = FakeClock::new(Mono::default());
    let hidden = FakeSnipFolder::hidden(Arc::new(clock));
    let recorder = FakeRecorder::new(rig.seal.clone(), &hidden);
    let started = recorder.start(ask(None, false), rig.tell());
    assert_eq!(refusal(started), Some(RecordError::NoFolder));
}

#[test]
fn every_start_is_logged_refused_ones_too() {
    let rig = Rig::new();
    let foreign = ask(Some(MicSeal::new().pass()), false);
    assert_eq!(
        refusal(rig.start(foreign.clone())),
        Some(RecordError::BadPass)
    );
    rig.folder.set_free(Ok(0));
    assert_eq!(refusal(rig.start(ask(None, true))), Some(RecordError::Full));
    assert_eq!(rig.recorder.starts(), [foreign, ask(None, true)]);
    assert!(rig.recorder.runs().is_empty(), "none recorded");
}

#[test]
fn a_new_video_never_takes_the_name_of_a_file_held() {
    let rig = Rig::new();
    let named = |n: u32| file(UNSAVED, &format!("{n}.mp4"), FileKind::Mp4);
    let held: Vec<_> = (1..=LEFTOVERS).map(named).collect();
    let old = Stored::Video { length_ms: OLD_MS };
    for file in &held {
        rig.folder.put(file.clone(), old.clone());
    }
    rig.clock.advance(AGE);
    let run = rig.start(ask(None, false)).unwrap();
    let asked = SpeedAsk {
        from: held[0].clone(),
        speed: Speed::new(SpeedAsk::SPEEDS[0]).unwrap(),
        min_free_mb: *RecordAsk::MIN_FREE_MB.start(),
        job_mb: *RecordAsk::JOB_MB.start(),
        waits: waits(),
    };
    let copy = rig.recorder.speed(&asked).unwrap();
    assert!(
        !held.contains(run.file()) && !held.contains(&copy),
        "new names"
    );
    let listed = rig.folder.unsaved().unwrap();
    for file in &held {
        assert_eq!(rig.folder.stored(file), Some(old.clone()), "kept");
        let aged = UnsavedFile {
            file: file.clone(),
            age: AGE,
            copied: None,
        };
        assert!(listed.contains(&aged), "with its age");
    }
    assert_eq!(listed.len(), held.len() + 2);
}

#[test]
fn a_failure_comes_once_and_the_facts_as_set() {
    let rig = Rig::new();
    let ready = RecordFacts {
        video: true,
        mic: Gear::Ready,
        sound: Gear::Ready,
    };
    assert_eq!(rig.recorder.facts(), ready);
    rig.recorder.fail(RecordCall::Start, RecordError::Gone);
    assert_eq!(
        refusal(rig.start(ask(None, false))),
        Some(RecordError::Gone)
    );
    assert!(rig.start(ask(None, false)).is_ok(), "once");
    let lacking = RecordFacts {
        video: false,
        ..ready
    };
    rig.recorder.set_facts(lacking);
    assert_eq!(rig.recorder.facts(), lacking);
}

#[test]
fn a_test_waits_for_a_start_on_another_thread() {
    let rig = Rig::new();
    assert!(!rig.recorder.wait(Duration::ZERO, |runs| !runs.is_empty()));
    let port: Arc<dyn Recorder> = Arc::new(rig.recorder.clone());
    let tell = rig.tell();
    let module = thread::spawn(move || port.start(ask(None, false), tell).is_ok());
    assert!(rig.recorder.wait(WITHIN, |runs| runs.len() == 1));
    assert!(module.join().unwrap());
}

#[test]
fn debug_shows_counts_and_no_path() {
    let rig = Rig::new();
    let _run = rig.start(ask(None, false)).unwrap();
    let shown = format!("{:?}", rig.recorder);
    assert!(shown.contains("FakeRecorder"), "{shown}");
    assert!(!shown.contains("Alex"), "{shown}");
    assert!(shown.contains("live: 1"), "{shown}");
}
