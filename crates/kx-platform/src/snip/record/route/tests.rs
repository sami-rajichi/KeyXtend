//! `Route` over random interleavings of its job's notes, a lost job, its owner's switches, Cancel and drop, and the
//! clock, then its goodbye: one ending at most is told and nothing after a cancel, each switch is answered once, and
//! the video goes unless an end kept it for an owner who heard it, or the owner dropped a run never cancelled.

use super::*;
use crate::worker::Lost;
use crate::worker::fake::Hands;
use kx_ipc::RecordEnd as End;
use kx_module_api::ports::{FileKind, FolderPath, RecordWaits};
use kx_platform_fake::{FakeSnipFolder, Stored};
use kx_test_support::paths::native_path;
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::sample::select;
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

/// The run's id, which the route never reads.
const ID: u32 = 7;
/// How long a switch waits by the clock: the start of its range.
const SWITCH_MS: u32 = *RecordWaits::SWITCH_MS.start();
/// The Snips folder, under a neutral user's folder, written for Windows and joined by `/` (`native_path`).
const SNIPS: &str = "C:/Users/someone/KeyXtend/snips";
/// The most steps one case takes.
const STEPS: usize = 40;

/// Where a switch's answer arrives.
type Heard = Receiver<Result<(), RecordError>>;

/// One thing that happens to a started run.
#[derive(Clone, Debug)]
enum Step {
    /// The job tells a note.
    Note(Note),
    /// The job tells a note while the owner cancels, the cancel landing between the route and the owner's ear.
    Race(Note),
    /// The job is lost.
    Lose,
    /// The owner switches a sound and waits for its answer.
    Wait(Source),
    /// The owner cancels.
    Cancel,
    /// The owner drops the run, which closes its ear before the job's goodbye.
    Drop,
    /// The clock moves on by this many ms.
    Tick(u32),
}

/// Any note the job may tell.
fn note() -> impl Strategy<Value = Note> {
    let switch = select(vec![Switch::Mic, Switch::Sound]);
    let faults = vec![
        RecordFault::NoMic,
        RecordFault::MicRefused,
        RecordFault::NoSound,
        RecordFault::NoVideo,
        RecordFault::Full,
        RecordFault::Failed,
    ];
    let ends = vec![End::Stopped, End::Longest, End::Full];
    prop_oneof![
        any::<u32>().prop_map(|ms| Note::Time { id: ID, ms }),
        (switch.clone(), any::<bool>()).prop_map(|(switch, on)| Note::Switched {
            id: ID,
            switch,
            on
        }),
        select(faults).prop_map(|fault| Note::Fault { id: ID, fault }),
        select(ends).prop_map(|end| Note::End {
            id: ID,
            length_ms: 1,
            end
        }),
        switch.prop_map(|switch| Note::Lost { id: ID, switch }),
    ]
}

/// Any step, notes the likeliest.
fn step() -> impl Strategy<Value = Step> {
    prop_oneof![
        4 => note().prop_map(Step::Note),
        1 => note().prop_map(Step::Race),
        1 => Just(Step::Lose),
        2 => select(vec![Source::Mic, Source::Sound]).prop_map(Step::Wait),
        1 => Just(Step::Cancel),
        1 => Just(Step::Drop),
        2 => (0..2 * SWITCH_MS).prop_map(Step::Tick),
    ]
}

/// Where the run stands, as its job told it.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Run {
    /// Its start waits for the first note.
    Starting,
    /// It records.
    Live,
    /// A start that failed, a fault or a lost job broke it.
    Broke,
    /// Its end kept the video.
    Kept,
}

/// A route over the fake folder and clock, what it handed on, and a model of what its owner may expect.
struct Bed {
    route: Route,
    clock: Hands,
    folder: FakeSnipFolder,
    file: SnipFile,
    started: Heard,
    told: Vec<RecordNote>,
    /// Notes handed on after a cancel.
    late: usize,
    taken: Vec<Heard>,
    refused: Vec<Heard>,
    run: Run,
    cancelled: bool,
    dropped: bool,
    /// The owner heard the run's end.
    heard: bool,
}

