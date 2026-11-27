//! The grab, evolved from the spike's `capture.rs` (G24): each screen copied through one screen DC, one right after the
//! other, as raw top-down pixels with layered windows; a shielded window is in none.

use kx_module_api::ports::{BGRX_BYTES, FreezeError, FrozenScreen, Rect, Shot};
use windows::Win32::Foundation::{E_FAIL, E_INVALIDARG, E_OUTOFMEMORY};
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BitBlt, CAPTUREBLT, CreateCompatibleBitmap,
    CreateCompatibleDC, DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDC, GetDIBits, HDC, ReleaseDC,
    SRCCOPY, SelectObject,
};
use windows::core::HRESULT;

/// Bits in one raw pixel: `BGRX_BYTES` bytes.
#[allow(
    clippy::cast_possible_truncation,
    reason = "A pixel's few bytes of bits fit a u16."
)]
const BITS: u16 = (BGRX_BYTES * u8::BITS as usize) as u16;
/// Colour planes; a bitmap always has one.
const PLANES: u16 = 1;

/// Grabs each of `screens`, a box and a DPI, through one screen DC, in the same order.
///
/// # Errors
/// `Refused` with `E_INVALIDARG` for a box with no pixels or too many, `E_OUTOFMEMORY` when its pixels find no room, or
/// the system's code when it will not copy.
pub(crate) fn grab(screens: &[(Rect, u32)]) -> Result<Vec<FrozenScreen>, FreezeError> {
    let screen = ScreenDc::new();
    let one = |&(rect, dpi): &(Rect, u32)| grab_one(&screen, rect, dpi);
    screens.iter().map(one).collect()
}

/// A `w` by `h` bitmap of raw pixels, top row first.
pub(super) fn top_down(w: i32, h: i32) -> BITMAPINFO {
    BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: u32::try_from(size_of::<BITMAPINFOHEADER>()).unwrap_or(u32::MAX),
            biWidth: w,
            // A negative height puts the top row first.
            biHeight: h.saturating_neg(),
            biPlanes: PLANES,
            biBitCount: BITS,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    }
}

/// The whole screen's device context, let go when dropped.
struct ScreenDc(HDC);

impl ScreenDc {
    #[allow(unsafe_code, reason = "Takes the screen's device context.")]
    fn new() -> Self {
        // SAFETY: a plain query; the context is let go once, when dropped.
        Self(unsafe { GetDC(None) })
    }
}

impl Drop for ScreenDc {
    #[allow(unsafe_code, reason = "Lets the screen's device context go.")]
    fn drop(&mut self) {
        // SAFETY: the context `new` took, let go once.
        let _ = unsafe { ReleaseDC(None, self.0) };
    }
}

/// The screen at `rect`, read at `dpi`, grabbed through `screen`; its size is checked before any copy.
fn grab_one(screen: &ScreenDc, rect: Rect, dpi: u32) -> Result<FrozenScreen, FreezeError> {
    let (Ok(w), Ok(h)) = (u32::try_from(rect.w), u32::try_from(rect.h)) else {
        return Err(refused(E_INVALIDARG));
    };
    let len = bytes(w, h).ok_or(refused(E_INVALIDARG))?;
    let mut bgrx = Vec::new();
    bgrx.try_reserve_exact(len)
        .map_err(|_| refused(E_OUTOFMEMORY))?;
    bgrx.resize(len, 0);
    copy(screen.0, rect, h, &mut bgrx)?;
    let shot = Shot::new(w, h, bgrx).ok_or(refused(E_FAIL))?;
    FrozenScreen::new(rect, dpi, shot).ok_or(refused(E_FAIL))
}

/// The bytes of a `w` by `h` grab; `None` when it is empty or larger than any buffer can be.
fn bytes(w: u32, h: u32) -> Option<usize> {
    let pixels = usize::try_from(w)
        .ok()?
        .checked_mul(usize::try_from(h).ok()?)?;
    let n = pixels.checked_mul(BGRX_BYTES)?;
    (n > 0 && isize::try_from(n).is_ok()).then_some(n)
}

/// Copies the screen's box `rect`, `rows` high, into `bgrx`, which holds exactly its pixels, layered windows included.
#[allow(
    unsafe_code,
    reason = "Copies the screen through GDI into our own buffer."
)]
fn copy(screen: HDC, rect: Rect, rows: u32, bgrx: &mut [u8]) -> Result<(), FreezeError> {
    let mut info = top_down(rect.w, rect.h);
    let rop = SRCCOPY | CAPTUREBLT;
    // SAFETY: every handle made here is let go before the end; `bgrx` holds `rows` rows of `rect.w` 32-bit pixels, as
    // `info` tells GetDIBits, and the bitmap is deselected before it runs.
    let (copied, read) = unsafe {
        let mem = CreateCompatibleDC(Some(screen));
        let bmp = CreateCompatibleBitmap(screen, rect.w, rect.h);
        let old = SelectObject(mem, bmp.into());
        let copied = BitBlt(mem, 0, 0, rect.w, rect.h, Some(screen), rect.x, rect.y, rop);
        SelectObject(mem, old);
        let out = Some(bgrx.as_mut_ptr().cast());
        let read = GetDIBits(mem, bmp, 0, rows, out, &raw mut info, DIB_RGB_COLORS);
        let _ = DeleteObject(bmp.into());
        let _ = DeleteDC(mem);
        (copied, read)
    };
    copied.map_err(|e| FreezeError::Refused(e.code().0))?;
    let whole = u32::try_from(read).is_ok_and(|n| n == rows);
    whole.then_some(()).ok_or(refused(E_FAIL))
}

/// A refused freeze with Windows' code `code`.
const fn refused(code: HRESULT) -> FreezeError {
    FreezeError::Refused(code.0)
}
