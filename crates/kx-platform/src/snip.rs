//! Snip's OS-neutral half (ADR-0056): the freeze's steps over an adapter's `Desk` with the cap checked first (edge 4)
//! and the windows it offers (edge 3); the recorder and dialog hosts over the worker; the Snips folder; the play rule.

mod cap;
mod folder;
mod offer;
mod pick;
mod play;
mod record;

pub use cap::{fits, screen_bytes};
pub use folder::{DiskSnips, Room};
pub(crate) use folder::{save_fault, unsaved_part};
pub use offer::{Looks, Seen, offered};
pub use pick::WorkerFileAsk;
pub use play::playable;
pub use record::{Kit, WorkerRecorder};

use kx_module_api::ports::{
    FreezeError, Frozen, FrozenScreen, Part, Point, Rect, ScreenFreeze, ShotCap,
};

/// What an OS adapter reads for a freeze, all in physical desktop pixels.
pub trait Desk: Send + Sync {
    /// Every screen's whole rectangle, taskbar included, and its DPI.
    fn screens(&self) -> Vec<(Rect, u32)>;

    /// Grabs each of `screens`, a rectangle and a DPI, as one moment's raw pixels, in the same order.
    ///
    /// # Errors
    /// `Refused` with the system's code when it will not grab them.
    fn grab(&self, screens: &[(Rect, u32)]) -> Result<Vec<FrozenScreen>, FreezeError>;

    /// Every top-level window, front first, with how it shows; only a `Shown` one's frame need be read.
    fn windows(&self) -> Vec<(Rect, Seen)>;

    /// Where the pointer is, or the desktop's origin, on the main screen, when the system cannot say.
    fn pointer(&self) -> Point;
}

/// The freeze over an adapter's desk: the screens read, the cap checked, every screen grabbed in one go, then the
/// windows and the pointer.
#[derive(Debug)]
pub struct Freezer<D> {
    desk: D,
}

impl<D: Desk> Freezer<D> {
    /// A freeze over `desk`.
    pub const fn new(desk: D) -> Self {
        Self { desk }
    }
}

impl<D: Desk> ScreenFreeze for Freezer<D> {
    fn freeze(&self, cap: ShotCap) -> Result<Frozen, FreezeError> {
        let mut screens = self.desk.screens();
        screens.retain(|&(rect, _)| Part::new(rect).is_some());
        if !fits(screens.iter().map(|&(rect, _)| rect), cap) {
            return Err(FreezeError::TooBig);
        }
        let screens = self.desk.grab(&screens)?;
        Ok(Frozen {
            screens,
            windows: offered(self.desk.windows()),
            pointer: self.desk.pointer(),
        })
    }
}

#[cfg(test)]
mod tests;
