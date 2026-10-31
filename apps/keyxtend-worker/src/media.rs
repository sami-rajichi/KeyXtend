//! The media job (ADR-0034 decision 5): PNG and DIB bytes in, RGBA8 pixels no larger than the asked side out. Rows
//! are scaled as they are read, so no full-size copy of an image is ever held.

mod dib;
mod png;
mod scale;

use kx_ipc::{Codec, Why};

/// What the PNG decoder may allocate for its rows and chunks: 16 MiB, rows up to 4 million pixels wide.
const DECODE_BYTES: usize = 16 * 1024 * 1024;

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
