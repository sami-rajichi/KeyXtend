//! What the host sends the worker: a picture to draw, or goodbye.

use crate::code::{Code, Codec, Kind};
use crate::error::IpcError;
use crate::frame::{Head, check_side, read_array, read_tail, write_head};
use crate::limits::DRAW_HEAD;
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
    /// End: the worker exits after this frame.
    Bye,
}

/// The kinds an ask may be.
pub(crate) const KINDS: &[Kind] = &[Kind::Draw, Kind::Bye];

impl Ask<'_> {
    /// Reads one ask from the host.
    ///
    /// # Errors
    /// Any `IpcError`, `Closed` when the host ended between frames; the stream is out of step after any other.
    pub fn read<R: Read + ?Sized>(r: &mut R) -> Result<Ask<'static>, IpcError> {
        let head = Head::read(r, KINDS)?;
        match head.kind {
            Kind::Draw => read_draw(r, head),
            Kind::Bye if head.id == 0 => Ok(Ask::Bye),
            Kind::Bye => Err(IpcError::Inconsistent),
            Kind::Pixels | Kind::Failed => Err(IpcError::BadKind),
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
            Self::Bye => write_head(w, Kind::Bye, 0, 0)?,
        }
        w.flush()?;
        Ok(())
    }
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

impl fmt::Debug for Ask<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Draw {
                id,
                codec,
                side,
                bytes,
            } => f
                .debug_struct("Draw")
                .field("id", id)
                .field("codec", codec)
                .field("side", side)
                .field("len", &bytes.len())
                .finish(),
            Self::Bye => f.write_str("Bye"),
        }
    }
}
