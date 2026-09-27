//! Screen copies in physical pixels: a frozen copy of the whole desktop, a region of it, and BMP files.

use windows::Win32::Foundation::RECT;
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BitBlt, CAPTUREBLT, CreateCompatibleBitmap,
    CreateCompatibleDC, DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDC, GetDIBits, ReleaseDC,
    SRCCOPY, SelectObject,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
};

use crate::hold::Pt;

/// "BM", the BMP file signature.
const MAGIC: &[u8; 2] = b"BM";
/// Size of the BMP file header, in bytes.
const FILE_HEADER: u32 = 14;
/// Size of the info header, in bytes.
const INFO: u32 = size_of::<BITMAPINFOHEADER>() as u32;
/// Where the pixels start: right after both headers.
const PIXELS_AT: u32 = FILE_HEADER + INFO;
/// Bits per pixel: blue, green, red and one unused byte.
const BPP: u16 = 32;
/// Bytes per pixel.
pub const BYTES_PP: u32 = BPP as u32 / 8;
/// Colour planes; BMP files always have one.
const PLANES: u16 = 1;

/// A copy of a screen rectangle: its top-left in screen pixels and its BGRA pixels, top row first.
#[derive(Clone, PartialEq, Eq)]
pub struct Shot {
    /// Left edge on the screen.
    pub left: i32,
    /// Top edge on the screen.
    pub top: i32,
    /// Width in pixels.
    pub width: i32,
    /// Height in pixels.
    pub height: i32,
    /// Blue, green, red and one unused byte per pixel.
    pub bgra: Vec<u8>,
}

/// The virtual desktop: left, top, width and height in physical pixels.
pub fn desktop() -> (i32, i32, i32, i32) {
    // SAFETY: plain metric queries.
    unsafe {
        (
            GetSystemMetrics(SM_XVIRTUALSCREEN),
            GetSystemMetrics(SM_YVIRTUALSCREEN),
            GetSystemMetrics(SM_CXVIRTUALSCREEN),
            GetSystemMetrics(SM_CYVIRTUALSCREEN),
        )
    }
}

/// Bytes of a `w`×`h` copy, or `None` when it is empty, overflows or passes `cap`.
fn byte_len(w: i32, h: i32, cap: usize) -> Option<usize> {
    let (w, h) = (usize::try_from(w).ok()?, usize::try_from(h).ok()?);
    let n = w.checked_mul(h)?.checked_mul(BYTES_PP as usize)?;
    (n > 0 && n <= cap).then_some(n)
}

/// The info header of a top-down `w`×`h` picture of 32-bit pixels, `size` bytes in all.
fn info_header(w: i32, h: i32, size: u32) -> BITMAPINFOHEADER {
    BITMAPINFOHEADER {
        biSize: INFO,
        biWidth: w,
        // A negative height means the first row is the top one.
        biHeight: -h,
        biPlanes: PLANES,
        biBitCount: BPP,
        biCompression: BI_RGB.0,
        biSizeImage: size,
        ..Default::default()
    }
}

/// BMP file header plus `info_header(w, h, size)`, as bytes.
pub fn header(w: i32, h: i32, size: u32) -> Vec<u8> {
    let i = info_header(w, h, size);
    let mut b = Vec::with_capacity(PIXELS_AT as usize);
    b.extend_from_slice(MAGIC);
    b.extend_from_slice(&(PIXELS_AT + i.biSizeImage).to_le_bytes());
    b.extend_from_slice(&0u32.to_le_bytes());
    b.extend_from_slice(&PIXELS_AT.to_le_bytes());
    b.extend_from_slice(&i.biSize.to_le_bytes());
    b.extend_from_slice(&i.biWidth.to_le_bytes());
    b.extend_from_slice(&i.biHeight.to_le_bytes());
    b.extend_from_slice(&i.biPlanes.to_le_bytes());
    b.extend_from_slice(&i.biBitCount.to_le_bytes());
    b.extend_from_slice(&i.biCompression.to_le_bytes());
    b.extend_from_slice(&i.biSizeImage.to_le_bytes());
    b.extend_from_slice(&i.biXPelsPerMeter.to_le_bytes());
    b.extend_from_slice(&i.biYPelsPerMeter.to_le_bytes());
    b.extend_from_slice(&i.biClrUsed.to_le_bytes());
    b.extend_from_slice(&i.biClrImportant.to_le_bytes());
    b
}

