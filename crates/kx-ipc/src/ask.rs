//! What the host sends the media job: a picture to draw, a DIB to make from a PNG, a snip's raw pixels to make into a
//! PNG, or goodbye.

use crate::code::{Code, Codec, Kind};
use crate::error::IpcError;
use crate::frame::{Head, check_cap, check_side, read_array, read_tail, write_head};
use crate::limits::snip::{BGRX_BYTES, MAX_ENCODE_BYTES};
use crate::limits::{DRAW_HEAD, ENCODE_HEAD, MAKE_HEAD};
use crate::rules::ok_if;
use std::borrow::Cow;
use std::fmt;
use std::io::{Read, Write};

/// A message from the host to the worker; it borrows the image when written and owns it when read.
#[derive(Clone, PartialEq, Eq)]
pub enum Ask<'a> {
    /// Draw an image as RGBA8 pixels, no larger than `side` on its longest side.
    Draw {
        /// The request, which the reply repeats.
        id: u32,
        /// How `bytes` are coded.
        codec: Codec,
        /// The longest side wanted, from 1 to `MAX_SIDE` pixels.
        side: u32,
        /// The image, from 1 to `MAX_IMAGE_BYTES` bytes.
        bytes: Cow<'a, [u8]>,
    },
    /// Make a `CF_DIB` of a PNG, at most `max_bytes` long.
    Dib {
        /// The request, which the reply repeats.
        id: u32,
        /// The most bytes the DIB may hold, from 1 to `MAX_IMAGE_BYTES`.
        max_bytes: u32,
        /// The PNG, from 1 to `MAX_IMAGE_BYTES` bytes.
        bytes: Cow<'a, [u8]>,
    },
    /// Make a PNG of raw pixels, `BGRX_BYTES` each, dropping each pixel's unused byte (ADR-0056 decision 2).
    Encode {
        /// The request, which the reply repeats.
        id: u32,
        /// Pixels in a row, 1 at least.
        width: u32,
        /// Rows, 1 at least.
        height: u32,
        /// Bytes from one row's start to the next's, at least `width` × `BGRX_BYTES`.
        stride: u32,
        /// `stride` × `height` bytes, at most `MAX_ENCODE_BYTES`.
        bgrx: Cow<'a, [u8]>,
    },
    /// End: the worker exits after this frame.
    Bye,
}

/// The kinds an ask may be.
pub(crate) const KINDS: &[Kind] = &[Kind::Draw, Kind::MakeDib, Kind::Encode, Kind::Bye];

impl Ask<'_> {
    /// Reads one ask from the host.
    ///
    /// # Errors
    /// Any `IpcError`, `Closed` when the host ended between frames; the stream is out of step after any other.
    pub fn read<R: Read + ?Sized>(r: &mut R) -> Result<Ask<'static>, IpcError> {
        let head = Head::read(r, KINDS)?;
        match head.kind {
            Kind::Draw => read_draw(r, head),
            Kind::MakeDib => read_make(r, head),
            Kind::Encode => read_encode(r, head),
            Kind::Bye if head.id == 0 => Ok(Ask::Bye),
            Kind::Bye => Err(IpcError::Inconsistent),
            _ => Err(IpcError::BadKind),
        }
    }

    /// Writes this ask as one frame, then flushes.
    ///
    /// # Errors
    /// `BadField`, `TooLong` or `Inconsistent`, before any byte is written, for an ask the worker would refuse;
    /// `Io` when the stream fails.
    pub fn write<W: Write + ?Sized>(&self, w: &mut W) -> Result<(), IpcError> {
        match self {
            Self::Draw {
                id,
                codec,
                side,
                bytes,
            } => {
                check_side(*side)?;
                write_head(w, Kind::Draw, *id, DRAW_HEAD.saturating_add(bytes.len()))?;
                w.write_all(&[codec.code()])?;
                w.write_all(&side.to_le_bytes())?;
                w.write_all(bytes)?;
            }
            Self::Dib {
                id,
                max_bytes,
                bytes,
            } => {
                check_cap(*max_bytes)?;
                write_head(w, Kind::MakeDib, *id, MAKE_HEAD.saturating_add(bytes.len()))?;
                w.write_all(&max_bytes.to_le_bytes())?;
                w.write_all(bytes)?;
            }
            Self::Encode { .. } => self.write_encode(w)?,
            Self::Bye => write_head(w, Kind::Bye, 0, 0)?,
        }
        w.flush()?;
        Ok(())
    }

    /// Writes an encode's sides, once its pixels are checked against them, then its pixels.
    fn write_encode<W: Write + ?Sized>(&self, w: &mut W) -> Result<(), IpcError> {
        if let Self::Encode {
            id,
            width,
            height,
            stride,
            bgrx,
        } = self
        {
            let len = encode_len(*width, *height, *stride)?;
            if bgrx.len() != len {
                return Err(IpcError::Inconsistent);
            }
            write_head(w, Kind::Encode, *id, ENCODE_HEAD + len)?;
            [width, height, stride]
                .into_iter()
                .try_for_each(|n| w.write_all(&n.to_le_bytes()))?;
            w.write_all(bgrx)?;
        }
        Ok(())
    }
}

