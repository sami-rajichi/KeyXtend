//! The speech host's job life under any run of runs opening, ids given and taken back, keeps, kills, closes, other
//! starts and model changes, notes, a lost worker and `idle_min`: a closed run hears nothing, a note reaches only the
//! run its id names, a wipe or an idle job tells each held recording `Wiped` once, a lost worker kills every run and
//! answers its keep, a keep hears nothing after its end, and no run is held and dead at once. A job's key, which says
//! when a run needs another job, changes with the model file, and its cap covers the model.

use super::*;
use crate::speech::router::{self, Kept, Router};
use crate::worker::fake::voice::{card_failed, done, failed, level, open, piece, shut};
use crate::worker::{Lost, Told};
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::sample::select;
use std::collections::BTreeMap;

/// The most steps in one case.
const MAX_STEPS: usize = 60;
/// A place among the runs or the ids given, wrapped.
const MAX_AT: u8 = 8;
/// How often a note names an id that was given.
const GIVEN: f64 = 0.9;
/// The memory a test job may get, in MB, and the longest recording a test ask keeps, in seconds.
const MAX_JOB_MB: u32 = 4_096;
const MAX_RECORD_S: u16 = TEN_MIN_S;

/// A note the job tells on an id.
type Note = fn(u32) -> VoiceNote;

/// The job cannot take that ask now.
fn out_of_step(id: u32) -> VoiceNote {
    let fault = VoiceFault::OutOfStep;
    VoiceNote::Fault { id, fault }
}

/// One thing the router meets; each place wraps over the runs or the ids given.
#[derive(Clone, Debug)]
enum Op {
    Add,
    /// A fresh id for the run at this place, as an ask begins.
    Map(u8),
    /// The newest id taken back, as its ask was not sent.
    Unmap,
    Kill(u8),
    Keep(u8),
    Close(u8),
    /// A run starts or changes its model: every recording goes but this run's, or all.
    Wipe(Option<u8>),
    /// The job tells this note on the id at this place, or on an id never given.
    Note(Note, Option<u8>),
    /// The worker died, late when true.
    Lost(bool),
    Idle,
}

fn op() -> impl Strategy<Value = Op> {
    let at = || 0..MAX_AT;
    let notes: Vec<Note> = vec![
        open,
        shut,
        level,
        piece,
        done,
        card_failed,
        failed,
        no_sound,
        out_of_step,
        first_sound,
        last_sound,
    ];
    prop_oneof![
        2 => Just(Op::Add),
        3 => at().prop_map(Op::Map),
        1 => Just(Op::Unmap),
        1 => at().prop_map(Op::Kill),
        2 => at().prop_map(Op::Keep),
        1 => at().prop_map(Op::Close),
        1 => prop::option::of(at()).prop_map(Op::Wipe),
        8 => (select(notes), prop::option::weighted(GIVEN, at())).prop_map(|(n, a)| Op::Note(n, a)),
        1 => any::<bool>().prop_map(Op::Lost),
        1 => Just(Op::Idle),
    ]
}

/// What a run must hear from one step.
#[derive(Clone, Debug, PartialEq)]
enum Hear {
    Nothing,
    Exactly(Vec<SpeechNote>),
    /// The note's own run hears it, or not, as the note says.
    AtMostOne,
}

/// A run as the case sees it.
struct Run {
    num: u64,
    heard: Receiver<SpeechNote>,
    keep: Option<Receiver<Kept>>,
    /// Its keep heard its last piece or a fault.
    answered: bool,
    open: bool,
    held: bool,
}

/// The router and what the case expects of it.
#[derive(Default)]
struct Case {
    router: Router,
    runs: Vec<Run>,
    /// Each id given that still names its run.
    owner: BTreeMap<u32, usize>,
    /// The last id given.
    last: u32,
}

impl Case {
    /// Calls `f` on the run at `at`, if any.
    fn on(&mut self, at: u8, f: fn(&mut Self, usize)) {
        if !self.runs.is_empty() {
            f(self, usize::from(at) % self.runs.len());
        }
    }

    fn add(&mut self) {
        let (tell, heard) = ear();
        let num = self.router.add(tell);
        let (keep, answered, open, held) = (None, false, true, false);
        let run = Run {
            num,
            heard,
            keep,
            answered,
            open,
            held,
        };
        self.runs.push(run);
    }

    fn map(&mut self, i: usize) {
        self.last += 1;
        self.router.map(self.runs[i].num, self.last);
        if self.runs[i].open {
            self.owner.insert(self.last, i);
            self.runs[i].held = true;
        }
    }

    fn unmap(&mut self) {
        self.router.unmap(self.last);
        if let Some(i) = self.owner.remove(&self.last) {
            self.runs[i].held = false;
        }
    }

    fn kill(&mut self, i: usize) {
        self.router.kill(self.runs[i].num);
        self.runs[i].held = false;
    }

    fn keep(&mut self, i: usize) {
        let (to, kept) = mpsc::channel();
        self.router.keep(self.runs[i].num, to);
        (self.runs[i].keep, self.runs[i].answered) = (Some(kept), false);
    }

    fn close(&mut self, i: usize) {
        if let Some(hearer) = self.router.close(self.runs[i].num) {
            router::hush(&hearer);
        }
        (self.runs[i].open, self.runs[i].held) = (false, false);
        self.owner.retain(|_, r| *r != i);
    }

    /// Every open run but `except` that held tells `Wiped` once, and only `except` keeps its ids.
    fn wipe(&mut self, except: Option<usize>, hear: &mut [Hear]) {
        let num = except.map(|e| self.runs[e].num);
        router::tell(self.router.wipe(num));
        for (i, run) in self.runs.iter_mut().enumerate() {
            if Some(i) != except && std::mem::take(&mut run.held) {
                hear[i] = Hear::Exactly(vec![SpeechNote::Wiped]);
            }
        }
        self.owner.retain(|_, r| Some(*r) == except);
    }

