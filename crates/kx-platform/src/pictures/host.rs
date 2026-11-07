//! The media job on the neutral host: a draw's pixels within its side, a DIB within its cap, the worker's reasons as
//! the port's errors, and asks the worker would refuse refused before any worker starts.

use crate::worker::{Host, Lost, MediaFrames, Spawner};
use kx_ipc::{Ask, Codec, IpcError, MAX_IMAGE_BYTES, MAX_SIDE, Reply, Why};
use kx_module_api::Redacted;
use kx_module_api::ports::{DibAsk, ImageCodec, PictureAsk, PictureError, Pixels};
use std::borrow::Cow;
use std::io;

impl<S: Spawner<MediaFrames>> Host<S, MediaFrames> {
    /// Draws `ask` on the worker, started first if need be: one draw at a time, within the deadline.
    pub(super) fn draw(&mut self, ask: PictureAsk) -> Result<Pixels, PictureError> {
        let side = ask.side.min(MAX_SIDE);
        let (codec, bytes) = (codec(ask.codec), Cow::Owned(ask.bytes.into_inner()));
        let make = |id| {
            fits(Ask::Draw {
                id,
                codec,
                side,
                bytes,
            })
        };
        self.call(make, |reply| match reply {
            Reply::Pixels {
                width,
                height,
                rgba,
                ..
            } => Pixels::new(width, height, rgba, side).map(Ok),
            Reply::Failed { why, .. } => Some(Err(refusal(why))),
            Reply::Dib { .. } => None,
        })
    }

    /// Makes the DIB of `ask`'s PNG on the worker, within the deadline; only its length is checked, never its bytes.
    pub(super) fn dib(&mut self, ask: DibAsk) -> Result<Redacted<Vec<u8>>, PictureError> {
        let max_bytes = cap(ask.max_bytes);
        let bytes = Cow::Owned(ask.png.into_inner());
        let make = |id| {
            fits(Ask::Dib {
                id,
                max_bytes,
                bytes,
            })
        };
        let in_cap = |bytes: &[u8]| u32::try_from(bytes.len()).is_ok_and(|n| n <= max_bytes);
        self.call(make, |reply| match reply {
            Reply::Dib { bytes, .. } if in_cap(&bytes) => Some(Ok(Redacted::new(bytes))),
            Reply::Failed { why, .. } => Some(Err(refusal(why))),
            Reply::Dib { .. } | Reply::Pixels { .. } => None,
        })
    }
}

impl From<Lost> for PictureError {
    /// A late worker is a timeout, a failed one gone, and a resting one down for its wait.
    fn from(lost: Lost) -> Self {
        match lost {
            Lost::Late => Self::Timeout,
            Lost::Gone => Self::Gone,
            Lost::Down { retry_in } => Self::Down { retry_in },
        }
    }
}

/// `ask` when the worker would take it, by `kx-ipc`'s own writer checks.
fn fits(ask: Ask<'static>) -> Result<Ask<'static>, PictureError> {
    ask.write(&mut io::sink()).map_err(|e| match e {
        IpcError::TooLong => PictureError::TooBig,
        _ => PictureError::Refused,
    })?;
    Ok(ask)
}

/// The most bytes a DIB may hold, as a frame carries it: never past `MAX_IMAGE_BYTES`.
fn cap(max_bytes: u64) -> u32 {
    let largest = u64::try_from(MAX_IMAGE_BYTES).unwrap_or(u64::MAX);
    u32::try_from(max_bytes.min(largest)).unwrap_or(u32::MAX)
}

/// The frame's code for `codec`.
const fn codec(codec: ImageCodec) -> Codec {
    match codec {
        ImageCodec::Png => Codec::Png,
        ImageCodec::Dib => Codec::Dib,
    }
}

/// The port's error for the worker's reason; a failure inside the worker leaves that image undrawn.
const fn refusal(why: Why) -> PictureError {
    match why {
        Why::TooBig => PictureError::TooBig,
        Why::Refused | Why::Internal => PictureError::Refused,
    }
}

#[cfg(test)]
mod tests;
