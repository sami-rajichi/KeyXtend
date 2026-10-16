//! One run's course on its target: the part of a step carried to the next tick, and the fallback to UI Automation's
//! steps when wheel steps move nothing.

use std::time::Duration;

use kx_module_api::Mono;
use kx_module_api::ports::{Aim, Dir, Reach, ScrollConfig, TargetId};

use super::{Route, Wheel, along, reach, route, sign};

/// The most one turn carries: a wheel message's delta is a signed 16-bit word.
pub const MAX_TURN: u16 = i16::MAX.unsigned_abs();
/// One `ScrollPattern` small step scrolls one line.
const STEPS_PER_LINE: f64 = 1.0;

/// One tick's turn: its route and how far.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Turn {
    /// The route.
    pub route: Route,
    /// The wheel delta, unsigned, for `Send` and `Post`; the line steps for `Uia`.
    pub count: u16,
}

impl Turn {
    /// The signed wheel delta towards `dir`.
    #[must_use]
    pub fn delta(self, dir: Dir) -> i32 {
        sign(dir) * i32::from(self.count)
    }
}

/// Lines turned into whole steps, with the part of a step left over kept for the next tick.
#[derive(Copy, Clone, Debug, Default)]
struct Carry {
    /// The steps one line takes; a new unit starts afresh.
    unit: f64,
    /// The part of a step owed, from 0 up to 1.
    owed: f64,
}

impl Carry {
    /// The whole steps `lines` make at `unit` steps a line, at most `MAX_TURN`; past that the rest drops, so nothing
    /// piles up. Lines that are not finite, or below 0, make none.
    fn take(&mut self, lines: f64, unit: f64) -> u16 {
        if self.unit.total_cmp(&unit).is_ne() {
            *self = Self { unit, owed: 0.0 };
        }
        let lines = if lines.is_finite() {
            lines.max(0.0)
        } else {
            0.0
        };
        let exact = self.owed + lines * unit;
        if !exact.is_finite() {
            self.owed = 0.0;
            return MAX_TURN;
        }
        let whole = exact.floor();
        self.owed = exact - whole;
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "The value is whole, at least 0 and at most MAX_TURN."
        )]
        let steps = whole.min(f64::from(MAX_TURN)) as u16;
        steps
    }
}

/// Watches the percent under wheel steps, for the fallback.
#[derive(Copy, Clone, Debug, Default)]
struct Watch {
    /// The percent last read short of the end, and since when it has not moved under wheel steps.
    base: Option<(f64, Mono)>,
    /// Wheel steps went out since the last read.
    stepped: bool,
}

impl Watch {
    /// Takes a percent read at `now`, none unless short of the end; true when it stayed still under wheel steps for
    /// `check`. A read after no wheel steps starts the wait afresh, so a hung window never trips.
    fn still(&mut self, now: Mono, pct: Option<f64>, check: Duration) -> bool {
        let stepped = std::mem::take(&mut self.stepped);
        let Some(pct) = pct else {
            self.base = None;
            return false;
        };
        match self.base {
            Some((was, since)) if stepped && was.total_cmp(&pct).is_eq() => {
                now.saturating_since(since) >= check
            }
            _ => {
                self.base = Some((pct, now));
                false
            }
        }
    }
}

/// One run's state on its target and way; a new target or way starts the carry and the watch afresh.
#[derive(Clone, Debug)]
pub struct Course {
    /// How long a percent may stay still under wheel steps, short of the end, before the target trips.
    check: Duration,
    /// The target and way the carry and the watch belong to.
    on: Option<(TargetId, Dir)>,
    /// The part of a step carried to the next tick.
    carry: Carry,
    /// The fallback's watch.
    watch: Watch,
    /// The target moves by UI Automation's steps until it changes.
    tripped: bool,
}

impl Course {
    /// A course for a run opened with `config`.
    #[must_use]
    pub fn new(config: ScrollConfig) -> Self {
        Self {
            check: Duration::from_millis(u64::from(config.check_ms)),
            on: None,
            carry: Carry::default(),
            watch: Watch::default(),
            tripped: false,
        }
    }

    /// This tick's turn of `lines` towards `dir` at `aim`, by the user's `wheel`; none while less than a whole step is
    /// owed.
    pub fn turn(&mut self, aim: &Aim, dir: Dir, lines: f64, wheel: Wheel) -> Option<Turn> {
        self.follow(aim.id, dir);
        let per_line = wheel.per_line(dir).filter(|_| !self.tripped);
        let route = route(aim.on_pointer, per_line.is_some());
        let count = self.carry.take(lines, per_line.unwrap_or(STEPS_PER_LINE));
        if count == 0 {
            return None;
        }
        self.watch.stepped |= route != Route::Uia;
        Some(Turn { route, count })
    }

    /// Takes UI Automation's percent pair at `aim`, read at `now`, and says how far `dir` can still go. A percent that
    /// stays still under wheel steps for `check_ms`, short of the end, trips this target to UI Automation's steps.
    pub fn read(&mut self, aim: &Aim, dir: Dir, now: Mono, pair: Option<(f64, f64)>) -> Reach {
        self.follow(aim.id, dir);
        let pct = pair.and_then(|p| along(dir, p));
        let reach = reach(dir, pct);
        let short = pct.filter(|_| reach == Reach::More);
        if !self.tripped && self.watch.still(now, short, self.check) {
            self.tripped = true;
        }
        reach
    }

    /// True once this target moves by UI Automation's steps, until the target changes.
    #[must_use]
    pub const fn tripped(&self) -> bool {
        self.tripped
    }

    /// Moves to target `id` and way `dir`: a new one drops the carry and the watch, and a new target its fallback.
    fn follow(&mut self, id: TargetId, dir: Dir) {
        if self.on == Some((id, dir)) {
            return;
        }
        if self.on.is_none_or(|(was, _)| was != id) {
            self.tripped = false;
        }
        (self.on, self.carry, self.watch) = (Some((id, dir)), Carry::default(), Watch::default());
    }
}

#[cfg(test)]
mod tests;
