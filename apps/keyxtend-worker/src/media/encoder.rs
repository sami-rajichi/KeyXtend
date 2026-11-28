//! A snip's raw pixels made into a PNG (ADR-0056 decision 2): 8-bit RGB rows, each pixel's unused byte dropped, one
//! row at a time into a buffer reserved once at the PNG's cap, so its memory is known before the first row.

use super::{Raw, check_size};
use ::png::{BitDepth, ColorType, DeflateCompression, Encoder, EncodingError};
use kx_ipc::{BGRX_BYTES, Why, png_most};
use std::io::{self, Write};

/// `flate2`'s deflate at its fastest level: unlike the faster `fdeflate`, it falls back to stored blocks, so a PNG of
/// noise stays within `png_most`.
const DEFLATE: DeflateCompression = DeflateCompression::Level(1);
/// The PNG's pixels: 8-bit RGB, each pixel's unused byte dropped.
pub(super) const COLOR: ColorType = ColorType::Rgb;
/// The filter type byte that starts each PNG row.
pub(super) const FILTER_BYTES: usize = 1;

/// The most bytes the PNG of a `width` by `height` picture takes: `png_most` of its rows as deflate stores them whole.
/// `None` past this system's sizes.
pub fn most(width: u32, height: u32) -> Option<usize> {
    let row = size(width).ok()?.checked_mul(COLOR.samples())?;
    let rows = row
        .checked_add(FILTER_BYTES)?
        .checked_mul(size(height).ok()?)?;
    Some(png_most(rows))
}

/// Makes the PNG of `raw`, at most `most` bytes long; its buffer and our row's are reserved before the first row.
///
/// # Errors
/// `TooBig` past `max_pixels` before any row is read, when the buffers' memory cannot be had, or for a PNG past `most`;
/// `Refused` for an empty picture or rows shorter than the stride; `Internal` when the encoder fails otherwise.
pub fn make(raw: &Raw<'_>, most: usize, max_pixels: u64) -> Result<Vec<u8>, Why> {
    check_size(raw.width, raw.height, max_pixels)?;
    let (width, height, stride) = (size(raw.width)?, size(raw.height)?, size(raw.stride)?);
    let row = width.checked_mul(BGRX_BYTES).ok_or(Why::TooBig)?;
    let len = stride.checked_mul(height).ok_or(Why::TooBig)?;
    if row > stride || raw.bgrx.len() < len {
        return Err(Why::Refused);
    }
    let mut rgb = Vec::new();
    let rgb_row = width.checked_mul(COLOR.samples()).ok_or(Why::TooBig)?;
    rgb.try_reserve_exact(rgb_row).map_err(|_| Why::TooBig)?;
    let mut out = Capped::new(most)?;
    let rows = raw.bgrx.chunks_exact(stride).take(height);
    let rows = rows.map(|line| line.get(..row).unwrap_or_default());
    match write(&mut out, raw, rows, &mut rgb) {
        Ok(()) => Ok(out.bytes),
        Err(_) if out.over => Err(Why::TooBig),
        Err(_) => Err(Why::Internal),
    }
}

/// Writes `raw`'s header, then each of `rows`' pixels as RGB through `rgb`, into `out`.
fn write<'a>(
    out: &mut Capped,
    raw: &Raw<'_>,
    rows: impl Iterator<Item = &'a [u8]>,
    rgb: &mut Vec<u8>,
) -> Result<(), EncodingError> {
    let mut encoder = Encoder::new(out, raw.width, raw.height);
    encoder.set_color(COLOR);
    encoder.set_depth(BitDepth::Eight);
    encoder.set_deflate_compression(DEFLATE);
    let mut writer = encoder.write_header()?;
    let mut stream = writer.stream_writer()?;
    for pixels in rows {
        rgb.clear();
        let (each, _) = pixels.as_chunks::<BGRX_BYTES>();
        rgb.extend(each.iter().flat_map(|&[b, g, r, _]| [r, g, b]));
        stream.write_all(rgb)?;
    }
    stream.finish()?;
    writer.finish()
}

/// `n` as a size.
fn size(n: u32) -> Result<usize, Why> {
    usize::try_from(n).map_err(|_| Why::TooBig)
}

/// A PNG's bytes, reserved once at `most`; a write past it fails and is marked.
struct Capped {
    bytes: Vec<u8>,
    most: usize,
    over: bool,
}

impl Capped {
    /// An empty buffer holding `most` bytes.
    ///
    /// # Errors
    /// `TooBig` when that memory cannot be had.
    fn new(most: usize) -> Result<Self, Why> {
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(most).map_err(|_| Why::TooBig)?;
        Ok(Self {
            bytes,
            most,
            over: false,
        })
    }
}

impl Write for Capped {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.bytes.len().saturating_add(buf.len()) > self.most {
            self.over = true;
            return Err(io::ErrorKind::FileTooLarge.into());
        }
        self.bytes.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
