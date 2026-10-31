//! Property test: random copies, pins, unpins, inserts, deletes, disk changes and day changes, forward or back, never
//! break the caps, the order or the places, and a sweep leaves nothing past its days.

use kx_module_api::ports::{ClipKind, LocalTime};
use proptest::prelude::*;

use super::*;
use crate::entry::Place;
use crate::fixture::{CFG, copied, date, files};

/// Random runs tried.
const CASES: u32 = 256;
/// The longest run of steps, all on one day's minutes.
const MAX_STEPS: usize = 64;
/// How many contents the copies hold, so some repeat.
const CONTENTS: u8 = 12;
/// Every this many contents, one is a video.
const VIDEO_EVERY: u8 = 4;
/// What each content keeps: this many bytes per content number, plus `BASE_BYTES`; the last ones are too big.
const STEP_BYTES: u64 = 25;
/// The bytes the smallest content keeps.
const BASE_BYTES: u64 = 10;
/// Small limits, so the caps and the sweeps act often.
const SMALL: Limits = Limits {
    keep_days: 3,
    media_days: 1,
    max_entries: 6,
    max_bytes: 600,
    min_free_bytes: 100,
    max_entry_bytes: 250,
};
/// Free space well over the floor.
const ROOM: u64 = 10_000;
/// Free space under the floor.
const TIGHT: u64 = 50;
/// Minutes in a day.
const DAY_MIN: i64 = 24 * 60;
/// Milliseconds in a minute.
const MIN_MS: i64 = 60_000;

/// One thing that can happen to the history.
#[derive(Debug, Clone)]
enum Step {
    /// A copy of content `n`.
    Copy(u8),
    /// Pin the entry at this place, wrapped.
    Pin(usize),
    /// Unpin it.
    Unpin(usize),
    /// Insert it, which moves it up.
    Insert(usize),
    /// Delete it.
    Delete(usize),
    /// The drive has room, or not.
    Disk(bool),
    /// The calendar moves this many days, back when below 0.
    Day(i8),
}

/// Any step.
fn step() -> impl Strategy<Value = Step> {
    prop_oneof![
        5 => (0..CONTENTS).prop_map(Step::Copy),
        1 => any::<usize>().prop_map(Step::Pin),
        1 => any::<usize>().prop_map(Step::Unpin),
        1 => any::<usize>().prop_map(Step::Insert),
        1 => any::<usize>().prop_map(Step::Delete),
        1 => any::<bool>().prop_map(Step::Disk),
        2 => (-10i8..=3).prop_map(Step::Day),
    ]
}

/// A run: the history, today's day number, the minute and the free bytes.
struct Run {
    history: History,
    day: i64,
    minute: i64,
    free: u64,
}

impl Run {
    /// An empty history on 2026-10-31 with room.
    fn new() -> Self {
        let day = date(2026, 10, 31).day_number();
        let history = History::default();
        Self {
            history,
            day,
            minute: 0,
            free: ROOM,
        }
    }

    /// The wall time now, in UTC.
    fn now(&self) -> LocalTime {
        LocalTime::at((self.day * DAY_MIN + self.minute) * MIN_MS, 0).unwrap()
    }

    /// The id at place `n`, wrapped, if any entry is kept.
    fn pick(&self, n: usize) -> Option<u64> {
        let all = self.history.entries();
        all.get(n.checked_rem(all.len())?).map(|e| e.id)
    }

    /// A copy of content `n`, kept, moved up as a twin, or refused.
    fn copy(&mut self, n: u8) {
        let now = self.now();
        let mut e = if n.is_multiple_of(VIDEO_EVERY) {
            Entry::new(&[files(&[&format!("clip{n}.mp4")])], None, &now, &CFG)
        } else {
            copied(&format!("item {n}"), &now)
        };
        e.stored = u64::from(n) * STEP_BYTES + BASE_BYTES;
        if let Some(id) = self.history.twin(e.hash, |old| old.title == e.title) {
            self.history.raise(id, &now);
        } else if admit(&self.history, &e, self.free, &SMALL).is_ok() {
            self.history.add(e);
            trim(&mut self.history, &SMALL);
        }
    }

    /// Does `act` to the entry at place `n`, wrapped, if any entry is kept.
    fn on(&mut self, n: usize, act: impl FnOnce(&mut History, u64)) {
        if let Some(id) = self.pick(n) {
            act(&mut self.history, id);
        }
    }

    /// Applies `step`.
    fn apply(&mut self, step: &Step) {
        let now = self.now();
        match *step {
            Step::Copy(n) => self.copy(n),
            Step::Pin(n) => self.on(n, |h, id| {
                h.pin(id);
            }),
            Step::Unpin(n) => self.on(n, |h, id| {
                h.unpin(id, &now);
            }),
            Step::Insert(n) => self.on(n, |h, id| {
                h.raise(id, &now);
            }),
            Step::Delete(n) => self.on(n, |h, id| {
                h.remove(id);
            }),
            Step::Disk(room) => self.free = if room { ROOM } else { TIGHT },
            Step::Day(by) => {
                self.day += i64::from(by);
                let today = self.now().date;
                sweep(&mut self.history, today, &SMALL);
            }
        }
    }
}

/// The caps hold, ids are unique, pins come first, each group newest first, and each day is its copy's day.
fn check(history: &History) -> Result<(), TestCaseError> {
    let all = history.entries();
    prop_assert!(all.len() <= SMALL.max_entries);
    prop_assert!(history.bytes() <= SMALL.max_bytes);
    for pair in all.windows(2) {
        let (a, b) = (&pair[0], &pair[1]);
        prop_assert!(a.pinned() || !b.pinned(), "pinned first");
        prop_assert!(a.pinned() != b.pinned() || a.seq > b.seq, "newest first");
    }
    let mut ids: Vec<u64> = all.iter().map(|e| e.id).collect();
    ids.sort_unstable();
    ids.dedup();
    prop_assert_eq!(ids.len(), all.len());
    for e in all {
        if let Place::Day(day) = e.place {
            prop_assert_eq!(Some(day), e.at.local().map(|t| t.date));
        }
    }
    Ok(())
}

/// No unpinned entry is past `keep_days`, and no media past `media_days`.
fn swept(run: &Run) -> Result<(), TestCaseError> {
    let today = run.now().date;
    for e in run.history.entries() {
        if let Place::Day(day) = e.place {
            prop_assert!(!past(day, today, SMALL.keep_days));
            prop_assert!(e.kind != ClipKind::Media || !past(day, today, SMALL.media_days));
        }
    }
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn random_steps_keep_the_caps_the_order_and_the_days(
        steps in prop::collection::vec(step(), 0..MAX_STEPS),
    ) {
        let mut run = Run::new();
        for (minute, step) in (0..).zip(&steps) {
            run.minute = minute;
            run.apply(step);
            check(&run.history)?;
            if matches!(step, Step::Day(_)) {
                swept(&run)?;
            }
        }
    }
}
