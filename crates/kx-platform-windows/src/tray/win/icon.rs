//! The tray icon's picture as a Windows icon, the taskbar's icon size, and the one call to the taskbar.

use std::fmt;

use kx_platform::ui::{TrayImage, UiError};
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateBitmap, CreateDIBSection, DIB_RGB_COLORS,
    DeleteObject, HBITMAP,
};
use windows::Win32::UI::HiDpi::GetSystemMetricsForDpi;
use windows::Win32::UI::Shell::{NOTIFY_ICON_MESSAGE, NOTIFYICONDATAW, Shell_NotifyIconW};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateIconIndirect, DestroyIcon, FindWindowW, HICON, ICONINFO, SM_CXSMICON,
};
use windows::core::PCWSTR;

use crate::screen::window_dpi;
use crate::tray::{BGRA, TASKBAR, bgra, mask};
use crate::{failed, refused, wide};

/// The small icon's side at 100 % scale, when Windows cannot say.
const BASE_PX: u32 = 16;
/// Bits per pixel of the colour picture, and of the mask.
const COLOR_BITS: u16 = 32;
const MASK_BITS: u32 = 1;

/// The taskbar's notification-area call, so tests never touch the real taskbar.
pub(crate) trait Shell: fmt::Debug + Send + Sync {
    /// Sends `what` with `data` to the taskbar; false when it refuses.
    fn notify(&self, what: NOTIFY_ICON_MESSAGE, data: &NOTIFYICONDATAW) -> bool;
}

/// The real taskbar.
#[derive(Debug)]
pub(crate) struct Taskbar;

impl Shell for Taskbar {
    #[allow(unsafe_code, reason = "The one call to the taskbar.")]
    fn notify(&self, what: NOTIFY_ICON_MESSAGE, data: &NOTIFYICONDATAW) -> bool {
        // SAFETY: `data` is a whole NOTIFYICONDATAW with its size set, alive through the call.
        unsafe { Shell_NotifyIconW(what, data) }.as_bool()
    }
}

/// The small icon's side in pixels, which the taskbar draws, at the DPI of the screen that holds it.
pub(super) fn small_px() -> u32 {
    small_px_at(taskbar_dpi())
}

/// The small icon's side in pixels at `dpi`; the base size when Windows cannot say.
#[allow(unsafe_code, reason = "A plain metrics query.")]
fn small_px_at(dpi: u32) -> u32 {
    // SAFETY: a plain query with a known index.
    let px = unsafe { GetSystemMetricsForDpi(SM_CXSMICON, dpi) };
    u32::try_from(px).ok().filter(|p| *p > 0).unwrap_or(BASE_PX)
}

/// The DPI of the screen holding the main taskbar, else the main screen's.
#[allow(unsafe_code, reason = "Finds the taskbar's window.")]
fn taskbar_dpi() -> u32 {
    let class = wide(TASKBAR);
    // SAFETY: `class` ends with a null and outlives the call; the taskbar is only looked up.
    let bar = unsafe { FindWindowW(PCWSTR(class.as_ptr()), PCWSTR::null()) };
    window_dpi(bar.ok())
}

/// An icon we own, destroyed once when dropped.
#[derive(Debug)]
pub(super) struct Icon(HICON);

// SAFETY: an icon is a process-wide user object; any thread may hand it to the taskbar or destroy it.
#[allow(unsafe_code, reason = "An icon handle may move between threads.")]
unsafe impl Send for Icon {}
// SAFETY: shared owners only read the handle; the last one destroys it, once.
#[allow(unsafe_code, reason = "Icon handles are shared.")]
unsafe impl Sync for Icon {}

impl Icon {
    /// The icon of `image`, which its alpha shapes.
    ///
    /// # Errors
    /// `Refused` when Windows will not make the bitmaps or the icon.
    #[allow(unsafe_code, reason = "Makes an icon from two bitmaps of ours.")]
    pub(super) fn new(image: &TrayImage) -> Result<Self, UiError> {
        let side = |n: u32| i32::try_from(n).map_err(|_| failed());
        let (w, h) = (side(image.width())?, side(image.height())?);
        // The mask first, so a mask that cannot be made leaves no colour bitmap behind.
        let bits = mask(image.width(), image.height()).ok_or_else(failed)?;
        let color = color(w, h, &bgra(image.rgba()))?;
        // SAFETY: a 1-bit bitmap of `w` by `h` read from `bits`, whose rows are padded as CreateBitmap reads them.
        let mono = unsafe { CreateBitmap(w, h, 1, MASK_BITS, Some(bits.as_ptr().cast())) };
        let info = ICONINFO {
            fIcon: true.into(),
            hbmMask: mono,
            hbmColor: color,
            ..ICONINFO::default()
        };
        // SAFETY: both bitmaps are ours and alive through the call, which copies them into the icon.
        let made = unsafe { CreateIconIndirect(&raw const info) };
        drop_bitmap(color);
        drop_bitmap(mono);
        made.map(Self).map_err(|e| refused(&e))
    }

