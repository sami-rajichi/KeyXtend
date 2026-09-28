//! Native Adaptive's frosted plate: a DWM backdrop or the accent-policy blur behind our window, else a solid plate.

use std::ffi::c_void;

use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Dwm::{
    DWM_SYSTEMBACKDROP_TYPE, DWMSBT_MAINWINDOW, DWMSBT_TRANSIENTWINDOW, DWMWA_SYSTEMBACKDROP_TYPE,
    DWMWA_USE_IMMERSIVE_DARK_MODE, DwmExtendFrameIntoClientArea, DwmSetWindowAttribute,
};
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
use windows::Win32::UI::Controls::MARGINS;
use windows::core::{BOOL, s, w};

use crate::lookcfg::Backdrop;

/// Margins of -1 extend the frame over the whole window, so the backdrop shows through.
const SHEET: i32 = -1;
/// `WCA_ACCENT_POLICY`, the undocumented composition attribute for the blur.
const WCA_ACCENT_POLICY: u32 = 19;
/// `ACCENT_ENABLE_ACRYLICBLURBEHIND`.
const ACCENT_ACRYLIC: u32 = 4;
/// A near-clear tint (`AABBGGRR`): some builds draw no blur at alpha 0, and the face paints the plate itself.
const CLEAR_TINT: u32 = 0x0100_0000;

/// What the face should paint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Plate {
    /// A solid colour.
    Solid,
    /// The plate at `plate_alpha`, since Windows took the backdrop; G13 checks by eye that it really shows.
    SeeThrough,
}

/// The plate after trying backdrop `b`; any refusal leaves it solid.
pub fn plate_after(b: Backdrop, tried: Result<(), String>) -> Plate {
    match (b, tried) {
        (Backdrop::None, _) | (_, Err(_)) => Plate::Solid,
        _ => Plate::SeeThrough,
    }
}

/// Puts backdrop `b` behind our window `hwnd`, tinted for `dark`; `None` tries nothing.
pub fn apply(hwnd: HWND, b: Backdrop, dark: bool) -> Result<(), String> {
    match b {
        Backdrop::None => Ok(()),
        Backdrop::Mica => set_dark(hwnd, dark).and_then(|()| system(hwnd, DWMSBT_MAINWINDOW)),
        Backdrop::Acrylic => {
            set_dark(hwnd, dark).and_then(|()| system(hwnd, DWMSBT_TRANSIENTWINDOW))
        }
        Backdrop::Blur => blur(hwnd),
    }
}

/// Tells DWM whether our window is dark, which sets the Mica and Acrylic tint; call again on each mode change.
pub fn set_dark(hwnd: HWND, dark: bool) -> Result<(), String> {
    let on = BOOL::from(dark);
    // SAFETY: our own live window; `on` outlives the call and its size is passed.
    unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            std::ptr::from_ref(&on).cast(),
            size_of::<BOOL>() as u32,
        )
    }
    .map_err(|e| e.to_string())
}

/// A DWM system backdrop; Windows may draw it solid while the window is inactive.
fn system(hwnd: HWND, kind: DWM_SYSTEMBACKDROP_TYPE) -> Result<(), String> {
    let m = MARGINS {
        cxLeftWidth: SHEET,
        cxRightWidth: SHEET,
        cyTopHeight: SHEET,
        cyBottomHeight: SHEET,
    };
    let size = size_of::<DWM_SYSTEMBACKDROP_TYPE>() as u32;
    // SAFETY: our own live window; `kind` outlives the call and its size is passed.
    unsafe {
        DwmExtendFrameIntoClientArea(hwnd, &m).map_err(|e| e.to_string())?;
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_SYSTEMBACKDROP_TYPE,
            std::ptr::from_ref(&kind).cast(),
            size,
        )
        .map_err(|e| e.to_string())
    }
}

/// The accent policy, as user32 reads it.
#[repr(C)]
struct AccentPolicy {
    state: u32,
    flags: u32,
    tint: u32,
    animation: u32,
}

/// The attribute block passed to `SetWindowCompositionAttribute`.
#[repr(C)]
struct CompositionData {
    attrib: u32,
    data: *mut c_void,
    size: usize,
}

/// `SetWindowCompositionAttribute` from user32.
type SetComposition = unsafe extern "system" fn(HWND, *mut CompositionData) -> BOOL;

/// The accent-policy blur: undocumented, but it stays on while the window is inactive.
fn blur(hwnd: HWND) -> Result<(), String> {
    // SAFETY: looks up an export of user32, which every GUI process has loaded.
    let proc = unsafe {
        let user32 = GetModuleHandleW(w!("user32.dll")).map_err(|e| e.to_string())?;
        GetProcAddress(user32, s!("SetWindowCompositionAttribute"))
    }
    .ok_or("SetWindowCompositionAttribute is missing")?;
    // SAFETY: this export has had this signature since Windows 10; the pointer came from GetProcAddress.
    let set: SetComposition = unsafe { std::mem::transmute(proc) };
    let mut policy = AccentPolicy {
        state: ACCENT_ACRYLIC,
        flags: 0,
        tint: CLEAR_TINT,
        animation: 0,
    };
    let mut data = CompositionData {
        attrib: WCA_ACCENT_POLICY,
        data: std::ptr::from_mut(&mut policy).cast(),
        size: size_of::<AccentPolicy>(),
    };
    // SAFETY: our own live window; `data` and `policy` outlive the call.
    let ok = unsafe { set(hwnd, &mut data) }.as_bool();
    if ok {
        Ok(())
    } else {
        Err("the accent-policy blur was refused".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lookcfg::Backdrop;

    #[test]
    fn a_refused_backdrop_leaves_a_solid_plate() {
        assert_eq!(plate_after(Backdrop::Acrylic, Ok(())), Plate::SeeThrough);
        assert_eq!(
            plate_after(Backdrop::Blur, Err("refused".into())),
            Plate::Solid
        );
        assert_eq!(
            plate_after(Backdrop::None, Ok(())),
            Plate::Solid,
            "none is never tried"
        );
    }

    #[test]
    fn nothing_is_tried_for_no_backdrop() {
        assert_eq!(
            apply(
                windows::Win32::Foundation::HWND::default(),
                Backdrop::None,
                true
            ),
            Ok(())
        );
    }
}
