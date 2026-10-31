//! A box filter that shrinks an image as its rows arrive, top first, holding only one row of sums and the small
//! output, so a large image stays within the worker's memory. Colours are averaged by alpha, so clear pixels do not
//! darken their neighbours.

use super::Picture;
use kx_ipc::{MAX_SIDE, RGBA_BYTES, Why};

/// One pixel: red, green, blue and alpha.
pub type Rgba = [u8; RGBA_BYTES];
/// One output pixel's sums: red, green and blue each times alpha, then alpha.
type Sums = [u64; RGBA_BYTES];

/// The size `width` by `height` shrinks to so its longest side is at most `side`, and never past `MAX_SIDE`; a
/// smaller image keeps its size.
pub fn fit(width: u32, height: u32, side: u32) -> (u32, u32) {
    let long = u64::from(width.max(height));
    let side = u64::from(side.clamp(1, MAX_SIDE));
    if long <= side {
        return (width, height);
    }
    let scale = |n: u32| {
        let n = (u64::from(n) * side + long / 2) / long;
        u32::try_from(n.max(1)).unwrap_or(u32::MAX)
    };
    (scale(width), scale(height))
}

/// One axis of the filter: `src` input pixels shrink to `out` output pixels, with `out` at most `src`.
#[derive(Copy, Clone)]
struct Axis {
    src: u32,
    out: u32,
}

impl Axis {
    /// The first input pixel of output pixel `k`: k × src / out, rounded up.
    fn edge(self, k: u32) -> u64 {
        (u64::from(k) * u64::from(self.src)).div_ceil(u64::from(self.out))
    }

    /// How many input pixels output pixel `k` covers, at least one.
    fn span(self, k: u32) -> u64 {
        self.edge(k + 1) - self.edge(k)
    }
}

/// Shrinks an image fed one row at a time, from the top.
pub struct Scaler {
    x: Axis,
    y: Axis,
    /// The sums of the output row being filled, one per output pixel.
    sums: Vec<Sums>,
    /// The output row being filled.
    oy: u32,
    /// Input rows added so far.
    rows: u64,
    /// The finished output rows.
    rgba: Vec<u8>,
}

impl Scaler {
    /// A scaler for a `width` by `height` image, drawn within `side`.
    ///
    /// # Errors
    /// `Refused` for an empty image; `TooBig` for an output past memory.
    pub fn new(width: u32, height: u32, side: u32) -> Result<Self, Why> {
        if width == 0 || height == 0 {
            return Err(Why::Refused);
        }
        let (out_w, out_h) = fit(width, height, side);
        let row = usize::try_from(out_w).map_err(|_| Why::TooBig)?;
        let len = usize::try_from(out_h)
            .ok()
            .and_then(|h| h.checked_mul(row)?.checked_mul(RGBA_BYTES))
            .ok_or(Why::TooBig)?;
        Ok(Self {
            x: Axis {
                src: width,
                out: out_w,
            },
            y: Axis {
                src: height,
                out: out_h,
            },
            sums: vec![[0; RGBA_BYTES]; row],
            oy: 0,
            rows: 0,
            rgba: Vec::with_capacity(len),
        })
    }

    /// Adds the next input row, which must hold exactly the image's width in pixels.
    ///
    /// # Errors
    /// `Refused` for a row too short or too long, or one past the image's height.
    pub fn row(&mut self, pixels: impl IntoIterator<Item = Rgba>) -> Result<(), Why> {
        if self.rows >= u64::from(self.y.src) {
            return Err(Why::Refused);
        }
        let mut pixels = pixels.into_iter();
        for (ox, sums) in (0..).zip(self.sums.iter_mut()) {
            for _ in 0..self.x.span(ox) {
                add(sums, pixels.next().ok_or(Why::Refused)?);
            }
        }
        if pixels.next().is_some() {
            return Err(Why::Refused);
        }
        self.rows += 1;
        if self.rows == self.y.edge(self.oy + 1) {
            self.flush();
        }
        Ok(())
    }

    /// The output, once every input row has arrived.
    ///
    /// # Errors
    /// `Refused` when rows are missing.
    pub fn finish(self) -> Result<Picture, Why> {
        if self.rows != u64::from(self.y.src) {
            return Err(Why::Refused);
        }
        Ok(Picture {
            width: self.x.out,
            height: self.y.out,
            rgba: self.rgba,
        })
    }

    /// Turns the sums into the next output row, then clears them.
    fn flush(&mut self) {
        let rows = self.y.span(self.oy);
        for (ox, sums) in (0..).zip(self.sums.iter_mut()) {
            self.rgba.extend(mean(sums, self.x.span(ox) * rows));
            *sums = [0; RGBA_BYTES];
        }
        self.oy += 1;
    }

    /// The bytes this scaler holds, for the tests' memory checks.
    #[cfg(test)]
    pub fn held(&self) -> usize {
        self.rgba.capacity() + self.sums.capacity() * size_of::<Sums>()
    }
}

/// Adds one pixel to its output pixel's sums, its colour weighted by its alpha.
fn add(sums: &mut Sums, [r, g, b, a]: Rgba) {
    let a = u64::from(a);
    let [sr, sg, sb, sa] = sums;
    for (sum, c) in [sr, sg, sb].into_iter().zip([r, g, b]) {
        *sum = sum.saturating_add(u64::from(c) * a);
    }
    *sa = sa.saturating_add(a);
}

/// The pixel of a box of `count` input pixels: alpha is their mean, each colour its alpha-weighted mean.
fn mean(&[sr, sg, sb, sa]: &Sums, count: u64) -> Rgba {
    let colour = |sum: u64| if sa == 0 { 0 } else { div_round(sum, sa) };
    [colour(sr), colour(sg), colour(sb), div_round(sa, count)]
}

/// `n` / `d`, rounded half up, as a byte; every mean here is at most 255.
fn div_round(n: u64, d: u64) -> u8 {
    u8::try_from(n.saturating_add(d / 2) / d).unwrap_or(u8::MAX)
}
