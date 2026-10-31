//! Property test of the insert's state machine: random Insert clicks, answers right or stale, clock steps, closes,
//! lost entries, refused writes and stalled file checks keep one paste request at a time, one write per request,
//! none outside an insert, never a join last, and always end idle with every step in when the panel closes.

use std::collections::HashSet;
use std::time::Duration;

use kx_module_api::ports::{ClipAct, ClipError, ClipPart, FileFact, Pasted};
use proptest::prelude::*;

use super::super::Msg;
use super::super::paste::Paste;
use super::rig::Rig;
use crate::fixture::{files, text};
use crate::settings::ClipSettings;

/// Random runs tried; each builds a rig.
const CASES: u32 = 256;
/// The longest run of steps.
const MAX_STEPS: usize = 40;
/// The share of answers that say the paste went out.
const OK_SHARE: f64 = 0.8;
/// The file copied first, so it shows last; a relative name reads the same on every platform.
const FILE: &str = "f.txt";
/// The texts copied after it, the last at the top.
const TEXTS: [&str; 3] = ["a", "b", "c"];
/// The rows the page shows: the texts and the file.
const ROWS: u8 = 4;
/// The steps the drain may take to end idle: two per write, twice over.
const DRAIN: usize = 32;

/// One thing that can happen during an insert.
#[derive(Clone, Debug)]
enum Step {
    /// Ticks these rows in this order, then clicks Insert; the panel opens first when shut.
    Insert(Vec<u8>),
    /// Answers the last request, or an older one when `back` is not 0.
    Answer { back: u8, ok: bool },
    /// Moves the clock on by this many ms.
    Advance(u64),
    /// Closes the panel.
    Close,
    /// Loses this entry's content behind the module's back.
    Lose(usize),
    /// The next write is refused as busy, while an insert is under way.
    Refuse,
    /// Stalls the file looks, or answers those stalled.
    Stall(bool),
}

/// Where the insert is, without its data.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Kind {
    Idle,
    Checking,
    Asked,
    Gap,
}

/// Any step; the clock steps hit the gap most, then the paste wait and the file check's deadline, or stay short of
/// the wait, and most answers are right and ok, so many inserts run to their end.
fn step() -> impl Strategy<Value = Step> {
    let s = ClipSettings::from_defaults().unwrap();
    let (gap, wait, check) = (s.paste_gap_ms, s.paste_wait_ms, s.check_ms);
    let ms = prop_oneof![
        4 => Just(gap),
        1 => Just(wait),
        1 => Just(check),
        2 => 0..wait,
    ];
    let back = prop_oneof![3 => Just(0u8), 1 => Just(1u8)];
    let answer = (back, prop::bool::weighted(OK_SHARE));
    let all: Vec<u8> = (0..ROWS).collect();
    let rows = prop::sample::subsequence(all, 1..=usize::from(ROWS)).prop_shuffle();
    prop_oneof![
        3 => rows.prop_map(Step::Insert),
        4 => answer.prop_map(|(back, ok)| Step::Answer { back, ok }),
        4 => ms.prop_map(|n| Step::Advance(u64::from(n))),
        1 => Just(Step::Close),
        1 => (0..usize::from(ROWS)).prop_map(Step::Lose),
        1 => Just(Step::Refuse),
        1 => any::<bool>().prop_map(Step::Stall),
    ]
}

/// The kind of `paste`.
fn kind(paste: &Paste) -> Kind {
    match paste {
        Paste::Idle => Kind::Idle,
        Paste::Checking { .. } => Kind::Checking,
        Paste::Asked { .. } => Kind::Asked,
        Paste::Gap { .. } => Kind::Gap,
    }
}

/// What a write held: its text, or its paths.
fn label(parts: &[ClipPart]) -> String {
    match parts.first() {
        Some(ClipPart::Text(t)) => t.expose().clone(),
        Some(ClipPart::Files(p)) => p.expose().iter().map(|p| p.display().to_string()).collect(),
        _ => String::new(),
    }
}

/// True when `small` appears in `big` in its order.
fn within(small: &[&String], big: &[&String]) -> bool {
    let mut rest = big.iter();
    small.iter().all(|s| rest.any(|b| b == s))
}

/// A rig and what the test knows of its insert.
struct Run {
    rig: Rig,
    /// The text pasted between two texts.
    join: String,
    /// The titles the running insert was asked for, in tick order.
    want: Vec<String>,
    /// The write the running insert started at.
    from: usize,
    /// The titles whose content went.
    lost: HashSet<String>,
}

impl Run {
    /// The file, then the texts, copied, with the panel open.
    fn new() -> Self {
        let mut rig = Rig::new();
        rig.fakes.clip.set_file(FILE, FileFact::Size(1));
        rig.copy(vec![files(&[FILE])], None);
        for t in TEXTS {
            rig.copy(vec![text(t)], None);
        }
        rig.open();
        let join = rig.settings.join_config().text();
        let (want, from, lost) = (Vec::new(), 0, HashSet::new());
        Self {
            rig,
            join,
            want,
            from,
            lost,
        }
    }

    /// Where the insert is now.
    fn kind(&self) -> Kind {
        kind(self.rig.pasting())
    }

    /// True while the panel shows.
    fn open(&self) -> bool {
        self.rig.panels().last() == Some(&true)
    }