/// The pixels of the screen rectangle at (x, y), `w`×`h`, top row first; layered windows included.
/// Refuses a copy bigger than `cap` bytes.
pub fn grab(x: i32, y: i32, w: i32, h: i32, cap: usize) -> Result<Vec<u8>, String> {
    let too_big = || format!("cannot copy a {w}x{h} screen area");
    let len = byte_len(w, h, cap).ok_or_else(too_big)?;
    let mut info = BITMAPINFO {
        bmiHeader: info_header(w, h, u32::try_from(len).map_err(|_| too_big())?),
        ..Default::default()
    };
    let mut pixels = vec![0u8; len];
    // SAFETY: every handle made here is released before returning; `pixels` holds `h` rows of
    // `w` 32-bit pixels, as `info` tells GetDIBits, and the bitmap is deselected before it runs.
    let (copied, rows) = unsafe {
        let screen = GetDC(None);
        let mem = CreateCompatibleDC(Some(screen));
        let bmp = CreateCompatibleBitmap(screen, w, h);
        let old = SelectObject(mem, bmp.into());
        let copied = BitBlt(mem, 0, 0, w, h, Some(screen), x, y, SRCCOPY | CAPTUREBLT);
        SelectObject(mem, old);
        let buf = Some(pixels.as_mut_ptr().cast());
        let rows = GetDIBits(mem, bmp, 0, h as u32, buf, &mut info, DIB_RGB_COLORS);
        let _ = DeleteObject(bmp.into());
        let _ = DeleteDC(mem);
        ReleaseDC(None, screen);
        (copied, rows)
    };
    copied.map_err(|e| format!("BitBlt: {e}"))?;
    if rows != h {
        return Err(format!("GetDIBits copied {rows} of {h} rows"));
    }
    Ok(pixels)
}

/// A frozen copy of the whole desktop, refused when bigger than `cap` bytes.
pub fn screen(cap: usize) -> Result<Shot, String> {
    let (left, top, width, height) = desktop();
    let bgra = grab(left, top, width, height, cap)?;
    Ok(Shot {
        left,
        top,
        width,
        height,
        bgra,
    })
}

/// The box between two picked corners, whatever order they were picked in; both corner pixels are inside.
pub fn region(a: Pt, b: Pt) -> RECT {
    RECT {
        left: a.x.min(b.x),
        top: a.y.min(b.y),
        right: a.x.max(b.x).saturating_add(1),
        bottom: a.y.max(b.y).saturating_add(1),
    }
}

impl Shot {
    /// The part of this copy inside screen box `r`, or `None` when they do not overlap.
    pub fn crop(&self, r: &RECT) -> Option<Shot> {
        let left = r.left.max(self.left);
        let top = r.top.max(self.top);
        let right = r.right.min(self.left.saturating_add(self.width));
        let bottom = r.bottom.min(self.top.saturating_add(self.height));
        if right <= left || bottom <= top {
            return None;
        }
        let (w, row) = (
            (right - left) as usize,
            self.width as usize * BYTES_PP as usize,
        );
        let mut bgra = Vec::with_capacity(w * (bottom - top) as usize * BYTES_PP as usize);
        for y in top..bottom {
            let at =
                (y - self.top) as usize * row + (left - self.left) as usize * BYTES_PP as usize;
            bgra.extend_from_slice(self.bgra.get(at..at + w * BYTES_PP as usize)?);
        }
        Some(Shot {
            left,
            top,
            width: right - left,
            height: bottom - top,
            bgra,
        })
    }

    /// This copy as a BMP file; fails only when it is too big for one.
    pub fn to_bmp(&self) -> Result<Vec<u8>, String> {
        let size = u32::try_from(self.bgra.len()).map_err(|_| "too big for a BMP file")?;
        let mut file = header(self.width, self.height, size);
        file.extend_from_slice(&self.bgra);
        Ok(file)
    }

    /// This copy's pixels as RGBA with full alpha, for a toolkit image.
    pub fn rgba(&self) -> Vec<u8> {
        self.bgra
            .as_chunks::<{ BYTES_PP as usize }>()
            .0
            .iter()
            .flat_map(|&[b, g, r, _]| [r, g, b, u8::MAX])
            .collect()
    }
}

/// Reads a BMP file written by `Shot::to_bmp`, up to `cap` pixel bytes; its place on the screen is unknown, so it is (0, 0).
pub fn from_bmp(b: &[u8], cap: usize) -> Result<Shot, String> {
    let u32_at = |at: usize| -> Option<u32> {
        Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
    };
    let bad = || "not a BMP file from a snip".to_string();
    if b.get(..2) != Some(MAGIC.as_slice()) || u32_at(10) != Some(PIXELS_AT) {
        return Err(bad());
    }
    let width = u32_at(18).ok_or_else(bad)? as i32;
    let height = (u32_at(22).ok_or_else(bad)? as i32)
        .checked_neg()
        .ok_or_else(bad)?;
    let size = byte_len(width, height, cap).ok_or_else(bad)?;
    let bgra = b
        .get(PIXELS_AT as usize..PIXELS_AT as usize + size)
        .ok_or_else(bad)?;
    Ok(Shot {
        left: 0,
        top: 0,
        width,
        height,
        bgra: bgra.to_vec(),
    })
}

#[cfg(test)]
mod tests;
