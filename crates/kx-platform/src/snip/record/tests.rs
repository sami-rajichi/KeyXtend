//! The recorder host over scripted record workers and the fake Snips folder, on a clock that moves only when a test
//! moves it: each run and copy in a job of its own, the ports' refusals before any job, notes told in the job's order,
//! switches answered, and a lost job ended with one fault; nothing sleeps.

use super::copy::deadline;
use super::*;
use crate::UNSAVED_DIR;
use crate::worker::fake::{Call, Hands, RecordAct, Script};
use kx_ipc::{Area, RecordAsk as Ask, RecordEnd as End, RecordFault, RecordNote as Note, Switch};
use kx_module_api::ports::{
    FileKind, Gear, MicPasses, Part, RecordDone, RecordEnd, Rect, Source, Speed,
};
use kx_platform_fake::{FakeSnipFolder, Stored};
use kx_test_support::paths::native_path;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

mod cancel;
mod copy;
mod lost;
mod start;
mod switch;

/// A note the scripted job says, made for its run's id.
type Says = fn(u32) -> Note;

/// How long a test waits for a note or a call; a wait, never a sleep.
const LONG: Duration = Duration::from_secs(10);
/// The Snips folder, under a neutral user's folder; paths here are written for Windows, joined by `/` as `Path::join`
/// writes `\` there, and read through `native_path`.
const SNIPS: &str = "C:/Users/someone/KeyXtend/snips";
/// The length a test run ends with.
const LENGTH_MS: u32 = 2_500;
/// The exit code of a worker that died.
const DIED: u32 = 3;
/// The part filmed.
const PART: Rect = Rect {
    x: -10,
    y: 20,
    w: 640,
    h: 360,
};

/// What the system can record with, as a test sets it.
struct Facts(RecordFacts);

impl Kit for Facts {
    fn facts(&self) -> RecordFacts {
        self.0
    }
}

/// Every device ready.
const READY: RecordFacts = RecordFacts {
    video: true,
    mic: Gear::Ready,
    sound: Gear::Ready,
};

/// A recorder over scripted workers and the fake folder, the app's seal, and what its runs told.
struct Rig {
    script: Script<RecordFrames>,
    clock: Hands,
    folder: FakeSnipFolder,
    seal: MicSeal,
    recorder: Arc<WorkerRecorder<Script<RecordFrames>>>,
}

impl Rig {
    fn new() -> Self {
        Self::with(|folder, _| Arc::new(folder.clone()))
    }

    /// A rig whose recorder sees the folder `wrap` makes of the fake one, over the script.
    fn with(
        wrap: impl FnOnce(&FakeSnipFolder, &Script<RecordFrames>) -> Arc<dyn SnipFolder>,
    ) -> Self {
        let (script, clock) = (Script::default(), Hands::default());
        let folder = folder(&clock);
        let seal = MicSeal::new();
        let snips = wrap(&folder, &script);
        let kit = Arc::new(Facts(READY));
        let recorder = Arc::new(WorkerRecorder::new(
            script.clone(),
            Arc::new(clock.clone()),
            seal.clone(),
            snips,
            kit,
        ));
        Self {
            script,
            clock,
            folder,
            seal,
            recorder,
        }
    }

    /// Starts `ask`, the job saying `notes` to its start; the run and what it tells.
    fn start(
        &self,
        ask: RecordAsk,
        notes: Vec<fn(u32) -> Note>,
    ) -> (
        Result<Box<dyn RecordRun>, RecordError>,
        Receiver<RecordNote>,
    ) {
        self.script.act(RecordAct::Say(notes));
        let (tell, heard) = ear();
        (self.recorder.start(ask, tell), heard)
    }

    /// A run started at once, with what it tells.
    fn live(&self) -> (Box<dyn RecordRun>, Receiver<RecordNote>) {
        let (run, heard) = self.start(ask(), vec![started]);
        let run = run.unwrap();
        assert_eq!(next(&heard), RecordNote::Time { ms: 0 });
        (run, heard)
    }

    /// Drops `run` and waits until its job is told goodbye and ended, as the script fakes one worker at a time.
    fn close(&self, run: Box<dyn RecordRun>) {
        let (calls, heard) = mpsc::channel();
        self.script.tell(calls);
        drop(run);
        while heard.recv_timeout(LONG).expect("ended in time") != Call::End {}
    }

