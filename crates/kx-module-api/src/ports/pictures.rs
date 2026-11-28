//! Pictures (ADR-0034 decisions 3 and 5, ADR-0056 decision 2): image bytes in, small RGBA pixels or a PNG's DIB out,
//! or a snip's raw pixels in and a PNG out, made outside our process by the media worker, so it needs no capability.
//! Both sides are private content, so no `Debug` prints bytes.

use super::{BGRX_BYTES, ImageCodec, Shot};
use crate::{Capability, Redacted, ServiceId, ServiceKey};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;

/// Bytes in one RGBA8 pixel.
pub const RGBA_BYTES: usize = 4;

/// The worker's own facts from its frames' crate: its decoder's budget, and the most times its length a read holds.
pub use kx_ipc::{DECODE_BYTES as WORKER_DECODE_BYTES, READ_TIMES as WORKER_READ_TIMES};

/// How the platform runs the worker; the module fills it from its settings.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PictureConfig {
    /// The longest one picture may take, in ms, before the worker is stopped.
    pub deadline_ms: u32,
    /// How long the worker may sit idle before it stops, in seconds.
    pub idle_s: u32,
    /// The most memory the worker may take, in MB.
    pub memory_mb: u32,
    /// The most pixels an image may hold; a larger one is refused before it is decoded.
    pub max_pixels: u64,
    /// The wait before a failed worker starts again, in ms; it doubles with each failure in a row.
    pub restart_ms: u32,
    /// The longest wait before a failed worker starts again, in ms.
    pub restart_max_ms: u32,
}

/// One picture to draw.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PictureAsk {
    /// How the bytes are coded.
    pub codec: ImageCodec,
    /// The image's bytes.
    pub bytes: Redacted<Vec<u8>>,
    /// The longest side wanted, in pixels; a smaller image keeps its size.
    pub side: u32,
}

/// A `CF_DIB` to make from a PNG, for an insert that sets both.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DibAsk {
    /// The PNG's bytes.
    pub png: Redacted<Vec<u8>>,
    /// The most bytes the DIB may hold; a larger one is refused before it is decoded.
    pub max_bytes: u64,
}

/// A PNG to make from a snip's raw pixels, for Copy and Save; the worker drops each pixel's unused byte.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EncodeAsk {
    /// The pixels.
    pub shot: Shot,
}

impl EncodeAsk {
    /// The most pixel bytes an ask holds: the largest frozen copy `shot_cap_mb` allows, the frames' own.
    pub const MAX_BYTES: usize = kx_ipc::MAX_ENCODE_BYTES;

    /// Refuses pixels past `MAX_BYTES`.
    ///
    /// # Errors
    /// `PictureError::TooBig` past it.
    pub fn check(&self) -> Result<(), PictureError> {
        let (width, height) = (self.shot.width(), self.shot.height());
        fits(width, height)
            .then_some(())
            .ok_or(PictureError::TooBig)
    }
}

impl EncodeAsk {
    /// The media job's memory cap this ask needs, in MB: its pixels, the PNG of its RGB rows at most, and a base.
    #[must_use]
    pub fn memory_mb(&self) -> u32 {
        memory_mb(self.shot.width(), self.shot.height())
    }
}

/// The media job's own memory beside an Encode's pixels and PNG, in MB, as an 8192 by 8192 noise picture peaking at
/// 455 MiB showed.
const ENCODE_BASE_MB: u64 = 16;
/// The bytes of a pixel the worker drops before the PNG: the unused one.
const UNUSED_BYTES: usize = 1;
/// The byte a PNG row starts with: its filter type.
const ROW_FILTER_BYTES: u64 = 1;

/// The memory cap an Encode of `width` by `height` pixels needs, in MB, rounded up; `u32::MAX` past it.
fn memory_mb(width: u32, height: u32) -> u32 {
    let bytes = |n: usize| u64::try_from(n).unwrap_or(u64::MAX);
    let (w, h) = (u64::from(width), u64::from(height));
    let raw = w.saturating_mul(h).saturating_mul(bytes(BGRX_BYTES));
    let rgb = bytes(BGRX_BYTES - UNUSED_BYTES);
    let rows = w
        .saturating_mul(rgb)
        .saturating_add(ROW_FILTER_BYTES)
        .saturating_mul(h);
    let png = usize::try_from(rows).map_or(u64::MAX, |r| bytes(kx_ipc::png_most(r)));
    let mb = raw.saturating_add(png).div_ceil(super::MB);
    u32::try_from(mb.saturating_add(ENCODE_BASE_MB)).unwrap_or(u32::MAX)
}