    /// The job tells `note` on the id at `at`: only the run that id names may hear it.
    fn note(&mut self, note: Note, at: Option<u8>, hear: &mut [Hear]) {
        let given = at.filter(|_| self.last > 0);
        let id = given.map_or(u32::MAX, |a| 1 + u32::from(a) % self.last);
        let lost = note(id) == no_sound(id);
        router::tell(self.router.hear(Told::Note(note(id))));
        if let Some(&i) = self.owner.get(&id) {
            hear[i] = Hear::AtMostOne;
            self.runs[i].held &= !lost;
        }
    }

    /// The worker died: every open run hears its fault and holds nothing, and no id names a run.
    fn lose(&mut self, late: bool, hear: &mut [Hear]) -> SpeechError {
        let (lost, error) = if late {
            (Lost::Late, SpeechError::Timeout)
        } else {
            (Lost::Gone, SpeechError::Gone)
        };
        router::tell(self.router.hear(Told::Lost(lost)));
        for (i, run) in self.runs.iter_mut().enumerate().filter(|(_, r)| r.open) {
            (hear[i], run.held) = (Hear::Exactly(vec![SpeechNote::Fault(error)]), false);
        }
        self.owner.clear();
        error
    }

    /// Applies `op`; what each run must hear, and the error a lost worker answers waiting keeps with.
    fn apply(&mut self, op: &Op) -> (Vec<Hear>, Option<SpeechError>) {
        let mut hear = vec![Hear::Nothing; self.runs.len() + 1];
        let mut lost = None;
        match *op {
            Op::Add => self.add(),
            Op::Map(at) => self.on(at, Self::map),
            Op::Unmap => self.unmap(),
            Op::Kill(at) => self.on(at, Self::kill),
            Op::Keep(at) => self.on(at, Self::keep),
            Op::Close(at) => self.on(at, Self::close),
            Op::Wipe(at) => {
                let except = at.filter(|_| !self.runs.is_empty());
                self.wipe(except.map(|a| usize::from(a) % self.runs.len()), &mut hear);
            }
            Op::Note(note, at) => self.note(note, at, &mut hear),
            Op::Lost(late) => lost = Some(self.lose(late, &mut hear)),
            Op::Idle => {
                router::tell(self.router.hear(Told::Idle));
                self.wipe(None, &mut hear);
            }
        }
        (hear, lost)
    }

    /// Checks what each run heard and where it stands, and its keep: nothing after its end, and a lost worker's fault.
    fn check(&mut self, hear: &[Hear], lost: Option<SpeechError>) -> Result<(), TestCaseError> {
        for (i, run) in self.runs.iter_mut().enumerate() {
            let got: Vec<SpeechNote> = run.heard.try_iter().collect();
            match &hear[i] {
                Hear::Nothing => prop_assert!(got.is_empty(), "run {} heard {:?}", i, got),
                Hear::Exactly(want) => prop_assert_eq!(&got, want, "run {}", i),
                Hear::AtMostOne => prop_assert!(got.len() <= 1, "run {} heard {:?}", i, got),
            }
            let state = self.router.state(run.num);
            prop_assert_eq!(state.is_some(), run.open, "run {} open", i);
            if let Some(s) = state {
                prop_assert_eq!(s.held, run.held, "run {} held", i);
                prop_assert!(!(s.held && s.dead), "run {} held and dead", i);
            }
            let Some(keep) = &run.keep else { continue };
            let waiting = run.open && !run.answered;
            let kept: Vec<Kept> = keep.try_iter().collect();
            for k in &kept {
                prop_assert!(!run.answered, "run {}'s keep heard after its end", i);
                run.answered = matches!(k, Ok((_, true)) | Err(_));
            }
            if let Some(error) = lost.filter(|_| waiting) {
                let answered = matches!(kept.last(), Some(Err(e)) if *e == error);
                prop_assert!(answered, "run {}'s keep never heard the lost worker", i);
            }
        }
        Ok(())
    }
}

proptest! {
    #[test]
    fn a_run_hears_only_its_own_notes_and_nothing_after_its_close(ops in vec(op(), 1..MAX_STEPS)) {
        let mut case = Case::default();
        for op in &ops {
            let (hear, lost) = case.apply(op);
            case.check(&hear, lost)?;
        }
    }

    #[test]
    fn a_job_is_shared_only_by_the_same_model_and_its_cap_covers_it(
        files in (0..3_usize, 0..3_usize),
        job_mb in 0..=MAX_JOB_MB,
        record_max_s in 1..=MAX_RECORD_S,
        extra_mb in 0..=MAX_JOB_MB,
    ) {
        let file = |n: usize| [None, Some(FILE_A), Some(FILE_B)][n];
        let ask_of = |n: usize| {
            let mut a = ask(file(n));
            (a.job.job_mb, a.record_max_s) = (job_mb, record_max_s);
            a
        };
        let (a, b) = (ask_of(files.0), ask_of(files.1));
        prop_assert_eq!(Key::of(&a) == Key::of(&b), files.0 == files.1);
        let cap = Key::of(&a).launch(0).memory_mb;
        let need = a.model.as_ref().map_or(0, |m| m.cpu_mb);
        prop_assert!(cap >= job_mb + need, "the cap {} misses the model", cap);
        prop_assert_eq!(Key::of(&a).launch(extra_mb).memory_mb, cap + extra_mb);
    }
}