/// The pixel bytes `height` rows of `stride` take, once each side is 1 or more, a row of `width` fits its stride and
/// they all fit `MAX_ENCODE_BYTES`.
fn encode_len(width: u32, height: u32, stride: u32) -> Result<usize, IpcError> {
    let size = |v: u32| usize::try_from(v).map_err(|_| IpcError::BadField);
    let (width, height, stride) = (size(width)?, size(height)?, size(stride)?);
    let row = width.checked_mul(BGRX_BYTES).ok_or(IpcError::BadField)?;
    let len = stride.checked_mul(height).ok_or(IpcError::BadField)?;
    ok_if(width > 0 && height > 0 && row <= stride && len <= MAX_ENCODE_BYTES)?;
    Ok(len)
}

/// Reads a draw's fields, checks them, then reads its image.
fn read_draw<R: Read + ?Sized>(r: &mut R, head: Head) -> Result<Ask<'static>, IpcError> {
    let [codec, s0, s1, s2, s3] = read_array::<DRAW_HEAD, _>(r)?;
    let codec = Codec::from_code(codec).ok_or(IpcError::BadField)?;
    let side = check_side(u32::from_le_bytes([s0, s1, s2, s3]))?;
    let len = head
        .len
        .checked_sub(DRAW_HEAD)
        .ok_or(IpcError::Inconsistent)?;
    let bytes = Cow::Owned(read_tail(r, len)?);
    Ok(Ask::Draw {
        id: head.id,
        codec,
        side,
        bytes,
    })
}

/// Reads a DIB ask's cap, checks it, then reads its PNG.
fn read_make<R: Read + ?Sized>(r: &mut R, head: Head) -> Result<Ask<'static>, IpcError> {
    let max_bytes = check_cap(u32::from_le_bytes(read_array::<MAKE_HEAD, _>(r)?))?;
    let len = head
        .len
        .checked_sub(MAKE_HEAD)
        .ok_or(IpcError::Inconsistent)?;
    let bytes = Cow::Owned(read_tail(r, len)?);
    Ok(Ask::Dib {
        id: head.id,
        max_bytes,
        bytes,
    })
}

/// Reads an encode's sides, checks the length against them, then reads its pixels.
fn read_encode<R: Read + ?Sized>(r: &mut R, head: Head) -> Result<Ask<'static>, IpcError> {
    let mut side = || read_array(r).map(u32::from_le_bytes);
    let (width, height, stride) = (side()?, side()?, side()?);
    let len = encode_len(width, height, stride)?;
    if head.len.checked_sub(ENCODE_HEAD) != Some(len) {
        return Err(IpcError::Inconsistent);
    }
    let bgrx = Cow::Owned(read_tail(r, len)?);
    Ok(Ask::Encode {
        id: head.id,
        width,
        height,
        stride,
        bgrx,
    })
}

impl fmt::Debug for Ask<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (name, id, len) = match self {
            Self::Draw { id, bytes, .. } => ("Draw", id, bytes.len()),
            Self::Dib { id, bytes, .. } => ("Dib", id, bytes.len()),
            Self::Encode { id, bgrx, .. } => ("Encode", id, bgrx.len()),
            Self::Bye => return f.write_str("Bye"),
        };
        let mut d = f.debug_struct(name);
        let s = d.field("id", id);
        let s = match self {
            Self::Draw { codec, side, .. } => s.field("codec", codec).field("side", side),
            Self::Dib { max_bytes, .. } => s.field("max_bytes", max_bytes),
            Self::Encode {
                width,
                height,
                stride,
                ..
            } => s
                .field("width", width)
                .field("height", height)
                .field("stride", stride),
            Self::Bye => s,
        };
        s.field("len", &len).finish()
    }
}
