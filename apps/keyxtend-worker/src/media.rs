//! The media job (ADR-0034 decision 5): PNG and DIB bytes in, RGBA8 pixels no larger than the asked side out, rows
//! scaled as they are read; and a PNG made into a `CF_DIB` for an insert, within the size the host asks.

mod bitmap;
mod dib;
mod png;
mod scale;

use kx_ipc::{Codec, Why};

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
