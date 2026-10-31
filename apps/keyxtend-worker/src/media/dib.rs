//! Windows DIBs as `CF_DIB` and `CF_DIBV5` hold them: a header, optional masks and colours, then rows read in place
//! into the scaler. Every size is checked against the buffer, with checked sums, before any pixel is read.

use super::scale::{Rgba, Scaler};
use super::{Picture, check_size};
use kx_ipc::{RGBA_BYTES, Why};

/// Header sizes read: `BITMAPINFOHEADER`, `BITMAPV4HEADER` and `BITMAPV5HEADER`.
pub(super) const INFO: usize = 40;
const V4: usize = 108;
const V5: usize = 124;
/// Field offsets in every header.
const WIDTH: usize = 4;
const HEIGHT: usize = 8;
const PLANES: usize = 12;
const BITS: usize = 14;
const COMPRESSION: usize = 16;
const COLOURS: usize = 32;
/// Where the red, green, blue and alpha masks sit inside a V4 or V5 header.
const MASKS: usize = 40;
/// Bits per pixel read: plain 24-bit rows, and 32-bit rows plain or with masks.
const BITS_RGB: u16 = 24;
pub(super) const BITS_RGBA: u16 = 32;
/// Compression codes read: plain rows, and rows with bit masks.
pub(super) const BI_RGB: u32 = 0;
const BI_BITFIELDS: u32 = 3;
/// Bytes in a mask or a colour-table entry.
const DWORD: usize = size_of::<u32>();
/// The red, green and blue masks after a plain header.
const RGB_MASKS: usize = 3 * DWORD;
/// Rows are padded to a multiple of this many bytes.
const ROW_ALIGN: u64 = 4;
/// A DIB's plane count, which is always one.
pub(super) const ONE_PLANE: u16 = 1;

/// Where each channel sits in a stored pixel.
#[derive(Copy, Clone)]
struct Layout {
    /// Bytes per stored pixel.
    bytes: usize,
    /// The byte holding red, green and blue.
    rgb: [usize; 3],
    /// The byte holding alpha, if any.
    alpha: Option<usize>,
}

/// Plain 24-bit rows: blue, green, red.
const BGR: Layout = Layout {
    bytes: 3,
    rgb: [2, 1, 0],
    alpha: None,
};
/// Plain 32-bit rows: blue, green, red, then alpha or nothing.
const BGRA: Layout = Layout {
    bytes: RGBA_BYTES,
    rgb: [2, 1, 0],
    alpha: Some(3),
};

impl Layout {
    /// The stored `pixel` as RGBA; with no alpha it is opaque.
    fn rgba(self, pixel: &[u8]) -> Rgba {
        let byte = |i: usize| pixel.get(i).copied().unwrap_or_default();
        let [r, g, b] = self.rgb.map(byte);
        [r, g, b, self.alpha.map_or(u8::MAX, byte)]
    }
}

/// A checked DIB: its size, its layout and where its rows lie in the buffer.
struct Dib {
    width: u32,
    height: u32,
    top_down: bool,
    layout: Layout,
    /// Where the first stored row starts.
    offset: usize,
    /// Bytes from one stored row to the next.
    stride: usize,
}

/// Draws a DIB within `side`.
///
/// # Errors
/// `TooBig` past `max_pixels`; `Refused` for any other header, depth or compression, or one that runs past `bytes`.
pub fn draw(bytes: &[u8], side: u32, max_pixels: u64) -> Result<Picture, Why> {
    let mut dib = Dib::read(bytes, max_pixels)?;
    // A 32-bit image whose alpha is all zero is opaque: many writers leave that byte unused.
    if let Some(a) = dib.layout.alpha
        && dib.rows(bytes).all(|row| {
            row.chunks_exact(dib.layout.bytes)
                .all(|p| p.get(a) == Some(&0))
        })
    {
        dib.layout.alpha = None;
    }
    let mut scaler = Scaler::new(dib.width, dib.height, side)?;
    let layout = dib.layout;
    for row in dib.rows(bytes) {
        scaler.row(row.chunks_exact(layout.bytes).map(|p| layout.rgba(p)))?;
    }
    scaler.finish()
}

impl Dib {
    /// Reads and checks the header, the masks and where the rows lie, before any pixel is read.
    fn read(bytes: &[u8], max_pixels: u64) -> Result<Self, Why> {
        let size = usize::try_from(u32_at(bytes, 0)?).map_err(|_| Why::Refused)?;
        if ![INFO, V4, V5].contains(&size) || bytes.len() < size {
            return Err(Why::Refused);
        }
        let width = u32::try_from(i32_at(bytes, WIDTH)?).map_err(|_| Why::Refused)?;
        let height = i32_at(bytes, HEIGHT)?;
        check_size(width, height.unsigned_abs(), max_pixels)?;
        if u16_at(bytes, PLANES)? != ONE_PLANE {
            return Err(Why::Refused);
        }
        let (layout, masked) = layout(bytes, size)?;
        let offset = offset(bytes, size, masked)?;
        let stride = (u64::from(width) * layout.bytes as u64).next_multiple_of(ROW_ALIGN);
        let end = stride
            .checked_mul(u64::from(height.unsigned_abs()))
            .and_then(|area| area.checked_add(offset as u64))
            .ok_or(Why::Refused)?;
        if end > bytes.len() as u64 {
            return Err(Why::Refused);
        }
        Ok(Self {
            width,
            height: height.unsigned_abs(),
            top_down: height < 0,
            layout,
            offset,
            stride: usize::try_from(stride).map_err(|_| Why::Refused)?,
        })
    }

