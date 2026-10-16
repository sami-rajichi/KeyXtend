//! Scrolling other apps from the pad (ADR-0029): the target port, which only `Scroll` reaches, and the pad's bus
//! events, which need no capability. A target's point and box can show what the user looks at, so no `Debug` here
//! prints them.

use super::{Point, Rect};
use crate::{Capability, Event, Redacted, ServiceId, ServiceKey};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// A way to scroll.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Dir {
    /// Up.
    Up,
    /// Down.
    Down,
    /// Left.
    Left,
    /// Right.
    Right,
}

impl Dir {
    /// Every direction, in declaration order.
    pub const ALL: [Self; 4] = [Self::Up, Self::Down, Self::Left, Self::Right];

    /// The way back.
    #[must_use]
    pub const fn opposite(self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }

    /// True for left and right, false for up and down.
    #[must_use]
    pub const fn sideways(self) -> bool {
        matches!(self, Self::Left | Self::Right)
    }
}

/// How the platform finds, reads and moves targets; the module fills it from its settings.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ScrollConfig {
    /// How long a target may stay still under our steps, short of its end, before the platform tries another way, in
    /// milliseconds.
    pub check_ms: u32,
    /// The longest one read of a target may take, in milliseconds.
    pub read_ms: u32,
    /// How far past the keyboard's nearest edge the target point goes when the pointer's path is unknown, in physical
    /// pixels.
    pub edge_px: u32,
}

/// Names a target, so the module notices when the pointer picks a new one.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct TargetId(u64);

impl TargetId {
    /// Wraps the platform's raw id.
    #[must_use]
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    /// The raw id.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// The target found now; its `Debug` hides the point and the box.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Aim {
    /// Where to scroll, in physical pixels.
    pub at: Point,
    /// The pointer itself is on the target.
    pub on_pointer: bool,
    /// The box to outline, in physical pixels; none when the platform cannot tell.
    pub area: Option<Rect>,
    /// The target is one of our own windows, such as an open list.
    pub ours: bool,
    /// Which target this is.
    pub id: TargetId,
}

/// Shows the flags and id, and only whether there is a box.
impl fmt::Debug for Aim {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Aim")
            .field("at", &Redacted::new(self.at))
            .field("on_pointer", &self.on_pointer)
            .field("area", &self.area.map(Redacted::new))
            .field("ours", &self.ours)
            .field("id", &self.id)
            .finish()
    }
}

/// Whether a target can still move a way.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Reach {
    /// It can move further.
    More,
    /// It is at its end.
    End,
    /// The platform cannot tell.
    Unknown,
}

/// Why a target could not be found or moved.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum ScrollError {
    /// No target point is known yet: the pointer has not left the keyboard.
    #[error("no scroll target is known yet")]
    NoTarget,
    /// The target closed.
    #[error("the scroll target closed")]
    Gone,
    /// The OS refused.
    #[error("the system refused to scroll the target")]
    Refused,
}

/// A running scroll on the platform's side. Dropping it is the only way to close it.
pub trait ScrollRun: Send {
    /// Finds the target now: under the pointer, or at its last point outside the keyboard while it is on the keyboard.
    ///
    /// # Errors
    /// `NoTarget` when no point is known yet, `Gone` when the window there closed, `Refused` when the OS refuses.
    fn aim(&mut self) -> Result<Aim, ScrollError>;

    /// Turns the target at `aim` towards `dir` by `lines`, which is finite and at least 0 and may be a fraction; it
    /// never waits on a hung app.
    ///
    /// # Errors
    /// `Gone` when the target closed, `Refused` when the OS refuses the step.
    fn turn(&mut self, aim: &Aim, dir: Dir, lines: f64) -> Result<(), ScrollError>;

    /// Whether the target at `aim` can still move towards `dir`, read within `ScrollConfig::read_ms`.
    fn reach(&mut self, aim: &Aim, dir: Dir) -> Reach;
}

/// Scrolls other apps' windows where the user pointed; the platform provides it.
pub trait ScrollTarget: Send + Sync {
    /// Starts a run, as `config` says.
    ///
    /// # Errors
    /// `Refused` when the platform cannot start one.
    fn open(&self, config: ScrollConfig) -> Result<Box<dyn ScrollRun>, ScrollError>;
}

/// The scroll target service; it needs `Scroll`.
pub struct ScrollTargetKey;

impl ServiceKey for ScrollTargetKey {
    type Api = dyn ScrollTarget;
    const ID: ServiceId = ServiceId::new("scroll-target");
    const CAPABILITY: Option<Capability> = Some(Capability::Scroll);
}

/// A tap on a pad arrow: the keyboard asks the scroll module to go that way, go faster, or stop.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ScrollTap {
    /// The arrow tapped.
    pub dir: Dir,
}

impl Event for ScrollTap {
    const NAME: &'static str = "scroll-tap";
}

/// The pad's Stop button: the keyboard asks the scroll module to stop.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ScrollStop;

impl Event for ScrollStop {
    const NAME: &'static str = "scroll-stop";
}

/// The pointer went onto the pad or off it; the scroll module outlines the target while it is on.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ScrollPeek {
    /// The pointer is on the pad.
    pub on: bool,
}

impl Event for ScrollPeek {
    const NAME: &'static str = "scroll-peek";
}

/// The scroll module's state, sent on each change.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ScrollChanged {
    /// The scroll module runs; false when it stops.
    pub live: bool,
    /// The way it scrolls now; none while still.
    pub dir: Option<Dir>,
    /// It scrolls at the fast speed.
    pub fast: bool,
}

impl ScrollChanged {
    /// The scroll module not running and still: before it speaks, and after it stops or fails.
    pub const OFF: Self = Self {
        live: false,
        dir: None,
        fast: false,
    };
}

impl Event for ScrollChanged {
    const NAME: &'static str = "scroll-changed";
}

/// Asks the scroll module to send `ScrollChanged` now, for a module or the UI that starts later.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ScrollAsk;

impl Event for ScrollAsk {
    const NAME: &'static str = "scroll-ask";
}

/// The box the UI outlines, or none to hide the outline; its `Debug` hides the box.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct ScrollAimed {
    /// The target's box, in physical pixels.
    pub area: Option<Rect>,
}

impl Event for ScrollAimed {
    const NAME: &'static str = "scroll-aimed";
}

/// Shows only whether there is a box.
impl fmt::Debug for ScrollAimed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScrollAimed")
            .field("area", &self.area.map(Redacted::new))
            .finish()
    }
}

/// The scroll module asks the keyboard to turn its open list one page: back for left and up, on for right and down.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ScrollPage {
    /// Turn back a page.
    pub back: bool,
}

impl Event for ScrollPage {
    const NAME: &'static str = "scroll-page";
}

#[cfg(test)]
mod tests;
