//! One scroll run on a desk of windows (ADR-0029): it finds the target, turns it by the course's route and reads how far
//! it can go, with one UI Automation read a check. Pure over the `Desk` trait; `win` gives Windows' desk.

use std::fmt;
use std::time::Duration;

use kx_module_api::ports::{
    Aim, Dir, Point, Reach, Rect, Screen, ScrollConfig, ScrollError, ScrollRun, TargetId,
};
use kx_module_api::{Mono, Redacted};

use super::entry::Kept;
use super::{Course, Route, Wheel, area};

/// Whose window a point lands on.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Whose {
    /// The keyboard, whose entries the guard watches.
    Keyboard,
    /// Another of our windows, such as an open list.
    Ours,
    /// Another app's.
    Theirs,
}

/// The window at a point: its raw id and whose it is.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Spot {
    /// The window's raw id.
    pub window: u64,
    /// Whose it is.
    pub whose: Whose,
}

/// What UI Automation read at a point: the box of the first element there that scrolls (left, top, right, bottom), and
/// its percent pair (horizontal, vertical). Its `Debug` hides the box.
#[derive(Copy, Clone, PartialEq)]
pub struct Read {
    /// The box; none when nothing there scrolls.
    pub area: Option<[i32; 4]>,
    /// The percents; none when nothing there scrolls.
    pub pair: Option<(f64, f64)>,
}

/// Shows the percents, and only whether there is a box.
impl fmt::Debug for Read {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Read")
            .field("area", &self.area.map(Redacted::new))
            .field("pair", &self.pair)
            .finish()
    }
}

/// The system a run scrolls on: Windows in `win`, a fake in tests.
pub trait Desk {
    /// Where the pointer is; none when the system cannot say.
    fn cursor(&self) -> Option<Point>;
    /// The window at `p`; none when there is none.
    fn spot(&self, p: Point) -> Option<Spot>;
    /// What the guard kept when the pointer last entered the keyboard.
    fn kept(&self) -> Option<Kept>;
    /// Every screen's work area.
    fn screens(&self) -> Vec<Screen>;
    /// The box around every screen.
    fn whole(&self) -> Rect;
    /// True while `window` exists.
    fn alive(&self, window: u64) -> bool;
    /// True while the system reports `window`'s app as hung.
    fn hung(&self, window: u64) -> bool;
    /// The user's wheel settings.
    fn wheel(&self) -> Wheel;
    /// Posts a wheel turn of `delta` on `dir`'s wheel to `window`, at `at`.
    ///
    /// # Errors
    /// `Gone` when the window closed, `Refused` when the system refuses.
    fn post(&self, window: u64, at: Point, dir: Dir, delta: i32) -> Result<(), ScrollError>;
    /// Sends a wheel turn of `delta` on `dir`'s wheel at the pointer, tagged as ours.
    ///
    /// # Errors
    /// `Refused` when the system refuses.
    fn send(&self, dir: Dir, delta: i32) -> Result<(), ScrollError>;
    /// UI Automation's read at `at`, waited on for `cap` at most; none when it took longer or the reader is stuck.
    fn read(&self, at: Point, cap: Duration) -> Option<Read>;
    /// True while UI Automation is free to take steps now.
    fn idle(&self) -> bool;
    /// Hands UI Automation `count` line steps towards `dir` at `at`, never waited on; false when they were not taken.
    fn steps(&self, at: Point, dir: Dir, count: u16) -> bool;
    /// The time now, on the crate's clock.
    fn now(&self) -> Mono;
}

/// This check's read of one target: its percents and when they were read.
#[derive(Copy, Clone, Debug)]
struct Fresh {
    id: TargetId,
    pair: Option<(f64, f64)>,
    at: Mono,
}

/// A running scroll on `D`.
pub struct Run<D> {
    desk: D,
    config: ScrollConfig,
    course: Course,
    wheel: Wheel,
    /// The target the wheel settings were last read for.
    wheel_for: Option<TargetId>,
    /// This check's read; an aim or a step that goes out clears it.
    fresh: Option<Fresh>,
    /// The latest box of a target, kept while a read of it misses.
    last: Option<(TargetId, Option<Rect>)>,
}

impl<D: Desk> Run<D> {
    /// A run on `desk` as `config` says; it reads the wheel settings now.
    pub fn new(desk: D, config: ScrollConfig) -> Self {
        let wheel = desk.wheel();
        Self {
            desk,
            config,
            course: Course::new(config),
            wheel,
            wheel_for: None,
            fresh: None,
            last: None,
        }
    }