    /// The stored rows from the top, each cut to its pixels; the header check keeps every row in `bytes`.
    fn rows<'a>(&self, bytes: &'a [u8]) -> impl Iterator<Item = &'a [u8]> + use<'a> {
        let (offset, stride, height, top_down) =
            (self.offset, self.stride, self.height, self.top_down);
        let len = self.width as usize * self.layout.bytes;
        (0..height).filter_map(move |y| {
            let stored = if top_down { y } else { height - 1 - y };
            let start = offset + stored as usize * stride;
            bytes.get(start..start + len)
        })
    }
}

/// The layout the depth and compression name, and whether it came from masks.
fn layout(bytes: &[u8], size: usize) -> Result<(Layout, bool), Why> {
    match (u16_at(bytes, BITS)?, u32_at(bytes, COMPRESSION)?) {
        (BITS_RGB, BI_RGB) => Ok((BGR, false)),
        (BITS_RGBA, BI_RGB) => Ok((BGRA, false)),
        (BITS_RGBA, BI_BITFIELDS) => masks(bytes, size).map(|layout| (layout, true)),
        _ => Err(Why::Refused),
    }
}

/// A 32-bit layout from its masks: after a plain header, or inside a V4 or V5 one with an alpha mask too.
fn masks(bytes: &[u8], size: usize) -> Result<Layout, Why> {
    let at = if size == INFO { INFO } else { MASKS };
    let mask = |i: usize| u32_at(bytes, at + i * DWORD);
    let rgb = [byte_of(mask(0)?)?, byte_of(mask(1)?)?, byte_of(mask(2)?)?];
    let alpha = match if size == INFO { 0 } else { mask(3)? } {
        0 => None,
        m => Some(byte_of(m)?),
    };
    let mut used: Vec<usize> = rgb.iter().copied().chain(alpha).collect();
    used.sort_unstable();
    used.dedup();
    if used.len() != rgb.len() + usize::from(alpha.is_some()) {
        return Err(Why::Refused);
    }
    Ok(Layout {
        bytes: RGBA_BYTES,
        rgb,
        alpha,
    })
}

/// The byte a mask covers whole, or `Refused` for any other mask.
fn byte_of(mask: u32) -> Result<usize, Why> {
    (0..RGBA_BYTES)
        .zip((0..).step_by(u8::BITS as usize))
        .find_map(|(byte, shift)| (mask == u32::from(u8::MAX) << shift).then_some(byte))
        .ok_or(Why::Refused)
}

/// Where the rows start: after the header, the masks and the colour table.
fn offset(bytes: &[u8], size: usize, masked: bool) -> Result<usize, Why> {
    // Some writers repeat the three masks after a V4 or V5 header; they are skipped when they match.
    let repeated =
        size != INFO && bytes.get(MASKS..MASKS + RGB_MASKS) == bytes.get(size..size + RGB_MASKS);
    let masks = if masked && (size == INFO || repeated) {
        RGB_MASKS
    } else {
        0
    };
    let table = usize::try_from(u32_at(bytes, COLOURS)?)
        .ok()
        .and_then(|n| n.checked_mul(DWORD))
        .ok_or(Why::Refused)?;
    size.checked_add(masks)
        .and_then(|n| n.checked_add(table))
        .ok_or(Why::Refused)
}

/// The little-endian field of `N` bytes at `at`, or `Refused` past the buffer.
fn field<const N: usize>(bytes: &[u8], at: usize) -> Result<[u8; N], Why> {
    bytes
        .get(at..at + N)
        .and_then(|b| b.try_into().ok())
        .ok_or(Why::Refused)
}

/// The little-endian `u16` at `at`.
fn u16_at(bytes: &[u8], at: usize) -> Result<u16, Why> {
    field(bytes, at).map(u16::from_le_bytes)
}

/// The little-endian `u32` at `at`.
fn u32_at(bytes: &[u8], at: usize) -> Result<u32, Why> {
    field(bytes, at).map(u32::from_le_bytes)
}

/// The little-endian `i32` at `at`.
fn i32_at(bytes: &[u8], at: usize) -> Result<i32, Why> {
    field(bytes, at).map(i32::from_le_bytes)
}
