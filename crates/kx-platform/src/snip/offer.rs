//! The windows a freeze offers the Window way (edge 3): shown ones only, never ours, front first.

use kx_module_api::ports::{Part, Rect};

/// How a top-level window shows at a freeze; only a `Shown` one is offered.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Seen {
    /// Shown, and not ours: one a user could pick.
    Shown,
    /// Not shown.
    Hidden,
    /// One of our own windows.
    Ours,
    /// Minimised.
    Minimised,
    /// Cloaked, as on another virtual desktop or a suspended app.
    Cloaked,
    /// A tool window, such as a floating palette, with no taskbar button.
    Tool,
    /// Click-through, as an overlay is, so it would take the pick of the window under it.
    Through,
    /// The system's own desktop behind every window.
    Desktop,
}

/// What an adapter tells of one top-level window, each fact read only when asked.
pub trait Looks {
    /// It shows.
    fn shown(&self) -> bool;
    /// It is one of ours.
    fn ours(&self) -> bool;
    /// It is the system's own desktop.
    fn desktop(&self) -> bool;
    /// It is minimised.
    fn minimised(&self) -> bool;
    /// It is a tool window.
    fn tool(&self) -> bool;
    /// Clicks pass through it to the windows below.
    fn through(&self) -> bool;
    /// The system cloaks it; the costliest fact, read last.
    fn cloaked(&self) -> bool;
}

impl Seen {
    /// How the window `looks` describes shows, its facts read in `Looks`' order and only until one rules it out.
    pub fn of(looks: &impl Looks) -> Self {
        if !looks.shown() {
            Self::Hidden
        } else if looks.ours() {
            Self::Ours
        } else if looks.desktop() {
            Self::Desktop
        } else if looks.minimised() {
            Self::Minimised
        } else if looks.tool() {
            Self::Tool
        } else if looks.through() {
            Self::Through
        } else if looks.cloaked() {
            Self::Cloaked
        } else {
            Self::Shown
        }
    }
}

/// The frames of `front_first` a freeze offers: the `Shown` windows with pixels, in the same order.
pub fn offered(front_first: impl IntoIterator<Item = (Rect, Seen)>) -> Vec<Rect> {
    let picked = |&(rect, seen): &(Rect, Seen)| seen == Seen::Shown && Part::new(rect).is_some();
    front_first
        .into_iter()
        .filter(picked)
        .map(|(rect, _)| rect)
        .collect()
}
