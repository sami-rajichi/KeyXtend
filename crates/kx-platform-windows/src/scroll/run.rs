//! One scroll run on a desk of windows (ADR-0029): it finds the target, turns it by the course's route and reads how far
//! it can go, with one UI Automation read a check along the latest way's axis. Pure over the `Desk` trait; `win` gives
//! Windows' desk.

use std::time::Duration;

use kx_module_api::Mono;
use kx_module_api::ports::{
    Aim, Dir, Point, Reach, Rect, Screen, ScrollConfig, ScrollError, ScrollRun, TargetId,
};

use super::entry::Kept;
use super::{Course, Route, Wheel, area};

mod look;
mod spot;

pub use look::{Read, Seek};
pub use spot::{Spot, Whose};

/// The system a run scrolls on: Windows in `win`, a fake in tests.
pub trait Desk {
    /// Where the pointer is; none when the system cannot say.
    fn cursor(&self) -> Option<Point>;
    /// The window at `p`; none when there is none.
    fn spot(&self, p: Point) -> Option<Spot>;
    /// The first other app's window at `p` in z-order, passing over ours; none when there is none.
    fn behind(&self, p: Point) -> Option<Spot>;
    /// Our shown window that pages, such as the open list; none while none shows.
    fn paging(&self) -> Option<Spot>;
    /// What the guard kept when the pointer last entered the keyboard.
    fn kept(&self) -> Option<Kept>;
    /// Every screen's work area.
    fn screens(&self) -> Vec<Screen>;
    /// The box around every screen.
    fn whole(&self) -> Rect;
    /// True while `window` exists.
    fn alive(&self, window: u64) -> bool;
    /// True while the system reports `window`'s app as hung, or `window` left a ping unanswered for too long.
    fn hung(&self, window: u64) -> bool;
    /// Asks whether `window` still answers its messages, never waiting; one ask is out at a time.
    fn ping(&self, window: u64);
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
    /// UI Automation's read from `seek` of the first element up that scrolls along `way`'s axis, waited on for `cap`
    /// at most; none when it took longer or the reader is stuck.
    fn read(&self, seek: Seek, way: Dir, cap: Duration) -> Option<Read>;
    /// True while UI Automation is free to take steps now.
    fn idle(&self) -> bool;
    /// Hands UI Automation `count` line steps towards `dir` from `seek`, never waited on; false when they were not
    /// taken.
    fn steps(&self, seek: Seek, dir: Dir, count: u16) -> bool;
    /// The time now, on the crate's clock.
    fn now(&self) -> Mono;
}

/// This check's read of one target along one axis (sideways or not): its percents and when they were read.
#[derive(Copy, Clone, Debug)]
struct Fresh {
    id: TargetId,
    sideways: bool,
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
    /// The way of the latest turn or reach, along whose axis each read looks; down until the first.
    way: Dir,
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
            way: Dir::Down,
            fresh: None,
            last: None,
        }
    }

    /// The point to scroll, whether the pointer is on it, and the window there: under the pointer, our open list while
    /// it shows, or the point the guard kept while the pointer is on the keyboard. Our window that now covers that point
    /// never takes a silent page turn: the app's window behind it does, or there is no target.
    fn locate(&self) -> Result<(Point, bool, Spot), ScrollError> {
        let cursor = self.desk.cursor().ok_or(ScrollError::Refused)?;
        let under = self.desk.spot(cursor).ok_or(ScrollError::Gone)?;
        if !under.whose.passed() {
            return Ok((cursor, true, under));
        }
        // The list wins whatever point was kept, as the pointer may cross the app between the list and the pad.
        if let Some(list) = self.desk.paging() {
            return Ok((cursor, false, list));
        }
        // Only the keyboard has a point kept for it: our other windows, and the keyboard before its watch holds, have
        // none, so the user is asked to point first.
        if under.whose == Whose::Ours {
            return Err(ScrollError::NoTarget);
        }
        let kept = self.desk.kept().ok_or(ScrollError::NoTarget)?;
        let at = kept.target(&self.desk.screens(), self.config.edge_px);
        let at = at.ok_or(ScrollError::NoTarget)?;
        let there = self.desk.spot(at).ok_or(ScrollError::Gone)?;
        let there = if there.whose.passed() {
            self.desk.behind(at).ok_or(ScrollError::NoTarget)?
        } else {
            there
        };
        Ok((at, false, there))
    }

    /// The box of target `id` at `at`, from this check's read; a read that misses, or a busy target, keeps the
    /// target's last box.
    fn look(&mut self, id: TargetId, at: Point) -> Option<Rect> {
        let read = if self.desk.hung(id.get()) {
            None
        } else {
            self.read(id, at)
        };
        let area = match read {
            Some(read) => read.area.and_then(|edges| area(edges, self.desk.whole())),
            None => self.last.filter(|(was, _)| *was == id).and_then(|(_, a)| a),
        };
        self.last = Some((id, area));
        area
    }

    /// Reads target `id` at `at` along the latest way's axis within `read_ms`, and keeps it as this check's read with
    /// one clock reading.
    fn read(&mut self, id: TargetId, at: Point) -> Option<Read> {
        let cap = Duration::from_millis(u64::from(self.config.read_ms));
        let read = self.desk.read(self.seek(id, at), self.way, cap);
        let pair = read.and_then(|r| r.pair);
        self.fresh = Some(Fresh {
            id,
            sideways: self.way.sideways(),
            pair,
            at: self.desk.now(),
        });
        read
    }

    /// Where UI Automation looks for target `id` at `at`: at the point while the target is the window there, else at
    /// the target's own window, as when ours now covers the point, so it never reads or steps ours.
    fn seek(&self, id: TargetId, at: Point) -> Seek {
        if self.desk.spot(at).is_some_and(|s| s.window == id.get()) {
            Seek::At(at)
        } else {
            Seek::Window(id.get())
        }
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
        if !ours {
            self.desk.ping(id.get());
        }
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
        self.way = dir;
        if !self.desk.alive(window) {
            return Err(ScrollError::Gone);
        }
        self.desk.ping(window);
        if self.desk.hung(window) || self.reader_busy(aim, dir) {
            return Ok(());
        }
        let Some(turn) = self.course.turn(aim, dir, lines, self.wheel) else {
            return Ok(());
        };
        self.fresh = None;
        let delta = turn.delta(dir);
        match turn.route {
            Route::Uia if self.desk.steps(self.seek(aim.id, aim.at), dir, turn.count) => Ok(()),
            Route::Uia => Err(ScrollError::Refused),
            Route::Send if self.wheel.at_pointer && self.under_pointer(window) => {
                self.desk.send(dir, delta)
            }
            Route::Send | Route::Post => self.desk.post(window, aim.at, dir, delta),
        }
    }

    fn reach(&mut self, aim: &Aim, dir: Dir) -> Reach {
        if aim.ours {
            return Reach::Unknown;
        }
        self.desk.ping(aim.id.get());
        if self.desk.hung(aim.id.get()) {
            return Reach::Unknown;
        }
        self.way = dir;
        if self
            .fresh
            .is_none_or(|f| f.id != aim.id || f.sideways != dir.sideways())
        {
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
