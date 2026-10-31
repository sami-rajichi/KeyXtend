//! A PNG made into a `CF_DIB` for an insert: a `BITMAPINFOHEADER`, then 32-bit rows from the bottom up, each pixel
//! blue, green, red and alpha, the alpha kept as browsers write it. Its size is checked against the cap first.

use super::dib::{BI_RGB, BITS_RGBA, INFO, ONE_PLANE};
use super::png::{Rows, open, rgba, rows, why};
use kx_ipc::{RGBA_BYTES, Why};

/// Makes the `CF_DIB` of `png`, at most `max_bytes` long, decoding one row at a time into its place.
///
/// # Errors
/// `TooBig` past `max_pixels` or `max_bytes`, before any row is decoded, or when its memory cannot be had; `Refused`
/// for a damaged, cut or interlaced PNG.
pub fn make(png: &[u8], max_bytes: usize, max_pixels: u64) -> Result<Vec<u8>, Why> {
    let (decoder, width, height) = open(png, max_pixels)?;
    let (stride, area) = sizes(width, height).ok_or(Why::TooBig)?;
    let len = area
        .checked_add(INFO)
        .filter(|&n| n <= max_bytes)
        .ok_or(Why::TooBig)?;
    let mut out = Vec::new();
    out.try_reserve_exact(len).map_err(|_| Why::TooBig)?;
    out.extend_from_slice(&header(width, height, area)?);
    out.resize(len, 0);
    let (reader, step) = rows(decoder)?;
    fill(
        reader,
        step,
        out.get_mut(INFO..).unwrap_or_default(),
        stride,
    )?;
    Ok(out)
}

/// The bytes of one stored row and of all of them, for `width` by `height` 32-bit pixels; `None` past memory.
fn sizes(width: u32, height: u32) -> Option<(usize, usize)> {
    let stride = usize::try_from(width).ok()?.checked_mul(RGBA_BYTES)?;
    let area = stride.checked_mul(usize::try_from(height).ok()?)?;
    Some((stride, area))
}

/// Decodes each row from `reader`, its pixels `step` bytes each, into its place in `area`, from the bottom up.
fn fill(mut reader: Rows<'_>, step: usize, area: &mut [u8], stride: usize) -> Result<(), Why> {
    let mut places = area.chunks_exact_mut(stride).rev();
    while let Some(row) = reader.next_row().map_err(|e| why(&e))? {
        let place = places.next().ok_or(Why::Refused)?;
        let pixels = row.data().chunks_exact(step);
        if pixels.len() * RGBA_BYTES != place.len() {
            return Err(Why::Refused);
        }
        for (to, from) in place.as_chunks_mut::<RGBA_BYTES>().0.iter_mut().zip(pixels) {
            let [r, g, b, a] = rgba(from);
            *to = [b, g, r, a];
        }
    }
    // A PNG that ends early leaves rows unfilled.
    places.next().map_or(Ok(()), |_| Err(Why::Refused))
}

/// The header's fields up to the image size, for `width` by `height` rows of `area` bytes stored from the bottom up;
/// the four fields after it stay zero.
fn header(width: u32, height: u32, area: usize) -> Result<Vec<u8>, Why> {
    let signed = |n: u32| i32::try_from(n).map_err(|_| Why::TooBig);
    let size = u32::try_from(INFO).map_err(|_| Why::Internal)?;
    let area = u32::try_from(area).map_err(|_| Why::TooBig)?;
    Ok([
        &size.to_le_bytes()[..],
        &signed(width)?.to_le_bytes(),
        &signed(height)?.to_le_bytes(),
        &ONE_PLANE.to_le_bytes(),
        &BITS_RGBA.to_le_bytes(),
        &BI_RGB.to_le_bytes(),
        &area.to_le_bytes(),
    ]
    .concat())
}