impl Bed {
    /// A run recording into a video of the fake folder, its start waiting.
    fn new() -> Self {
        let clock = Hands::default();
        let part = format!("{SNIPS}/{}", crate::UNSAVED_DIR);
        let unsaved = FolderPath::new(native_path(&part)).unwrap();
        let file = native_path(&format!("{part}/1.mp4"));
        let file = SnipFile::new(file, FileKind::Mp4).unwrap();
        let shared: Arc<dyn Clock> = Arc::new(clock.clone());
        let snips = FolderPath::new(native_path(SNIPS)).unwrap();
        let folder = FakeSnipFolder::new(shared, snips, unsaved);
        folder.put(file.clone(), Stored::Video { length_ms: 1 });
        let (answer, started) = mpsc::channel();
        let switch = Duration::from_millis(u64::from(SWITCH_MS));
        let route = Route::new(
            file.clone(),
            Arc::new(folder.clone()),
            Arc::new(clock.clone()),
            answer,
            switch,
        );
        Self {
            route,
            clock,
            folder,
            file,
            started,
            told: Vec::new(),
            late: 0,
            taken: Vec::new(),
            refused: Vec::new(),
            run: Run::Starting,
            cancelled: false,
            dropped: false,
            heard: false,
        }
    }

    /// The owner's ear is open until it cancels or drops the run.
    const fn open(&self) -> bool {
        !self.cancelled && !self.dropped
    }

    /// What the stream told, relayed as `WorkerRun` does: the owner hears a note while its ear is open. A `race`
    /// cancels after the route took it and before the ear heard it.
    fn relay(&mut self, told: Told<Note>, race: bool) {
        self.model(&told);
        let note = self.route.heard(told);
        let race = race && !self.dropped;
        if race {
            self.cancel();
        }
        let Some(note) = note else {
            return;
        };
        self.told.push(note);
        self.late += usize::from(self.cancelled && !race);
        if !self.open() {
            self.route.unheard();
        } else if matches!(note, RecordNote::Done(_)) {
            self.heard = true;
        }
    }

    /// The owner cancels, closing its ear first.
    fn cancel(&mut self) {
        self.cancelled = true;
        self.route.cancel();
    }

    /// Moves the model as `told` says: the first note starts the run or says why not, then a fault or a lost job
    /// breaks it, and its end keeps it.
    fn model(&mut self, told: &Told<Note>) {
        self.run = match (self.run, told) {
            (Run::Starting, Told::Note(Note::Time { .. })) => Run::Live,
            (Run::Starting, _) | (Run::Live, Told::Lost(_) | Told::Idle) => Run::Broke,
            (Run::Live, Told::Note(Note::Fault { fault, .. })) if !sound_fault(*fault) => {
                Run::Broke
            }
            (Run::Live, Told::Note(Note::End { .. })) => Run::Kept,
            (run, _) => run,
        };
    }

    /// Takes `step`; the owner's calls stop once it dropped the run.
    fn take(&mut self, step: Step) {
        match step {
            Step::Note(note) => self.relay(Told::Note(note), false),
            Step::Race(note) => self.relay(Told::Note(note), true),
            Step::Lose => self.relay(Told::Lost(Lost::Gone), false),
            _ if self.dropped => {}
            Step::Wait(source) => {
                let (answer, heard) = mpsc::channel();
                let took = self.route.wait(source, answer);
                (if took {
                    &mut self.taken
                } else {
                    &mut self.refused
                })
                .push(heard);
            }
            Step::Cancel => self.cancel(),
            Step::Drop => self.dropped = true,
            Step::Tick(ms) => self.clock.advance(Duration::from_millis(u64::from(ms))),
        }
    }

    /// True when the video should be gone after the goodbye.
    const fn goes(&self) -> bool {
        match self.run {
            Run::Broke => true,
            Run::Kept => self.cancelled && !self.heard,
            Run::Starting | Run::Live => self.cancelled,
        }
    }

    /// Checks what the owner saw once the route left.
    fn check(self) -> Result<(), TestCaseError> {
        let endings = self.told.iter().filter(|n| n.ends()).count();
        prop_assert!(endings <= 1, "{:?}", self.told);
        prop_assert_eq!(self.late, 0, "told after a cancel: {:?}", self.told);
        prop_assert_eq!(self.started.try_iter().count(), 1);
        for heard in &self.taken {
            prop_assert_eq!(heard.try_iter().count(), 1, "each switch answered once");
        }
        for heard in &self.refused {
            prop_assert_eq!(
                heard.try_iter().count(),
                0,
                "a refused switch is never answered"
            );
        }
        let gone = self.folder.stored(&self.file).is_none();
        prop_assert_eq!(gone, self.goes(), "{:?} {:?}", self.run, self.told);
        Ok(())
    }
}

proptest! {
    #[test]
    fn any_interleaving_keeps_the_routes_promises(first in note(), steps in vec(step(), 0..STEPS)) {
        let mut bed = Bed::new();
        bed.relay(Told::Note(first), false);
        // A start that failed hands no run to its owner, so nothing else happens before the goodbye.
        if bed.run == Run::Live {
            for step in steps {
                bed.take(step);
            }
        }
        bed.dropped = true;
        bed.route.leave();
        bed.check()?;
    }
}