    /// The asks the record jobs were sent, in order.
    fn asks(&self) -> Vec<Ask<'static>> {
        let calls = self.script.calls().into_iter();
        calls
            .filter_map(|c| match c {
                Call::Record(ask) => Some(ask),
                _ => None,
            })
            .collect()
    }
}

/// The unsaved part of `SNIPS`.
fn unsaved() -> FolderPath {
    FolderPath::new(native_path(&format!("{SNIPS}/{UNSAVED_DIR}"))).unwrap()
}

/// The file `name` in the unsaved part.
fn unsaved_at(name: &str) -> PathBuf {
    native_path(&format!("{SNIPS}/{UNSAVED_DIR}/{name}"))
}

/// The fake Snips folder with its unsaved part, reading `clock`.
fn folder(clock: &Hands) -> FakeSnipFolder {
    let snips = FolderPath::new(native_path(SNIPS)).unwrap();
    FakeSnipFolder::new(Arc::new(clock.clone()), snips, unsaved())
}

/// A callback that hands each note on, and where they arrive.
fn ear() -> (Box<dyn Fn(RecordNote) + Send>, Receiver<RecordNote>) {
    let (said, heard) = mpsc::channel();
    (
        Box::new(move |note| {
            let _ = said.send(note);
        }),
        heard,
    )
}

/// The next ask a record job was sent, past the host's other calls.
fn sent(calls: &Receiver<Call>) -> Ask<'static> {
    loop {
        if let Call::Record(ask) = calls.recv_timeout(LONG).expect("sent in time") {
            return ask;
        }
    }
}

/// The next note told.
fn next(heard: &Receiver<RecordNote>) -> RecordNote {
    heard.recv_timeout(LONG).expect("told in time")
}

/// A recording at 30 fps of `PART`, both sounds off, the floor and cap at their least.
fn ask() -> RecordAsk {
    RecordAsk {
        part: Part::new(PART).unwrap(),
        fps: RecordAsk::FPS[1],
        mic: None,
        sound: true,
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

/// The first time, which says the run started.
const fn started(id: u32) -> Note {
    Note::Time { id, ms: 0 }
}

/// A second in.
const fn second(id: u32) -> Note {
    Note::Time { id, ms: 1_000 }
}

/// The run ended on Stop.
const fn stopped(id: u32) -> Note {
    let (length_ms, end) = (LENGTH_MS, End::Stopped);
    Note::End { id, length_ms, end }
}

/// The run reached `max_min` (edge 9).
const fn longest(id: u32) -> Note {
    let (length_ms, end) = (LENGTH_MS, End::Longest);
    Note::End { id, length_ms, end }
}

/// A fault of the job.
const fn fault(id: u32, fault: RecordFault) -> Note {
    Note::Fault { id, fault }
}

/// The run was killed by a failure inside the job.
const fn failed(id: u32) -> Note {
    fault(id, RecordFault::Failed)
}

/// The microphone stopped by itself.
const fn mic_lost(id: u32) -> Note {
    let switch = Switch::Mic;
    Note::Lost { id, switch }
}

#[test]
fn the_facts_are_the_kits() {
    let rig = Rig::new();
    assert_eq!(rig.recorder.facts(), READY);
}

#[test]
fn a_fresh_name_is_counted_from_its_start_past_every_video_held() {
    let at = |n: u64| {
        let path = unsaved_at(&format!("{n}.{}", FileKind::Mp4.ext()));
        SnipFile::new(path, FileKind::Mp4).unwrap()
    };
    assert_eq!(fresh(&unsaved(), 7, &[]), Some((7, at(7))));
    let held = [at(7), at(8), at(10)];
    assert_eq!(fresh(&unsaved(), 7, &held), Some((9, at(9))));
    assert_eq!(fresh(&unsaved(), u64::MAX, &[at(u64::MAX)]), None);
}

#[test]
fn a_copys_deadline_grows_with_each_mb_begun_of_its_source() {
    let w = waits();
    let (least, per) = (ms(w.speed_ms), ms(w.speed_mb_ms));
    assert_eq!(deadline(&w, 0), least, "a size it cannot read adds none");
    assert_eq!(deadline(&w, 1), least + per);
    assert_eq!(deadline(&w, MB), least + per);
    assert_eq!(deadline(&w, MB + 1), least + per * 2);
    assert!(deadline(&w, u64::MAX) > least, "it saturates");
}
