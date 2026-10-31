//! PNG through the `png` crate: the header is checked first, then rows go one at a time into the scaler, with the
//! decoder's memory capped and its text and colour-profile chunks skipped.

use super::scale::{Rgba, Scaler};
use super::{DECODE_BYTES, Picture, check_size};
use ::png::{BitDepth, ColorType, Decoder, DecodingError, Limits, Transformations};
use kx_ipc::Why;
use std::io::Cursor;

/// Every colour type and depth comes out as 8-bit grey with alpha, or RGBA.
const TRANSFORMS: Transformations = Transformations::EXPAND
    .union(Transformations::ALPHA)
    .union(Transformations::STRIP_16);
/// Bytes in one 8-bit grey pixel with alpha.
const GREY_ALPHA: usize = 2;

/// Draws a PNG within `side`.
///
/// # Errors
/// `TooBig` past `max_pixels` or the decoder's memory cap; `Refused` for a damaged, cut or interlaced PNG.
pub fn draw(bytes: &[u8], side: u32, max_pixels: u64) -> Result<Picture, Why> {
    let mut decoder = Decoder::new_with_limits(
        Cursor::new(bytes),
        Limits {
            bytes: DECODE_BYTES,
        },
    );
    decoder.set_transformations(TRANSFORMS);
    decoder.set_ignore_text_chunk(true);
    decoder.set_ignore_iccp_chunk(true);
    let head = decoder.read_header_info().map_err(|e| why(&e))?;
    let (width, height, interlaced) = (head.width, head.height, head.interlaced);
    check_size(width, height, max_pixels)?;
    // Interlaced rows come in seven passes over the whole image, which would need a full-size buffer.
    if interlaced {
        return Err(Why::Refused);
    }
    let mut reader = decoder.read_info().map_err(|e| why(&e))?;
    let step = match reader.output_color_type() {
        (ColorType::Rgba, BitDepth::Eight) => size_of::<Rgba>(),
        (ColorType::GrayscaleAlpha, BitDepth::Eight) => GREY_ALPHA,
        _ => return Err(Why::Internal),
    };
    let mut scaler = Scaler::new(width, height, side)?;
    while let Some(row) = reader.next_row().map_err(|e| why(&e))? {
        scaler.row(row.data().chunks_exact(step).map(rgba))?;
    }
    scaler.finish()
}

/// One decoded pixel, grey with alpha or RGBA, as RGBA.
fn rgba(pixel: &[u8]) -> Rgba {
    match *pixel {
        [grey, alpha] => [grey, grey, grey, alpha],
        [r, g, b, a] => [r, g, b, a],
        _ => Rgba::default(),
    }
}

/// The fixed reason for a decoder error: too big past its memory cap, internal for misuse, else refused.
fn why(e: &DecodingError) -> Why {
    match e {
        DecodingError::LimitsExceeded => Why::TooBig,
        DecodingError::Parameter(_) => Why::Internal,
        DecodingError::IoError(_) | DecodingError::Format(_) => Why::Refused,
    }
}
