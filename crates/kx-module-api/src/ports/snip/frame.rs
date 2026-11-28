//! A frame that adjusts a region or a crop (plan 2026-11-26 "The frame"): eight handles, four sides and four corners;
//! a click picks one up and the next sets it, never a drag.

use crate::ports::{Point, Rect};

listed! {
    /// A frame's handle.
    pub enum Handle ("Every handle, clockwise from the top-left corner.") {
        /// The top-left corner.
        TopLeft,
        /// The top side.
        Top,
        /// The top-right corner.
        TopRight,
        /// The right side.
        Right,
        /// The bottom-right corner.
        BottomRight,
        /// The bottom side.
        Bottom,
        /// The bottom-left corner.
        BottomLeft,
        /// The left side.
        Left,
    }
}

/// A handle held, and where its click picked it up, so it moves by the pointer's travel from there.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Grip {
    /// The handle.
    pub handle: Handle,
    /// Where it was picked up, in the frame's pixels.
    pub at: Point,
}

/// A frame with its handles: the box no side passes, its smallest side, and the handle held.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    /// The box it keeps within: the screen or screens for a region, the picture for a crop.
    pub within: Rect,
    /// The smallest side a moved handle leaves, in the frame's pixels (`[snip] frame_min_px`).
    pub min: u16,
    /// The handle held, if any.
    pub grip: Option<Grip>,
}
