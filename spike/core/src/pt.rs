//! The screen point the spike's tools and kx-gates share.

/// A screen point in physical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Pt {
    /// Pixels from the left of the virtual desktop.
    pub x: i32,
    /// Pixels from the top of the virtual desktop.
    pub y: i32,
}
