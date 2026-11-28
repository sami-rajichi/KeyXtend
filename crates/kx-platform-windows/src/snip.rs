//! Snip on Windows (ADR-0056): the freeze grabs every monitor through GDI and reads every window's frame through DWM, in
//! physical pixels, over the neutral steps of `kx_platform::snip`; the recorder's facts; videos played through Explorer.

pub(crate) mod aware;
pub(crate) mod frames;
pub(crate) mod grab;
mod kit;
mod play;

pub use kit::WinKit;
pub use play::WinMediaOpen;

use kx_module_api::ports::{FreezeError, Frozen, FrozenScreen, Point, Rect, ScreenFreeze, ShotCap};
use kx_platform::snip::{Desk, Freezer, Seen};
use windows::Win32::Foundation::E_NOTIMPL;

use crate::screen::{cursor, whole_screens};
use aware::Aware;

/// The refusal when this Windows cannot read physical pixels, as before Windows 10 1703.
const NOT_PHYSICAL: FreezeError = FreezeError::Refused(E_NOTIMPL.0);

/// The Windows freeze; each call reads physical pixels on its caller's thread, whichever it is.
#[derive(Debug)]
pub struct WinScreenFreeze(Freezer<WinDesk>);

impl WinScreenFreeze {
    /// The freeze of what Windows shows.
    #[must_use]
    pub const fn new() -> Self {
        Self(Freezer::new(WinDesk))
    }
}

impl Default for WinScreenFreeze {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenFreeze for WinScreenFreeze {
    fn freeze(&self, cap: ShotCap) -> Result<Frozen, FreezeError> {
        held(Aware::physical(), || self.0.freeze(cap))
    }
}

/// Runs `freeze` while `physical` holds this thread in physical pixels; refuses it when the switch did not take.
fn held<T>(
    physical: Option<Aware>,
    freeze: impl FnOnce() -> Result<T, FreezeError>,
) -> Result<T, FreezeError> {
    let _physical = physical.ok_or(NOT_PHYSICAL)?;
    freeze()
}

/// What Windows shows now: its monitors, their pixels, its top-level windows and the pointer.
#[derive(Clone, Copy, Debug, Default)]
struct WinDesk;

impl Desk for WinDesk {
    fn screens(&self) -> Vec<(Rect, u32)> {
        whole_screens()
    }

    fn grab(&self, screens: &[(Rect, u32)]) -> Result<Vec<FrozenScreen>, FreezeError> {
        grab::grab(screens)
    }

    fn windows(&self) -> Vec<(Rect, Seen)> {
        frames::windows()
    }

    fn pointer(&self) -> Point {
        cursor().unwrap_or_else(|e| {
            tracing::debug!(error = %e, "the pointer could not be read for a freeze");
            Point::default()
        })
    }
}

#[cfg(test)]
mod tests;
