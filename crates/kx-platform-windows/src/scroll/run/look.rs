//! What UI Automation reads at a target and where it starts looking; the debug output hides every point and box.

use std::fmt;

use kx_module_api::Redacted;
use kx_module_api::ports::Point;

/// What UI Automation read at a point: the box of the first element there that scrolls along the way asked (left, top,
/// right, bottom), and its percent pair (horizontal, vertical). Its `Debug` hides the box.
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

/// Where UI Automation starts its walk up to what scrolls: the element at a screen point, or a window's own element,
/// for a target that another window now covers.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Seek {
    /// The element at this screen point.
    At(Point),
    /// The element of the window with this raw id.
    Window(u64),
}

/// Shows the window, and only that there is a point.
impl fmt::Debug for Seek {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::At(p) => f.debug_tuple("At").field(&Redacted::new(p)).finish(),
            Self::Window(window) => f.debug_tuple("Window").field(window).finish(),
        }
    }
}
