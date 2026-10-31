//! Pictures (ADR-0034 decisions 3 and 5): image bytes in, small RGBA pixels out, drawn outside our process by the
//! media worker, so it needs no capability. Both sides are copied content, so no `Debug` here prints their bytes.

use super::ImageCodec;
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