    /// The point to scroll, whether the pointer is on it, and the window there: under the pointer, or the point the
    /// guard kept while the pointer is on the keyboard.
    fn locate(&self) -> Result<(Point, bool, Spot), ScrollError> {
        let cursor = self.desk.cursor().ok_or(ScrollError::Refused)?;
        let under = self.desk.spot(cursor).ok_or(ScrollError::Gone)?;
        if under.whose != Whose::Keyboard {
            return Ok((cursor, true, under));
        }
        let kept = self.desk.kept().ok_or(ScrollError::NoTarget)?;
        let at = kept.target(&self.desk.screens(), self.config.edge_px);
        let at = at.ok_or(ScrollError::NoTarget)?;
        Ok((at, false, self.desk.spot(at).ok_or(ScrollError::Gone)?))
    }

    /// The box of target `id` at `at`, from this check's read; a read that misses keeps the target's last box.
    fn look(&mut self, id: TargetId, at: Point) -> Option<Rect> {
        let area = match self.read(id, at) {
            Some(read) => read.area.and_then(|edges| area(edges, self.desk.whole())),
            None => self.last.filter(|(was, _)| *was == id).and_then(|(_, a)| a),
        };
        self.last = Some((id, area));
        area
    }

    /// Reads target `id` at `at` within `read_ms`, unless its app hangs, and keeps it as this check's read with one
    /// clock reading.
    fn read(&mut self, id: TargetId, at: Point) -> Option<Read> {
        let cap = Duration::from_millis(u64::from(self.config.read_ms));
        let read = if self.desk.hung(id.get()) {
            None
        } else {
            self.desk.read(at, cap)
        };
        let pair = read.and_then(|r| r.pair);
        self.fresh = Some(Fresh {
            id,
            pair,
            at: self.desk.now(),
        });
        read
    }

    /// True when the next step towards `dir` at `aim` goes by UI Automation, which is busy.
    fn reader_busy(&self, aim: &Aim, dir: Dir) -> bool {
        self.course.next_route(aim, dir, self.wheel) == Route::Uia && !self.desk.idle()
    }

    /// True while the pointer is still on `window`, so a sent wheel reaches it and nothing else.
    fn under_pointer(&self, window: u64) -> bool {
        let spot = self.desk.cursor().and_then(|p| self.desk.spot(p));
        spot.is_some_and(|s| s.window == window)
    }
}

impl<D: Desk + Send> ScrollRun for Run<D> {
    fn aim(&mut self) -> Result<Aim, ScrollError> {
        self.fresh = None;
        let (at, on_pointer, spot) = self.locate()?;
        let id = TargetId::new(spot.window);
        let ours = spot.whose != Whose::Theirs;
        if !ours && self.wheel_for.replace(id) != Some(id) {
            self.wheel = self.desk.wheel();
        }
        let area = if ours { None } else { self.look(id, at) };
        Ok(Aim {
            at,
            on_pointer,
            area,
            ours,
            id,
        })
    }

    fn turn(&mut self, aim: &Aim, dir: Dir, lines: f64) -> Result<(), ScrollError> {
        let window = aim.id.get();
        if aim.ours {
            return Ok(());
        }
        if !self.desk.alive(window) {
            return Err(ScrollError::Gone);
        }
        if self.desk.hung(window) || self.reader_busy(aim, dir) {
            return Ok(());
        }
        let Some(turn) = self.course.turn(aim, dir, lines, self.wheel) else {
            return Ok(());
        };
        self.fresh = None;
        let delta = turn.delta(dir);
        match turn.route {
            Route::Uia => {
                self.desk.steps(aim.at, dir, turn.count);
                Ok(())
            }
            Route::Send if self.under_pointer(window) => self.desk.send(dir, delta),
            Route::Send | Route::Post => self.desk.post(window, aim.at, dir, delta),
        }
    }

    fn reach(&mut self, aim: &Aim, dir: Dir) -> Reach {
        if aim.ours {
            return Reach::Unknown;
        }
        if self.fresh.is_none_or(|f| f.id != aim.id) {
            self.read(aim.id, aim.at);
        }
        let Some(fresh) = self.fresh else {
            return Reach::Unknown;
        };
        self.course.read(aim, dir, fresh.at, fresh.pair)
    }
}

#[cfg(test)]
mod tests;
