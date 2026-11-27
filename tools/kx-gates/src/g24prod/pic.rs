//! A picture's pixels as blue, green and red, top row first, placed on the screen: from the clipboard's DIB or a screen
//! grab, compared by the share of pixels that differ. Alpha is never compared, as a grab's is not reliable.

use spike_core::capture::Shot;
use windows::Win32::Foundation::RECT;

/// Bytes of the smallest DIB header, `BITMAPINFOHEADER`.
const HEADER: usize = 40;
/// Where that header holds its own size, the width, the height (negative when top-down), the bits a pixel and the
/// compression.
const SIZE_AT: usize = 0;
const WIDTH_AT: usize = 4;
const HEIGHT_AT: usize = 8;
const BITS_AT: usize = 14;
const KIND_AT: usize = 16;
/// Bytes of the colour masks after that header in a `BI_BITFIELDS` DIB.
const MASKS: usize = 12;
/// `BI_RGB`: plain pixels.
const PLAIN: u32 = 0;
/// `BI_BITFIELDS`: plain pixels after their masks.
const FIELDS: u32 = 3;
/// Bytes a pixel takes here.
const BGR: usize = 3;
/// Bytes a grab's pixel takes.
pub(super) const BGRA: usize = 4;
/// Bits in a byte.
pub(super) const BITS: usize = 8;
/// A box's part, each side, outside its middle half: a quarter.
const QUARTER: i32 = 4;
/// A DIB row's alignment, in bytes.
const ROW_ALIGN: usize = 4;

/// A picture placed with its top-left at `left`, `top` in screen pixels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pic {
    pub left: i32,
    pub top: i32,
    width: usize,
    height: usize,
    bgr: Vec<u8>,
}

/// The little-endian `u32` at `at` in `b`.
pub(super) fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        b.get(at..at + size_of::<u32>())?.try_into().ok()?,
    ))
}

/// The little-endian `u16` at `at` in `b`.
fn u16_at(b: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes(
        b.get(at..at + size_of::<u16>())?.try_into().ok()?,
    ))
}

/// What a DIB's header says: its size, whether its rows run bottom-up, its bytes a pixel and where its pixels start.
struct Head {
    width: usize,
    height: usize,
    up: bool,
    bytes: usize,
    first: usize,
}

/// The header of DIB `b`, refused unless it holds plain 24 or 32-bit pixels.
fn head(b: &[u8]) -> Option<Result<Head, String>> {
    let own = usize::try_from(u32_at(b, SIZE_AT)?).ok()?;
    let [width, height] = [WIDTH_AT, HEIGHT_AT].map(|at| u32_at(b, at).map(u32::cast_signed));
    let (width, height) = (width?, height?);
    let bits = usize::from(u16_at(b, BITS_AT)?);
    let kind = u32_at(b, KIND_AT)?;
    let plain = kind == PLAIN || kind == FIELDS;
    if own < HEADER || !plain || !(bits == BGR * BITS || bits == BGRA * BITS) {
        return Some(Err(format!(
            "a DIB of {bits} bits, kind {kind}: only plain 24 or 32 bits are read"
        )));
    }
    let side = |v: i32| usize::try_from(v.unsigned_abs()).unwrap_or(0);
    let masks = if kind == FIELDS && own == HEADER {
        MASKS
    } else {
        0
    };
    Some(Ok(Head {
        width: side(width),
        height: side(height),
        up: height > 0,
        bytes: bits / BITS,
        first: own + masks,
    }))
}

impl Pic {
    /// The picture in a packed DIB of 24 or 32 bits a pixel, bottom-up or top-down, placed at 0, 0.
    pub fn from_dib(b: &[u8]) -> Result<Self, String> {
        let short = || "the DIB is too short".to_string();
        let h = head(b).ok_or_else(short)??;
        if h.width == 0 || h.height == 0 {
            return Err("an empty DIB".into());
        }
        let stride = (h.width * h.bytes * BITS).div_ceil(BITS * ROW_ALIGN) * ROW_ALIGN;
        let end = stride
            .checked_mul(h.height)
            .and_then(|n| n.checked_add(h.first));
        let rows = b.get(h.first..end.ok_or_else(short)?).ok_or_else(short)?;
        let mut bgr = Vec::with_capacity(h.width * h.height * BGR);
        for y in 0..h.height {
            let line = &rows[if h.up { h.height - 1 - y } else { y } * stride..];
            (0..h.width).for_each(|x| bgr.extend_from_slice(&line[x * h.bytes..x * h.bytes + BGR]));
        }
        Ok(Self {
            left: 0,
            top: 0,
            width: h.width,
            height: h.height,
            bgr,
        })
    }

    /// A screen grab's pixels at its place, without alpha.
    pub fn from_shot(s: &Shot) -> Self {
        let (pixels, _) = s.bgra.as_chunks::<BGRA>();
        let bgr = pixels
            .iter()
            .flat_map(|p| p[..BGR].iter().copied())
            .collect();
        let side = |v: i32| usize::try_from(v).unwrap_or(0);
        Self {
            left: s.left,
            top: s.top,
            width: side(s.width),
            height: side(s.height),
            bgr,
        }
    }

    /// The same picture with its top-left at `left`, `top`.
    pub fn placed(self, left: i32, top: i32) -> Self {
        Self { left, top, ..self }
    }

    /// Its width and height in pixels.
    pub fn size(&self) -> (i32, i32) {
        let side = |v: usize| i32::try_from(v).unwrap_or(i32::MAX);
        (side(self.width), side(self.height))
    }

    /// The pixel at screen point `x`, `y`; none outside the picture.
    pub fn at(&self, x: i32, y: i32) -> Option<[u8; 3]> {
        let col = usize::try_from(x.checked_sub(self.left)?)
            .ok()
            .filter(|&c| c < self.width)?;
        let row = usize::try_from(y.checked_sub(self.top)?)
            .ok()
            .filter(|&r| r < self.height)?;
        let i = (row * self.width + col) * BGR;
        self.bgr.get(i..i + BGR)?.try_into().ok()
    }

    /// True when every pixel is one colour, as in a blank drawing.
    pub fn uniform(&self) -> bool {
        self.bgr
            .as_chunks::<BGR>()
            .0
            .windows(2)
            .all(|w| w[0] == w[1])
    }
}

/// The middle half of box `r` each way, which a round window fills.
pub fn middle(r: &RECT) -> RECT {
    let (w, h) = ((r.right - r.left) / QUARTER, (r.bottom - r.top) / QUARTER);
    RECT {
        left: r.left + w,
        top: r.top + h,
        right: r.right - w,
        bottom: r.bottom - h,
    }
}

/// The share of the pixels in `r` that both `a` and `b` hold and that differ; none when they share no pixel there.
#[allow(
    clippy::cast_precision_loss,
    reason = "A count of a screen's pixels, far below 2^52."
)]
pub fn differing(a: &Pic, b: &Pic, r: &RECT) -> Option<f64> {
    let (mut both, mut differ) = (0u64, 0u64);
    for y in r.top..r.bottom {
        for x in r.left..r.right {
            if let (Some(p), Some(q)) = (a.at(x, y), b.at(x, y)) {
                both += 1;
                differ += u64::from(p != q);
            }
        }
    }
    (both > 0).then(|| differ as f64 / both as f64)
}

#[cfg(test)]
mod tests;