    /// The handle, for the taskbar call.
    pub(super) fn handle(&self) -> HICON {
        self.0
    }
}

impl Drop for Icon {
    #[allow(unsafe_code, reason = "Destroys the icon we own.")]
    fn drop(&mut self) {
        // SAFETY: an icon Windows made for us, destroyed once.
        let _ = unsafe { DestroyIcon(self.0) };
    }
}

/// A top-down 32-bit bitmap of `w` by `h` holding `bgra`.
#[allow(unsafe_code, reason = "Makes a bitmap and fills its pixels.")]
fn color(w: i32, h: i32, bgra: &[u8]) -> Result<HBITMAP, UiError> {
    let header = BITMAPINFOHEADER {
        biSize: u32::try_from(size_of::<BITMAPINFOHEADER>()).map_err(|_| failed())?,
        biWidth: w,
        biHeight: -h,
        biPlanes: 1,
        biBitCount: COLOR_BITS,
        biCompression: BI_RGB.0,
        ..BITMAPINFOHEADER::default()
    };
    let info = BITMAPINFO {
        bmiHeader: header,
        ..BITMAPINFO::default()
    };
    let mut bits = std::ptr::null_mut();
    // SAFETY: `info` describes the bitmap and outlives the call; Windows points `bits` at its pixels.
    let map = unsafe {
        CreateDIBSection(
            None,
            &raw const info,
            DIB_RGB_COLORS,
            &raw mut bits,
            None,
            0,
        )
    }
    .map_err(|e| refused(&e))?;
    let room = [w, h].map(usize::try_from);
    let fits = matches!(room, [Ok(w), Ok(h)] if w.checked_mul(h).and_then(|n| n.checked_mul(BGRA)) == Some(bgra.len()));
    if bits.is_null() || !fits {
        drop_bitmap(map);
        return Err(failed());
    }
    // SAFETY: the bitmap holds `w` by `h` pixels of four bytes, which is `bgra`'s length, checked just above.
    unsafe { std::ptr::copy_nonoverlapping(bgra.as_ptr(), bits.cast::<u8>(), bgra.len()) };
    Ok(map)
}

/// Deletes a bitmap we made; a null one is skipped.
#[allow(unsafe_code, reason = "Deletes a bitmap we own.")]
fn drop_bitmap(map: HBITMAP) {
    if !map.is_invalid() {
        // SAFETY: a bitmap we made and no longer use.
        let _ = unsafe { DeleteObject(map.into()) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kx_module_api::ports::BASE_DPI;
    use windows::Win32::Graphics::Gdi::{BITMAP, GetObjectW};
    use windows::Win32::UI::WindowsAndMessaging::GetIconInfo;

    #[test]
    fn the_taskbar_draws_icons_of_at_least_the_base_size() {
        assert!(small_px() >= BASE_PX);
    }

    #[test]
    fn the_small_icon_follows_the_dpi_asked_for() {
        assert_eq!(small_px_at(BASE_DPI), BASE_PX);
        assert_eq!(small_px_at(2 * BASE_DPI), 2 * BASE_PX);
        assert_eq!(small_px_at(0), BASE_PX, "a DPI Windows cannot use");
    }

    #[test]
    #[allow(unsafe_code, reason = "Reads back the test's own icon.")]
    fn an_icon_is_made_at_the_picture_size() {
        let image = TrayImage::new(3, 2, vec![0x80; 24]).unwrap();
        let icon = Icon::new(&image).unwrap();
        let mut info = ICONINFO::default();
        // SAFETY: our own icon; `info` is a local it fills.
        unsafe { GetIconInfo(icon.handle(), &raw mut info) }.unwrap();
        let mut map = BITMAP::default();
        let size = i32::try_from(size_of::<BITMAP>()).unwrap();
        // SAFETY: reads the colour bitmap GetIconInfo copied for us into a local of its size.
        let read = unsafe { GetObjectW(info.hbmColor.into(), size, Some((&raw mut map).cast())) };
        drop_bitmap(info.hbmColor);
        drop_bitmap(info.hbmMask);
        assert!(info.fIcon.as_bool() && read > 0);
        assert_eq!(
            (map.bmWidth, map.bmHeight, map.bmBitsPixel),
            (3, 2, COLOR_BITS)
        );
    }
}
