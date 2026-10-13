//! The pointer overlay's state: the mouse modes, the last pointer point, the hold ring and its burst, which QML reads
//! as one JSON. A bus call only stores and wakes, at most one `Wake::Assist` waits for QML's read, and nothing here
//! logs.

use super::{Hub, Wake, json};
use kx_module_api::ports::{AssistChanged, AssistNote, GrabState, Point};
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};

/// A hold's ring: where the press began, when the ring shows and how long its waves run, in ms.
#[derive(Copy, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct Ring {
    x: i32,
    y: i32,
    after_ms: u32,
    wave_ms: u32,
    seq: u64,
}

/// A fired hold's burst, where it right-clicked.
#[derive(Copy, Clone, PartialEq, Eq, Serialize)]
struct Burst {
    x: i32,
    y: i32,
    seq: u64,
}

/// Where Grab stands, as QML reads it.
#[derive(Copy, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
enum Grab {
    #[default]
    Off,
    Armed,
    Carrying,
}

impl From<GrabState> for Grab {
    fn from(g: GrabState) -> Self {
        match g {
            GrabState::Off => Self::Off,
            GrabState::Armed => Self::Armed,
            GrabState::Carrying => Self::Carrying,
        }
    }
}

/// What the overlay shows; `seq` grows on each change.
#[derive(Copy, Clone, Default, PartialEq, Eq, Serialize)]
struct Shown {
    live: bool,
    rclick: bool,
    grab: Grab,
    point: Option<Point>,
    ring: Option<Ring>,
    burst: Option<Burst>,
    seq: u64,
}

impl Shown {
    /// True while no mode is on or the mouse module is not running, so nothing shows.
    fn idle(&self) -> bool {
        !self.live || (!self.rclick && self.grab == Grab::Off)
    }
}

/// The state as QML reads it, with the pointer's size.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct View {
    #[serde(flatten)]
    shown: Shown,
    pointer_px: u32,
}

/// The overlay's state, and whether a wake waits for QML to read it.
#[derive(Default)]
pub(super) struct Assist {
    shown: Mutex<Shown>,
    waiting: AtomicBool,
}

impl Assist {
    fn lock(&self) -> MutexGuard<'_, Shown> {
        self.shown.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Hub {
    /// The overlay's state as JSON for QML, which lets the next change wake again; it holds points, so never log it.
    #[must_use]
    pub fn assist_json(&self) -> String {
        self.assist.waiting.store(false, Ordering::SeqCst);
        let shown = *self.assist.lock();
        let pointer_px = self.lock().sys.pointer_px;
        json(&View { shown, pointer_px })
    }

    /// Takes the mouse module's state; with no mode on, the point, ring and burst go. The keyboard reads its state
    /// again.
    pub(super) fn assist_changed(&self, now: AssistChanged) {
        self.change(|s, _| {
            (s.live, s.rclick, s.grab) = (now.live, now.rclick, now.grab.into());
            if s.idle() {
                (s.point, s.ring, s.burst) = (None, None, None);
            }
        });
        self.wake(Wake::Keys);
    }

    /// Keeps the pointer's point while a mode is on.
    pub(super) fn pointer_moved(&self, at: Point) {
        self.change(|s, _| {
            if !s.idle() {
                s.point = Some(at);
            }
        });
    }

    /// Follows a hold: its start rings for the time left after the ring shows, its fire bursts in place of the ring,
    /// and its end or a cancelled carry clears the ring.
    pub(super) fn assist_note(&self, note: &AssistNote) {
        self.change(|s, seq| match *note {
            AssistNote::HoldStart {
                at,
                ring_after_ms,
                hold_ms,
            } => {
                let wave_ms = hold_ms.saturating_sub(ring_after_ms);
                let (x, y, after_ms) = (at.x, at.y, ring_after_ms);
                s.ring = Some(Ring {
                    x,
                    y,
                    after_ms,
                    wave_ms,
                    seq,
                });
            }
            AssistNote::Fired { at } => {
                s.ring = None;
                s.burst = Some(Burst {
                    x: at.x,
                    y: at.y,
                    seq,
                });
            }
            AssistNote::HoldEnd | AssistNote::Cancelled => s.ring = None,
            AssistNote::Picked { .. } | AssistNote::Dropped { .. } => {}
        });
    }

    /// The pointer's size changed: the overlay reads its state again.
    pub(super) fn pointer_sized(&self) {
        self.assist.lock().seq += 1;
        self.nudge();
    }

    /// Hands a new waker the overlay wake still waiting for a read, as the last waker's QML may never have read it.
    pub(super) fn rewake_assist(&self) {
        if self.assist.waiting.load(Ordering::SeqCst) {
            self.wake(Wake::Assist);
        }
    }

    /// Applies `f`, which gets the seq a change takes; a change keeps that seq and wakes QML.
    fn change(&self, f: impl FnOnce(&mut Shown, u64)) {
        let changed = {
            let mut s = self.assist.lock();
            let (before, seq) = (*s, s.seq + 1);
            f(&mut s, seq);
            let changed = *s != before;
            if changed {
                s.seq = seq;
            }
            changed
        };
        if changed {
            self.nudge();
        }
    }

    /// Wakes QML for the overlay unless a wake already waits for its read.
    fn nudge(&self) {
        if !self.assist.waiting.swap(true, Ordering::SeqCst) {
            self.wake(Wake::Assist);
        }
    }
}