/// True when `width` by `height` raw pixels hold at most `EncodeAsk::MAX_BYTES`.
fn fits(width: u32, height: u32) -> bool {
    let wide = usize::try_from(width).ok();
    let len = wide.zip(usize::try_from(height).ok());
    let bytes = len.and_then(|(w, h)| w.checked_mul(h)?.checked_mul(BGRX_BYTES));
    bytes.is_some_and(|n| n <= EncodeAsk::MAX_BYTES)
}

/// Checked RGBA8 pixels, row by row from the top; cloning shares the bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pixels {
    /// Pixels in a row.
    width: u32,
    /// Rows.
    height: u32,
    /// `width` × `height` × `RGBA_BYTES` bytes.
    rgba: Redacted<Arc<[u8]>>,
}

impl Pixels {
    /// `rgba` as `width` by `height` pixels drawn for an ask of longest side `side`; `None` when a side is 0 or over
    /// `side`, or `rgba` is not `width` × `height` × `RGBA_BYTES` bytes long.
    #[must_use]
    pub fn new(width: u32, height: u32, rgba: Vec<u8>, side: u32) -> Option<Self> {
        let fits = |n: u32| (1..=side).contains(&n);
        let len = usize::try_from(width)
            .ok()?
            .checked_mul(usize::try_from(height).ok()?)?
            .checked_mul(RGBA_BYTES)?;
        (fits(width) && fits(height) && rgba.len() == len).then(|| Self {
            width,
            height,
            rgba: Redacted::new(rgba.into()),
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
    pub const fn rgba(&self) -> &Redacted<Arc<[u8]>> {
        &self.rgba
    }
}

/// Why a picture was not drawn.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum PictureError {
    /// The bytes are not an image the worker reads.
    #[error("the image could not be read")]
    Refused,
    /// The worker gave no answer within `PictureConfig::deadline_ms`.
    #[error("the picture took too long")]
    Timeout,
    /// The worker ended or answered badly on this image, so it is not asked again.
    #[error("the picture worker failed on this image")]
    Gone,
    /// The worker rests after a failure or could not start, so this image was not tried.
    #[error("the picture worker is down")]
    Down {
        /// How long until the worker may start again.
        retry_in: Duration,
    },
    /// The image holds more than `PictureConfig::max_pixels` pixels.
    #[error("the image holds too many pixels")]
    TooBig,
}

/// A session with the media worker on the platform's side. Dropping it stops the worker.
pub trait PictureRun: Send {
    /// Draws `ask` within its longest side, one picture at a time, within `PictureConfig::deadline_ms`.
    ///
    /// # Errors
    /// `Refused` for bytes that are no image, `TooBig` past `max_pixels`, `Timeout` past the deadline, `Gone` when the
    /// worker ends or answers badly on it, and `Down` when the worker rests or cannot start, so it was not tried.
    fn draw(&mut self, ask: PictureAsk) -> Result<Pixels, PictureError>;

    /// Makes the `CF_DIB` of `ask`'s PNG within its `max_bytes`, one ask at a time, within `deadline_ms`. Our side
    /// checks only its length, never its bytes.
    ///
    /// # Errors
    /// As `draw`; `TooBig` also when the DIB would pass `max_bytes`.
    fn dib(&mut self, ask: DibAsk) -> Result<Redacted<Vec<u8>>, PictureError>;

    /// Makes the PNG of `ask`'s pixels, one ask at a time, within `deadline_ms`, so our process links no image crate.
    ///
    /// # Errors
    /// As `draw`; `TooBig` when `ask.check` refuses it.
    fn encode(&mut self, ask: EncodeAsk) -> Result<Redacted<Vec<u8>>, PictureError>;
}

/// Draws images outside our process; the platform provides it.
pub trait Pictures: Send + Sync {
    /// Starts a session as `cfg` says; the worker itself starts at the first draw.
    fn open(&self, cfg: PictureConfig) -> Box<dyn PictureRun>;
}

/// The pictures service; it needs no capability, as it only turns bytes it is given into pixels.
pub struct PicturesKey;

impl ServiceKey for PicturesKey {
    type Api = dyn Pictures;
    const ID: ServiceId = ServiceId::new("pictures");
    const CAPABILITY: Option<Capability> = None;
}

#[cfg(test)]
mod tests;
