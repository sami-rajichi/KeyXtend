//! A fake freeze: a desktop the test scripts, with screens at any place and scaling, painted so each pixel tells its
//! desktop point, windows offered by the platform's own rule (edge 3), and the pointer; refused past the cap before any
//! paint, as the port says.

use crate::lock;
use kx_module_api::ports::{
    BGRX_BYTES, FreezeError, Frozen, FrozenScreen, Part, Point, Rect, ScreenFreeze, Shot, ShotCap,
};
use kx_platform::snip::offered;
use std::sync::{Arc, Mutex};

pub use kx_platform::snip::Seen;

/// The pixel the fake paints at desktop point `at`: the low bytes of x, then of y, so a cut is checked to the pixel.
#[must_use]
pub fn pixel_at(at: Point) -> [u8; BGRX_BYTES] {
    let (x, y) = (at.x.to_le_bytes(), at.y.to_le_bytes());
    [x[0], x[1], y[0], y[1]]
}

/// The screen at `rect`, read at `dpi`, each pixel `pixel_at` its point; `None` when it has no pixels.
#[must_use]
pub fn painted(rect: Rect, dpi: u32) -> Option<FrozenScreen> {
    Part::new(rect)?;
    let (right, bottom) = (rect.x + rect.w, rect.y + rect.h);
    let row = move |y| (rect.x..right).flat_map(move |x| pixel_at(Point { x, y }));
    let bgrx = (rect.y..bottom).flat_map(row).collect();
    let shot = Shot::new(
        u32::try_from(rect.w).ok()?,
        u32::try_from(rect.h).ok()?,
        bgrx,
    )?;
    FrozenScreen::new(rect, dpi, shot)
}

/// The bytes a screen at `rect` holds, 0 for one with no pixels.
fn size(rect: Rect) -> u64 {
    if Part::new(rect).is_none() {
        return 0;
    }
    let side = |n: i32| u64::try_from(n).unwrap_or_default();
    let pixel = u64::try_from(BGRX_BYTES).unwrap_or(u64::MAX);
    side(rect.w)
        .saturating_mul(side(rect.h))
        .saturating_mul(pixel)
}

/// The scripted desktop, and every cap asked.
#[derive(Debug, Default)]
struct Desk {
    /// Each screen's place and DPI.
    screens: Vec<(Rect, u32)>,
    /// Each window's frame and how it shows, front first.
    windows: Vec<(Rect, Seen)>,
    pointer: Point,
    /// The error every freeze gives while set.
    error: Option<FreezeError>,
    caps: Vec<ShotCap>,
}

/// A freeze shared by its clones: each freeze paints the scripted screens anew; a screen with no pixels is left out.
#[derive(Clone, Debug, Default)]
pub struct FakeScreenFreeze {
    desk: Arc<Mutex<Desk>>,
}

impl FakeScreenFreeze {
    /// A desktop of `screens`, each a place and a DPI, with no window and the pointer at `pointer`.
    #[must_use]
    pub fn new(screens: Vec<(Rect, u32)>, pointer: Point) -> Self {
        let desk = Desk {
            screens,
            pointer,
            ..Desk::default()
        };
        Self {
            desk: Arc::new(Mutex::new(desk)),
        }
    }

    /// Replaces the screens, as when one is plugged in or out.
    pub fn set_screens(&self, screens: Vec<(Rect, u32)>) {
        lock(&self.desk).screens = screens;
    }

    /// Sets the windows, front first, each with how it shows.
    pub fn set_windows(&self, windows: Vec<(Rect, Seen)>) {
        lock(&self.desk).windows = windows;
    }

    /// Moves the pointer.
    pub fn set_pointer(&self, at: Point) {
        lock(&self.desk).pointer = at;
    }

    /// Refuses every later freeze with `error`, or takes them again with `None`.
    pub fn set_error(&self, error: Option<FreezeError>) {
        lock(&self.desk).error = error;
    }

    /// Every cap asked, refused freezes too, oldest first.
    #[must_use]
    pub fn caps(&self) -> Vec<ShotCap> {
        lock(&self.desk).caps.clone()
    }
}

impl ScreenFreeze for FakeScreenFreeze {
    fn freeze(&self, cap: ShotCap) -> Result<Frozen, FreezeError> {
        let mut desk = lock(&self.desk);
        desk.caps.push(cap);
        desk.error.map_or(Ok(()), Err)?;
        let sizes = desk.screens.iter().map(|&(rect, _)| size(rect));
        if sizes.fold(0, u64::saturating_add) > cap.bytes() {
            return Err(FreezeError::TooBig);
        }
        let screens = desk.screens.iter().filter_map(|&(r, dpi)| painted(r, dpi));
        Ok(Frozen {
            screens: screens.collect(),
            windows: offered(desk.windows.iter().copied()),
            pointer: desk.pointer,
        })
    }
}

#[cfg(test)]
mod tests;
