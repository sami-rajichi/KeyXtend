//! The media job (ADR-0034 decision 5): PNG and DIB bytes in, RGBA8 pixels no larger than the asked side out, rows
//! scaled as they are read; and a PNG made into a `CF_DIB` for an insert, within the size the host asks.

mod bitmap;
mod dib;
mod png;
mod scale;

use crate::serve::{Answer, Asked};
use kx_ipc::{Ask, Codec, IpcError, Reply, Why};
use std::io::{Read, Write};

impl Asked for Ask<'static> {
    type Reply = Reply;

    fn next<R: Read + ?Sized>(r: &mut R) -> Result<Self, IpcError> {
        Ask::read(r)
    }

    fn id(&self) -> Option<u32> {
        match self {
            Ask::Draw { id, .. } | Ask::Dib { id, .. } => Some(*id),
            Ask::Bye => None,
        }
    }
}

impl Answer for Reply {
    fn internal(id: u32) -> Self {
        Reply::Failed {
            id,
            why: Why::Internal,
        }
    }

    fn send<W: Write + ?Sized>(&self, w: &mut W) -> Result<(), IpcError> {
        self.write(w)
    }
}

/// Answers one media ask, a draw with `draw` and a DIB with `make`; goodbye is never answered, as the loop ends first.
pub fn reply<D, M>(ask: Ask<'_>, draw: D, make: M) -> Reply
where
    D: Fn(Codec, &[u8], u32) -> Result<Picture, Why>,
    M: Fn(&[u8], usize) -> Result<Vec<u8>, Why>,
{
    let (id, out) = match ask {
        Ask::Draw {
            id,
            codec,
            side,
            bytes,
        } => (id, draw(codec, &bytes, side).map(|p| pixels(id, p))),
        Ask::Dib {
            id,
            max_bytes,
            bytes,
        } => (
            id,
            make(&bytes, cap(max_bytes)).map(|bytes| Reply::Dib { id, bytes }),
        ),
        Ask::Bye => (0, Err(Why::Internal)),
    };
    out.unwrap_or_else(|why| Reply::Failed { id, why })
}

/// The most bytes a DIB may hold, as asked; the frame keeps it within `MAX_IMAGE_BYTES`.
fn cap(max_bytes: u32) -> usize {
    usize::try_from(max_bytes).unwrap_or(usize::MAX)
}

/// The reply carrying draw `id`'s pixels.
fn pixels(id: u32, p: Picture) -> Reply {
    Reply::Pixels {
        id,
        width: p.width,
        height: p.height,
        rgba: p.rgba,
    }
}

/// Pixels drawn for one ask: RGBA8, row by row from the top.
#[derive(Debug, PartialEq, Eq)]
pub struct Picture {
    /// Pixels in a row.
    pub width: u32,
    /// Rows.
    pub height: u32,
    /// `width` × `height` × `RGBA_BYTES` bytes.
    pub rgba: Vec<u8>,
}

/// Draws `bytes`, coded as `codec`, within `side` on its longest side.
///
/// # Errors
/// `TooBig` past `max_pixels` pixels, before any pixel is decoded; `Refused` for bytes that are not an image read here.
pub fn draw(codec: Codec, bytes: &[u8], side: u32, max_pixels: u64) -> Result<Picture, Why> {
    match codec {
        Codec::Png => png::draw(bytes, side, max_pixels),
        Codec::Dib => dib::draw(bytes, side, max_pixels),
    }
}

/// Makes the `CF_DIB` of `png`, at most `max_bytes` long.
///
/// # Errors
/// `TooBig` past `max_pixels` or `max_bytes`, before any row is decoded; `Refused` for a damaged, cut or interlaced PNG.
pub fn make_dib(png: &[u8], max_bytes: usize, max_pixels: u64) -> Result<Vec<u8>, Why> {
    bitmap::make(png, max_bytes, max_pixels)
}

/// Refuses an image of `width` by `height` that is empty or holds more than `max_pixels` pixels.
fn check_size(width: u32, height: u32, max_pixels: u64) -> Result<(), Why> {
    if width == 0 || height == 0 {
        Err(Why::Refused)
    } else if u64::from(width) * u64::from(height) > max_pixels {
        Err(Why::TooBig)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests;
