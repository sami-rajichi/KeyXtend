//! What the media job sends back: pixels, a DIB, a PNG, or a fixed reason it made none.

use crate::code::{Code, Kind, Why};
use crate::error::IpcError;
use crate::frame::{Head, check_side, read_array, read_tail, write_head};
use crate::limits::{FAILED_LEN, MAX_PIXELS_LEN, PIXELS_HEAD, RGBA_BYTES};
use std::fmt;
use std::io::{Read, Write};

/// A message from the worker to the host, answering one ask.
#[derive(Clone, PartialEq, Eq)]
pub enum Reply {
    /// RGBA8 pixels, row by row from the top.
    Pixels {
        /// The draw this answers.
        id: u32,
        /// Pixels in a row, from 1 to `MAX_SIDE`.
        width: u32,
        /// Rows, from 1 to `MAX_SIDE`.
        height: u32,
        /// `width` × `height` × `RGBA_BYTES` bytes.
        rgba: Vec<u8>,
    },
    /// A `CF_DIB` made from a PNG.
    Dib {
        /// The ask this answers.
        id: u32,
        /// The DIB, from 1 to `MAX_IMAGE_BYTES` bytes.
        bytes: Vec<u8>,
    },
    /// The PNG of an `Encode`'s pixels.
    Png {
        /// The ask this answers.
        id: u32,
        /// The PNG, from 1 to `MAX_PNG_BYTES` bytes; the host never decodes it.
        bytes: Vec<u8>,
    },
    /// No pixels, DIB or PNG, for a fixed reason.
    Failed {
        /// The ask this answers.
        id: u32,
        /// Why nothing was made.
        why: Why,
    },
}

/// The kinds a reply may be.
pub(crate) const KINDS: &[Kind] = &[Kind::Pixels, Kind::Dib, Kind::Png, Kind::Failed];

impl Reply {
    /// Reads one reply to a draw or a DIB ask, whose payload is at most `MAX_PIXELS_LEN`, so a worker that decodes
    /// hostile images cannot make the host hold an `Encode`'s size.
    ///
    /// # Errors
    /// As `read_within`.
    pub fn read<R: Read + ?Sized>(r: &mut R) -> Result<Self, IpcError> {
        Self::read_within(r, MAX_PIXELS_LEN)
    }

    /// Reads one reply whose payload is at most `most` bytes; the host reads an `Encode`'s PNG within `png_most` of
    /// the pixel bytes it sent.
    ///
    /// # Errors
    /// Any `IpcError`, `TooLong` past `most` before any payload byte is read, and `Closed` when the worker ended
    /// between frames; the stream is out of step after any but `Closed`.
    pub fn read_within<R: Read + ?Sized>(r: &mut R, most: usize) -> Result<Self, IpcError> {
        let head = Head::read(r, KINDS)?;
        if head.len > most {
            return Err(IpcError::TooLong);
        }
        let id = head.id;
        match head.kind {
            Kind::Pixels => read_pixels(r, head),
            Kind::Dib => Ok(Reply::Dib {
                id,
                bytes: read_tail(r, head.len)?,
            }),
            Kind::Png => Ok(Reply::Png {
                id,
                bytes: read_tail(r, head.len)?,
            }),
            Kind::Failed => read_failed(r, head),
            _ => Err(IpcError::BadKind),
        }
    }

    /// Writes this reply as one frame, then flushes.
    ///
    /// # Errors
    /// `BadField`, `TooLong` or `Inconsistent`, before any byte is written, for a reply the host would refuse; `Io`
    /// when the stream fails.
    pub fn write<W: Write + ?Sized>(&self, w: &mut W) -> Result<(), IpcError> {
        match self {
            Self::Pixels {
                id,
                width,
                height,
                rgba,
            } => {
                let len = rgba_len(*width, *height)?;
                if rgba.len() != len {
                    return Err(IpcError::Inconsistent);
                }
                write_head(w, Kind::Pixels, *id, PIXELS_HEAD + len)?;
                w.write_all(&width.to_le_bytes())?;
                w.write_all(&height.to_le_bytes())?;
                w.write_all(rgba)?;
            }
            Self::Dib { id, bytes } | Self::Png { id, bytes } => {
                write_head(w, self.kind(), *id, bytes.len())?;
                w.write_all(bytes)?;
            }
            Self::Failed { id, why } => {
                write_head(w, Kind::Failed, *id, FAILED_LEN)?;
                w.write_all(&[why.code()])?;
            }
        }
        w.flush()?;
        Ok(())
    }

    const fn kind(&self) -> Kind {
        match self {
            Self::Pixels { .. } => Kind::Pixels,
            Self::Dib { .. } => Kind::Dib,
            Self::Png { .. } => Kind::Png,
            Self::Failed { .. } => Kind::Failed,
        }
    }

    /// The ask this reply answers.
    #[must_use]
    pub const fn id(&self) -> u32 {
        match self {
            Self::Pixels { id, .. }
            | Self::Dib { id, .. }
            | Self::Png { id, .. }
            | Self::Failed { id, .. } => *id,
        }
    }
}

/// The pixel bytes `width` by `height` takes, once both sides are in range.
fn rgba_len(width: u32, height: u32) -> Result<usize, IpcError> {
    let side = |n: u32| usize::try_from(check_side(n)?).map_err(|_| IpcError::BadField);
    side(width)?
        .checked_mul(side(height)?)
        .and_then(|n| n.checked_mul(RGBA_BYTES))
        .ok_or(IpcError::BadField)
}

/// Reads the sides, checks the length against them, then reads the pixels.
fn read_pixels<R: Read + ?Sized>(r: &mut R, head: Head) -> Result<Reply, IpcError> {
    let [w0, w1, w2, w3, h0, h1, h2, h3] = read_array::<PIXELS_HEAD, _>(r)?;
    let width = u32::from_le_bytes([w0, w1, w2, w3]);
    let height = u32::from_le_bytes([h0, h1, h2, h3]);
    let len = rgba_len(width, height)?;
    if head.len.checked_sub(PIXELS_HEAD) != Some(len) {
        return Err(IpcError::Inconsistent);
    }
    let rgba = read_tail(r, len)?;
    Ok(Reply::Pixels {
        id: head.id,
        width,
        height,
        rgba,
    })
}

/// Reads a failure's reason.
fn read_failed<R: Read + ?Sized>(r: &mut R, head: Head) -> Result<Reply, IpcError> {
    let [why] = read_array::<FAILED_LEN, _>(r)?;
    let why = Why::from_code(why).ok_or(IpcError::BadField)?;
    Ok(Reply::Failed { id: head.id, why })
}

impl fmt::Debug for Reply {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = f.debug_struct(match self {
            Self::Pixels { .. } => "Pixels",
            Self::Dib { .. } => "Dib",
            Self::Png { .. } => "Png",
            Self::Failed { .. } => "Failed",
        });
        s.field("id", &self.id());
        match self {
            Self::Pixels {
                width,
                height,
                rgba,
                ..
            } => s
                .field("width", width)
                .field("height", height)
                .field("len", &rgba.len()),
            Self::Dib { bytes, .. } | Self::Png { bytes, .. } => s.field("len", &bytes.len()),
            Self::Failed { why, .. } => s.field("why", why),
        }
        .finish()
    }
}
