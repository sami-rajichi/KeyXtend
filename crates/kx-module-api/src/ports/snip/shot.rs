//! The freeze port (ADR-0056 decision 1): every screen grabbed as raw pixels at one moment, with the windows' frames
//! read then, refused past a cap; and the shots and parts a snip is cut from. No pixel ever prints.

use crate::ports::{MB, Point, Rect};
use crate::{Capability, Redacted, ServiceId, ServiceKey};
use std::ops::RangeInclusive;
use std::sync::Arc;
use thiserror::Error;

/// Bytes in one raw pixel, the frames' own: blue, green, red and one unused byte.
pub use kx_ipc::BGRX_BYTES;

/// The range of a freeze's cap (`shot_cap_mb`), in MB of `MB` bytes, the frames' own.
pub const SHOT_CAP_MB: RangeInclusive<u32> = kx_ipc::SNIP_SHOT_MB;

/// A freeze's cap, within `SHOT_CAP_MB`, so no frozen copy, nor a picture cut from one, outgrows `EncodeAsk`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ShotCap(u32);

impl ShotCap {
    /// `mb` MB as a cap; `None` outside `SHOT_CAP_MB`.
    #[must_use]
    pub fn new(mb: u32) -> Option<Self> {
        SHOT_CAP_MB.contains(&mb).then_some(Self(mb))
    }

    /// The cap in MB.
    #[must_use]
    pub const fn mb(self) -> u32 {
        self.0
    }

    /// The cap in bytes.
    #[must_use]
    pub fn bytes(self) -> u64 {
        u64::from(self.0) * MB
    }
}

/// Raw pixels of a screen or a picture, `BGRX_BYTES` each, rows from the top; cloning shares them, and `Debug` shows
/// only the size.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shot {
    /// Pixels in a row.
    width: u32,
    /// Rows.
    height: u32,
    /// `width` × `height` × `BGRX_BYTES` bytes.
    bgrx: Redacted<Arc<[u8]>>,
}

impl Shot {
    /// `bgrx` as `width` by `height` pixels; `None` when a side is 0 or `bgrx` is not exactly their bytes.
    #[must_use]
    pub fn new(width: u32, height: u32, bgrx: Vec<u8>) -> Option<Self> {
        let sized = bytes(width, height)? == bgrx.len();
        sized.then(|| Self {
            width,
            height,
            bgrx: Redacted::new(bgrx.into()),
        })
    }

    /// A `width` by `height` shot whose bytes `fill` writes once, in place, so a grab is never copied again; `None`
    /// when a side is 0, the size overflows, or `fill` says it could not.
    #[must_use]
    pub fn filled(width: u32, height: u32, fill: impl FnOnce(&mut [u8]) -> bool) -> Option<Self> {
        let mut bgrx: Arc<[u8]> = std::iter::repeat_n(0, bytes(width, height)?).collect();
        fill(Arc::get_mut(&mut bgrx)?).then(|| Self {
            width,
            height,
            bgrx: Redacted::new(bgrx),
        })
    }

    /// Pixels in a row.
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Rows.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// The bytes, row by row from the top.
    #[must_use]
    pub const fn bgrx(&self) -> &Redacted<Arc<[u8]>> {
        &self.bgrx
    }
}

/// The bytes of `width` by `height` pixels; `None` when a side is 0 or they pass the address range.
fn bytes(width: u32, height: u32) -> Option<usize> {
    let len = usize::try_from(width)
        .ok()?
        .checked_mul(usize::try_from(height).ok()?)?
        .checked_mul(BGRX_BYTES)?;
    (len > 0).then_some(len)
}

/// A part of the screens in physical desktop pixels, never empty, whose far edges stay in the pixel range: what a
/// picture or a video takes.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Part(Rect);

impl Part {
    /// `rect` as a part; `None` when it is empty or its right or bottom edge passes `i32::MAX`.
    #[must_use]
    pub fn new(rect: Rect) -> Option<Self> {
        let edges = rect.x.checked_add(rect.w).zip(rect.y.checked_add(rect.h));
        (rect.w > 0 && rect.h > 0 && edges.is_some()).then_some(Self(rect))
    }

    /// The part's rectangle.
    #[must_use]
    pub const fn rect(self) -> Rect {
        self.0
    }
}

/// One screen as frozen: where it lies on the desktop in physical pixels, its whole DPI, and its pixels.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrozenScreen {
    /// The whole screen, taskbar included.
    rect: Rect,
    /// Its DPI, as `Screen::dpi` reads it.
    dpi: u32,
    /// Its pixels, the size of `rect`.
    shot: Shot,
}

impl FrozenScreen {
    /// `shot` as the screen at `rect`, read at `dpi`; `None` when `rect` is not the shot's size.
    #[must_use]
    pub fn new(rect: Rect, dpi: u32, shot: Shot) -> Option<Self> {
        let size = (u32::try_from(rect.w).ok()?, u32::try_from(rect.h).ok()?);
        (size == (shot.width(), shot.height())).then_some(Self { rect, dpi, shot })
    }

    /// Where the screen lies, in physical desktop pixels.
    #[must_use]
    pub const fn rect(&self) -> Rect {
        self.rect
    }

    /// Its DPI.
    #[must_use]
    pub const fn dpi(&self) -> u32 {
        self.dpi
    }

    /// Its pixels.
    #[must_use]
    pub const fn shot(&self) -> &Shot {
        &self.shot
    }
}

/// Every screen frozen at one moment, what the Window way offers then, and where the pointer was.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frozen {
    /// The screens.
    pub screens: Vec<FrozenScreen>,
    /// The visible windows' frames, without shadows, front first; never ours, a cloaked or a minimised one (edge 3).
    pub windows: Vec<Rect>,
    /// Where the pointer was, so the capture bar goes on its screen.
    pub pointer: Point,
}

/// Why nothing was frozen.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum FreezeError {
    /// The screens together pass the cap (edge 4).
    #[error("the screens together pass the cap")]
    TooBig,
    /// The system refused to grab the screens, with its error code.
    #[error("the system refused to grab the screens (code {0})")]
    Refused(i32),
}

/// Freezes the screens; the platform provides it.
pub trait ScreenFreeze: Send + Sync {
    /// Grabs every screen as raw pixels at one moment, with the windows' frames read then; it takes tens of ms.
    ///
    /// # Errors
    /// `TooBig` when the screens together pass `cap`, so nothing is grabbed; `Refused` when the system says no. No cap
    /// outside `SHOT_CAP_MB` can be made, so none reaches the port.
    fn freeze(&self, cap: ShotCap) -> Result<Frozen, FreezeError>;
}

/// The freeze service; it needs `ScreenCapture`.
pub struct ScreenFreezeKey;

impl ServiceKey for ScreenFreezeKey {
    type Api = dyn ScreenFreeze;
    const ID: ServiceId = ServiceId::new("screen-freeze");
    const CAPABILITY: Option<Capability> = Some(Capability::ScreenCapture);
}
