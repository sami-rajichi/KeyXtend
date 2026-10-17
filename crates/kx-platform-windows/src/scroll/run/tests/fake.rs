//! A desk the run tests set up: windows by box, a pointer, the kept entry, hung and closed windows, scripted reads, and a
//! record of every step, read and clock reading. The first app's window holding a point is the one behind ours, and
//! our list pages while it is among the windows.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use kx_module_api::ports::{Dir, Point, Rect, Screen, ScrollError};
use kx_module_api::{Mono, US_PER_MS};

use crate::lock;
use crate::screen::contains;
use crate::scroll::entry::Kept;
use crate::scroll::{Desk, Read, Seek, Spot, Wheel, Whose};

/// What the desk holds and what the run did to it.
#[derive(Debug, Default)]
pub(super) struct State {
    pub cursor: Option<Point>,
    /// Windows front first: the first box holding a point is the window there.
    pub windows: Vec<(Rect, Spot)>,
    pub kept: Option<Kept>,
    pub screens: Vec<Screen>,
    pub whole: Rect,
    pub gone: Vec<u64>,
    pub hung: Vec<u64>,
    /// Windows that owe a ping past its time.
    pub owing: Vec<u64>,
    pub pings: Vec<u64>,
    pub wheel: Option<Wheel>,
    pub wheel_reads: usize,
    /// What each read gives; none is a read that took too long.
    pub read: Option<Read>,
    /// Where each read looked, the way along whose axis, and its cap.
    pub reads: Vec<(Seek, Dir, Duration)>,
    pub busy: bool,
    pub steps_refused: bool,
    pub post_error: Option<ScrollError>,
    pub sent: Vec<(Dir, i32)>,
    pub posted: Vec<(u64, Point, Dir, i32)>,
    pub steps: Vec<(Seek, Dir, u16)>,
    /// Milliseconds the clock moves at each reading.
    pub tick_ms: u32,
    pub clock: u64,
    pub clock_reads: usize,
}

/// The fake desk; its clones share one state.
#[derive(Clone, Debug, Default)]
pub(super) struct Fake(pub Arc<Mutex<State>>);

impl Fake {
    /// Runs `f` on the state.
    pub fn with<T>(&self, f: impl FnOnce(&mut State) -> T) -> T {
        f(&mut lock(&self.0))
    }
}

impl Desk for Fake {
    fn cursor(&self) -> Option<Point> {
        self.with(|s| s.cursor)
    }

    fn spot(&self, p: Point) -> Option<Spot> {
        self.with(|s| s.windows.iter().find(|(r, _)| contains(*r, p)).map(|w| w.1))
    }

    fn behind(&self, p: Point) -> Option<Spot> {
        let theirs = |w: &&(Rect, Spot)| contains(w.0, p) && w.1.whose == Whose::Theirs;
        self.with(|s| s.windows.iter().find(theirs).map(|w| w.1))
    }

    fn paging(&self) -> Option<Spot> {
        let pages = |w: &&(Rect, Spot)| w.1.whose == Whose::Pages;
        self.with(|s| s.windows.iter().find(pages).map(|w| w.1))
    }

    fn kept(&self) -> Option<Kept> {
        self.with(|s| s.kept)
    }

    fn screens(&self) -> Vec<Screen> {
        self.with(|s| s.screens.clone())
    }

    fn whole(&self) -> Rect {
        self.with(|s| s.whole)
    }

    fn alive(&self, window: u64) -> bool {
        self.with(|s| !s.gone.contains(&window))
    }

    fn hung(&self, window: u64) -> bool {
        self.with(|s| s.hung.contains(&window) || s.owing.contains(&window))
    }

    fn ping(&self, window: u64) {
        self.with(|s| s.pings.push(window));
    }

    fn wheel(&self) -> Wheel {
        self.with(|s| {
            s.wheel_reads += 1;
            s.wheel.expect("the test sets a wheel")
        })
    }

    fn post(&self, window: u64, at: Point, dir: Dir, delta: i32) -> Result<(), ScrollError> {
        self.with(|s| {
            s.posted.push((window, at, dir, delta));
            s.post_error.map_or(Ok(()), Err)
        })
    }

    fn send(&self, dir: Dir, delta: i32) -> Result<(), ScrollError> {
        self.with(|s| s.sent.push((dir, delta)));
        Ok(())
    }

    fn read(&self, seek: Seek, way: Dir, cap: Duration) -> Option<Read> {
        self.with(|s| {
            s.reads.push((seek, way, cap));
            s.read
        })
    }

    fn idle(&self) -> bool {
        self.with(|s| !s.busy)
    }

    fn steps(&self, seek: Seek, dir: Dir, count: u16) -> bool {
        self.with(|s| {
            assert!(!s.busy, "steps only go to a free reader");
            if s.steps_refused {
                return false;
            }
            s.steps.push((seek, dir, count));
            true
        })
    }

    fn now(&self) -> Mono {
        self.with(|s| {
            s.clock_reads += 1;
            s.clock += u64::from(s.tick_ms) * u64::from(US_PER_MS);
            Mono::from_us(s.clock)
        })
    }
}
