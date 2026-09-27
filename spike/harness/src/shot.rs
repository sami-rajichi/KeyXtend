//! Screenshots of the whole desktop as BMP files, taken when something blocks a test.

use std::path::Path;

use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BitBlt, CAPTUREBLT, CreateCompatibleBitmap,
    CreateCompatibleDC, DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDC, GetDIBits, ReleaseDC,
    SRCCOPY, SelectObject,
};

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

/// The info header of a top-down `w`×`h` picture of 32-bit pixels.
fn info_header(w: i32, h: i32) -> BITMAPINFOHEADER {
    BITMAPINFOHEADER {
        biSize: INFO,
        biWidth: w,
        // A negative height means the first row is the top one.
        biHeight: -h,
        biPlanes: PLANES,
        biBitCount: BPP,
        biCompression: BI_RGB.0,
        biSizeImage: w.unsigned_abs() * h.unsigned_abs() * BYTES_PP,
        ..Default::default()
    }
}

/// BMP file header plus `info_header(w, h)`, as bytes.
fn header(w: i32, h: i32) -> Vec<u8> {
    let i = info_header(w, h);
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
pub fn grab(x: i32, y: i32, w: i32, h: i32) -> Result<Vec<u8>, String> {
    let mut info = BITMAPINFO {
        bmiHeader: info_header(w, h),
        ..Default::default()
    };
    let mut pixels = vec![0u8; info.bmiHeader.biSizeImage as usize];
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

/// Saves the whole desktop to `path` as a BMP file.
pub fn save(path: &Path) -> Result<(), String> {
    let (x, y, w, h) = crate::win::desktop();
    if w <= 0 || h <= 0 {
        return Err(format!("the desktop is {w}x{h} pixels; nothing to capture"));
    }
    let pixels = grab(x, y, w, h)?;
    let mut file = header(w, h);
    file.extend_from_slice(&pixels);
    std::fs::write(path, file).map_err(|e| format!("{}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn u32_at(b: &[u8], at: usize) -> u32 {
        u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
    }

    #[test]
    fn header_describes_a_top_down_32_bit_picture() {
        let b = header(3, 2);
        assert_eq!(b.len(), 54);
        assert_eq!(&b[..2], b"BM");
        assert_eq!(u32_at(&b, 2), 54 + 3 * 2 * 4);
        assert_eq!(u32_at(&b, 10), 54);
        assert_eq!(u32_at(&b, 14), 40);
        assert_eq!(u32_at(&b, 18), 3);
        assert_eq!(u32_at(&b, 22) as i32, -2);
        assert_eq!(u16::from_le_bytes([b[28], b[29]]), 32);
        assert_eq!(u32_at(&b, 34), 3 * 2 * 4);
    }
}
