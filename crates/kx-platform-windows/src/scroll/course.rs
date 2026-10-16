//! One run's course on its target: the part of a step carried to the next tick, the fallback to UI Automation's steps
//! when a notch of wheel steps moves nothing, and the end those steps cannot pass.

use std::time::Duration;

use kx_module_api::Mono;
use kx_module_api::ports::{Aim, Dir, Reach, ScrollConfig, TargetId};

use super::{Route, Wheel, along, reach, route, sign};

/// The most one turn carries: a wheel message's delta is a signed 16-bit word.
pub const MAX_TURN: u16 = i16::MAX.unsigned_abs();
/// One `ScrollPattern` small step scrolls one line.
const STEPS_PER_LINE: f64 = 1.0;
/// Reads may come this many `check_ms` apart, plus one `read_ms`, before the gap is a pause; a tick never outlasts a
/// check, so a late tick and a slow read still fit.
const PAUSE_CHECKS: u32 = 2;

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

/// What the latest turn went by, which sets how much must go out before a still percent counts.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Mode {
    /// Wheel delta, with a notch of this size.
    Wheel(u32),
    /// UI Automation's line steps.
    Steps,
}

impl Mode {
    /// The steps that must go out: a whole notch of delta, or one line step.
    fn need(self) -> u64 {
        match self {
            Self::Wheel(notch) => u64::from(notch).max(1),
            Self::Steps => 1,
        }
    }
}

/// How long a percent must stay still, and the gap between reads that starts the wait afresh.
#[derive(Copy, Clone, Debug)]
struct Times {
    /// `check_ms`.
    check: Duration,
    /// `PAUSE_CHECKS` times `check_ms`, plus `read_ms`: a later read follows a pause.
    stale: Duration,
}

/// Watches the percent under steps: the base it has stayed at, and what went out since.
#[derive(Copy, Clone, Debug, Default)]
struct Watch {
    /// The percent last read short of the end, and since when it has stayed there.
    base: Option<(f64, Mono)>,
    /// The steps sent since then, in the latest mode's unit.
    sent: u64,
    /// The steps sent since then up to the previous read: only these had a whole read to show.
    shown: u64,
    /// The previous read.
    last: Option<Mono>,
}

impl Watch {
    /// Takes a percent read at `now`, none unless short of the end; true when it stayed still for `check` while
    /// `need` steps went out before the previous read. A moved percent, none, or a read after a pause starts afresh.
    fn still(&mut self, now: Mono, pct: Option<f64>, need: u64, times: Times) -> bool {
        let paused = self
            .last
            .is_none_or(|t| now.saturating_since(t) > times.stale);
        self.last = Some(now);
        let judged = std::mem::replace(&mut self.shown, self.sent);
        match (self.base, pct) {
            (Some((was, since)), Some(pct)) if !paused && was.total_cmp(&pct).is_eq() => {
                now.saturating_since(since) >= times.check && judged >= need
            }
            _ => {
                (self.base, self.sent, self.shown) = (pct.map(|p| (p, now)), 0, 0);
                false
            }
        }
    }
}

/// One run's state on its target and way; a new target or way starts the carry and the watch afresh.
#[derive(Clone, Debug)]
pub struct Course {
    /// The watch's times.
    times: Times,
    /// The target and way the carry and the watch belong to.
    on: Option<(TargetId, Dir)>,
    /// The part of a step carried to the next tick.
    carry: Carry,
    /// The fallback's and the end's watch.
    watch: Watch,
    /// What the latest turn went by; none before the first.
    mode: Option<Mode>,
    /// The target moves by UI Automation's steps until it changes.
    tripped: bool,
}

impl Course {
    /// A course for a run opened with `config`.
    #[must_use]
    pub fn new(config: ScrollConfig) -> Self {
        let ms = |n: u32| Duration::from_millis(u64::from(n));
        let times = Times {
            check: ms(config.check_ms),
            stale: ms(config
                .check_ms
                .saturating_mul(PAUSE_CHECKS)
                .saturating_add(config.read_ms)),
        };
        Self {
            times,
            on: None,
            carry: Carry::default(),
            watch: Watch::default(),
            mode: None,
            tripped: false,
        }
    }

    /// This tick's turn of `lines` towards `dir` at `aim`, by the user's `wheel`; none while less than a whole step is
    /// owed. Call it only for a step that will really go out, so a hung window's skipped tick never counts.
    pub fn turn(&mut self, aim: &Aim, dir: Dir, lines: f64, wheel: Wheel) -> Option<Turn> {
        self.follow(aim.id, dir);
        let per_line = wheel.per_line(dir).filter(|_| !self.tripped);
        let mode = per_line.map_or(Mode::Steps, |_| Mode::Wheel(wheel.notch));
        if self.mode.replace(mode).is_some_and(|was| was != mode) {
            self.watch = Watch::default();
        }
        let route = route(aim.on_pointer, per_line.is_some());
        let count = self.carry.take(lines, per_line.unwrap_or(STEPS_PER_LINE));
        if count == 0 {
            return None;
        }
        self.watch.sent = self.watch.sent.saturating_add(u64::from(count));
        Some(Turn { route, count })
    }

    /// Takes UI Automation's percent pair at `aim`, read at `now`, and says how far `dir` can still go. A percent still
    /// for `check_ms` short of the end, after a notch of wheel delta or a UI Automation step sent before the previous
    /// read, trips the target or reads as the end.
    pub fn read(&mut self, aim: &Aim, dir: Dir, now: Mono, pair: Option<(f64, f64)>) -> Reach {
        self.follow(aim.id, dir);
        let pct = pair.and_then(|p| along(dir, p));
        let reach = reach(dir, pct);
        let short = pct.filter(|_| reach == Reach::More);
        let need = self.mode.map_or(u64::MAX, Mode::need);
        if !self.watch.still(now, short, need, self.times) {
            return reach;
        }
        if self.mode == Some(Mode::Steps) {
            return Reach::End;
        }
        self.tripped = true;
        reach
    }

    /// True once this target moves by UI Automation's steps, until the target changes.
    #[must_use]
    pub const fn tripped(&self) -> bool {
        self.tripped
    }

    /// Moves to target `id` and way `dir`: a new one starts the carry, the watch and the mode afresh, and a new
    /// target its fallback.
    fn follow(&mut self, id: TargetId, dir: Dir) {
        if self.on == Some((id, dir)) {
            return;
        }
        if self.on.is_none_or(|(was, _)| was != id) {
            self.tripped = false;
        }
        self.on = Some((id, dir));
        (self.carry, self.watch, self.mode) = (Carry::default(), Watch::default(), None);
    }
}

#[cfg(test)]
mod props;
#[cfg(test)]
mod tests;