    /// Ticks `rows` and clicks Insert, noting what an insert starting now was asked for.
    fn insert(&mut self, rows: &[u8]) {
        if !self.open() {
            self.rig.open();
        }
        for &row in rows {
            self.rig.tick_row(row);
        }
        if self.kind() == Kind::Idle {
            self.want = self.ticked();
            self.from = self.rig.fakes.writer.writes().len();
        }
        self.rig.send(Msg::Act(ClipAct::Insert));
    }

    /// The ticked rows' titles, in tick order.
    fn ticked(&self) -> Vec<String> {
        let mut rows = self.rig.page().rows;
        rows.retain(|r| r.tick.is_some());
        rows.sort_by_key(|r| r.tick);
        rows.iter().map(|r| r.title.expose().clone()).collect()
    }

    /// Does `step`.
    fn apply(&mut self, step: &Step) {
        match step {
            Step::Insert(rows) => self.insert(rows),
            Step::Answer { back, ok } => {
                let last = self.rig.pastes().last().map_or(0, |p| p.round);
                let round = last.wrapping_sub(u32::from(*back));
                self.rig.send(Msg::Pasted(Pasted { round, ok: *ok }));
            }
            Step::Advance(ms) => self.rig.advance(Duration::from_millis(*ms)),
            Step::Close if self.open() => self.rig.send(Msg::Act(ClipAct::Close)),
            Step::Lose(n) => self.lose(*n),
            Step::Refuse if self.kind() != Kind::Idle => {
                self.rig.fakes.writer.queue(Err(ClipError::Busy));
            }
            Step::Close | Step::Refuse => {}
            Step::Stall(on) => self.rig.hold_looks(*on),
        }
    }

    /// Loses entry `n`'s content once; only a lost entry ever leaves, so it is still kept the first time.
    fn lose(&mut self, n: usize) {
        let title = [FILE].into_iter().chain(TEXTS).nth(n).unwrap_or(FILE);
        if self.lost.insert(title.to_owned()) {
            self.rig.lose(title);
        }
    }

    /// Does `step`, then checks it kept the rules.
    fn step(&mut self, step: &Step) -> Result<(), TestCaseError> {
        let (was, asks) = (self.kind(), self.rig.pastes().len());
        self.apply(step);
        self.check(step, was, asks)
    }

    /// One write per request, one request at a time and none outside an insert; a close stops it.
    fn check(&self, step: &Step, was: Kind, asks: usize) -> Result<(), TestCaseError> {
        let all = self.rig.pastes();
        let writes = self.rig.fakes.writer.writes().len();
        prop_assert_eq!(writes, all.len(), "one write per request");
        let new = all.len() - asks;
        prop_assert!(new <= 1, "one request per step");
        if new == 1 {
            prop_assert!(was != Kind::Asked, "a request waits for the one before");
            let last = all.last().map(|p| p.round);
            let asked =
                matches!(self.rig.pasting(), Paste::Asked { round, .. } if Some(*round) == last);
            prop_assert!(asked, "the newest request is the one waited for");
        }
        if was == Kind::Idle && !matches!(step, Step::Insert(_)) {
            prop_assert_eq!(new, 0, "no paste outside an insert");
        }
        if matches!(step, Step::Close) {
            prop_assert_eq!(self.kind(), Kind::Idle, "a close stops the insert");
        }
        if was != Kind::Idle && self.kind() == Kind::Idle {
            self.ended(step)?;
        }
        Ok(())
    }

    /// An insert that ended alone with no note closed the panel, with every entry kept in tick order, no join last.
    fn ended(&self, step: &Step) -> Result<(), TestCaseError> {
        if matches!(step, Step::Close) || self.rig.note().is_some() {
            return Ok(());
        }
        let closed = self.rig.panels().last() == Some(&false);
        prop_assert!(closed, "the panel closes when all went in");
        let writes = self.rig.fakes.writer.writes();
        let went: Vec<String> = writes[self.from..].iter().map(|w| label(w)).collect();
        prop_assert!(went.last() != Some(&self.join), "never a join last");
        let entries: Vec<&String> = went.iter().filter(|w| **w != self.join).collect();
        let ticked: Vec<&String> = self.want.iter().collect();
        let mut kept = ticked.clone();
        kept.retain(|t| !self.lost.contains(*t));
        prop_assert!(
            within(&entries, &ticked),
            "ticked only, in order: {:?}",
            went
        );
        prop_assert!(
            within(&kept, &entries),
            "every kept one went in: {:?}",
            went
        );
        Ok(())
    }

    /// Answers and waits until the insert ends, as a keyboard that always pastes.
    fn drain(&mut self) -> Result<(), TestCaseError> {
        let gap = u64::from(self.rig.settings.paste_gap_ms);
        for _ in 0..DRAIN {
            let step = match self.kind() {
                Kind::Idle => return Ok(()),
                Kind::Checking => Step::Stall(false),
                Kind::Asked => Step::Answer { back: 0, ok: true },
                Kind::Gap => Step::Advance(gap),
            };
            self.step(&step)?;
        }
        prop_assert_eq!(self.kind(), Kind::Idle, "the insert always ends");
        Ok(())
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn random_inserts_paste_one_step_at_a_time_and_always_end_idle(
        steps in prop::collection::vec(step(), 0..MAX_STEPS),
    ) {
        let mut run = Run::new();
        for step in &steps {
            run.step(step)?;
        }
        run.drain()?;
    }
}
