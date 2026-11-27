//! A picture's `CF_DIB`, built from its own raw pixels so the app never decodes one (ADR-0056 decision 2): a
//! `BITMAPINFOHEADER`, then 24-bit rows from the bottom, each padded to four bytes, which every app reads as opaque.

use kx_module_api::Redacted;
use kx_module_api::ports::{BGRX_BYTES, Shot};

/// The header's size, which also names its kind.
const HEADER_BYTES: u32 = 40;
/// A DIB has one colour plane.
const PLANES: u16 = 1;
/// Bits per pixel: blue, green and red.
const BITS: u16 = 24;
/// The bytes a 24-bit pixel takes.
const PIXEL_BYTES: usize = 3;
/// `BI_RGB`: rows stored as they are.
const UNCOMPRESSED: u32 = 0;
/// Each row starts at a multiple of this many bytes.
const ROW_ALIGN: usize = 4;
/// The header's last fields, all 0: the two resolutions, the palette's size and its important colours.
const ZERO_FIELDS: usize = 4;

/// The bytes of one row of `width` pixels, padded.
fn stride(width: usize) -> usize {
    (width * PIXEL_BYTES).next_multiple_of(ROW_ALIGN)
}

/// The `BITMAPINFOHEADER` of a `width` by `height` picture whose rows take `image` bytes, little-endian.
fn header(width: u32, height: u32, image: usize) -> Vec<u8> {
    let side = |n: u32| i32::try_from(n).unwrap_or(i32::MAX).to_le_bytes();
    let mut out = Vec::with_capacity(usize::try_from(HEADER_BYTES).unwrap_or_default());
    out.extend(HEADER_BYTES.to_le_bytes());
    out.extend(side(width));
    // A height above 0 stores the rows from the bottom up.
    out.extend(side(height));
    out.extend(PLANES.to_le_bytes());
    out.extend(BITS.to_le_bytes());
    out.extend(UNCOMPRESSED.to_le_bytes());
    out.extend(u32::try_from(image).unwrap_or(u32::MAX).to_le_bytes());
    out.extend([0; ZERO_FIELDS * size_of::<u32>()]);
    out
}

/// `shot`'s `CF_DIB`.
pub(super) fn of(shot: &Shot) -> Redacted<Vec<u8>> {
    let (width, height) = (shot.width(), shot.height());
    let wide = usize::try_from(width).unwrap_or_default();
    let (row, pad) = (wide * BGRX_BYTES, stride(wide) - wide * PIXEL_BYTES);
    let rows = shot.bgrx().expose().chunks_exact(row.max(1));
    let image = stride(wide) * rows.len();
    let mut out = header(width, height, image);
    out.reserve(image);
    for line in rows.rev() {
        for pixel in line.as_chunks::<BGRX_BYTES>().0 {
            out.extend(&pixel[..PIXEL_BYTES]);
        }
        out.extend(std::iter::repeat_n(0, pad));
    }
    Redacted::new(out)
}

#[cfg(test)]
mod tests;
