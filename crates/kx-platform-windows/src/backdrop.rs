//! The material behind a window: Mica or Acrylic from DWM, or none so the UI draws a solid plate.
//! Which DWM backdrop each kind is, and how far the frame reaches, are pure and tested everywhere.
#![cfg_attr(
    not(windows),
    allow(
        dead_code,
        reason = "The backdrop table serves only the DWM calls in win."
    )
)]

use kx_platform::ui::{BackdropKind, UiError};

#[cfg(windows)]
pub use win::WinBackdrop;

/// `DWMSBT_NONE`: no system material.
const DWMSBT_NONE: i32 = 1;
/// `DWMSBT_MAINWINDOW`: Mica.
const DWMSBT_MAINWINDOW: i32 = 2;
/// `DWMSBT_TRANSIENTWINDOW`: Acrylic.
const DWMSBT_TRANSIENTWINDOW: i32 = 3;
/// A frame margin of -1 reaches over the whole window, so the material shows behind all of it.
const SHEET: i32 = -1;
/// `E_INVALIDARG`: DWM's answer on a Windows without system backdrops (before Windows 11 22H2).
const UNSUPPORTED: i32 = 0x8007_0057_u32.cast_signed();

/// The result of trying `kind`: clearing a backdrop Windows does not have leaves nothing to clear.
fn outcome(kind: BackdropKind, tried: Result<(), UiError>) -> Result<(), UiError> {
    match tried {
        Err(UiError::Refused(UNSUPPORTED)) if kind == BackdropKind::None => Ok(()),
        other => other,
    }
}

/// The DWM backdrop type for `kind`.
fn system_type(kind: BackdropKind) -> i32 {
    match kind {
        BackdropKind::None => DWMSBT_NONE,
        BackdropKind::Mica => DWMSBT_MAINWINDOW,
        BackdropKind::Acrylic => DWMSBT_TRANSIENTWINDOW,
    }
}

/// How far the frame reaches into the window: all of it for a material, none for a plain plate.
fn margin(kind: BackdropKind) -> i32 {
    match kind {
        BackdropKind::None => 0,
        BackdropKind::Mica | BackdropKind::Acrylic => SHEET,
    }
}

#[cfg(windows)]
mod win {
    use std::ffi::c_void;

    use kx_platform::ui::{Backdrop, BackdropKind, UiError};
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Dwm::{
        DWM_SYSTEMBACKDROP_TYPE, DWMWA_SYSTEMBACKDROP_TYPE, DWMWA_USE_IMMERSIVE_DARK_MODE,
        DWMWINDOWATTRIBUTE, DwmExtendFrameIntoClientArea, DwmSetWindowAttribute,
    };
    use windows::Win32::UI::Controls::MARGINS;
    use windows::core::BOOL;

    use super::{margin, outcome, system_type};
    use crate::refused;
    use crate::window::own_titled;

    /// DWM backdrops on our own windows; a Windows without them refuses a material, and the UI draws a solid plate.
    #[derive(Clone, Copy, Debug, Default)]
    pub struct WinBackdrop;

    impl Backdrop for WinBackdrop {
        fn apply(&self, title: &str, kind: BackdropKind, dark: bool) -> Result<(), UiError> {
            let hwnd = own_titled(title).ok_or(UiError::NoWindow)?;
            if kind != BackdropKind::None {
                set(hwnd, DWMWA_USE_IMMERSIVE_DARK_MODE, &BOOL::from(dark))?;
            }
            extend(hwnd, margin(kind))?;
            let backdrop = DWM_SYSTEMBACKDROP_TYPE(system_type(kind));
            outcome(kind, set(hwnd, DWMWA_SYSTEMBACKDROP_TYPE, &backdrop))
        }
    }

    /// Reaches the frame `m` pixels into every side of our window `hwnd`; -1 covers all of it.
    #[allow(unsafe_code, reason = "A DWM call with a local struct.")]
    fn extend(hwnd: HWND, m: i32) -> Result<(), UiError> {
        let margins = MARGINS {
            cxLeftWidth: m,
            cxRightWidth: m,
            cyTopHeight: m,
            cyBottomHeight: m,
        };
        // SAFETY: `hwnd` is one of our live windows and `margins` outlives the call.
        unsafe { DwmExtendFrameIntoClientArea(hwnd, &raw const margins) }.map_err(|e| refused(&e))
    }

    /// Sets DWM attribute `attr` of our window `hwnd` to `value`.
    #[allow(unsafe_code, reason = "A DWM call with a local value and its size.")]
    fn set<T>(hwnd: HWND, attr: DWMWINDOWATTRIBUTE, value: &T) -> Result<(), UiError> {
        // Values here are a few bytes; a size past `u32` would read as 0, which DWM refuses.
        let size = u32::try_from(size_of::<T>()).unwrap_or_default();
        let data = std::ptr::from_ref(value).cast::<c_void>();
        // SAFETY: `value` outlives the call and `size` is its exact size.
        unsafe { DwmSetWindowAttribute(hwnd, attr, data, size) }.map_err(|e| refused(&e))
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn a_window_we_do_not_show_has_no_backdrop() {
            for kind in [
                BackdropKind::None,
                BackdropKind::Mica,
                BackdropKind::Acrylic,
            ] {
                let tried = WinBackdrop.apply("KeyXtend test: no such window", kind, true);
                assert_eq!(tried, Err(UiError::NoWindow), "{kind:?}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_kind_names_its_dwm_backdrop() {
        assert_eq!(system_type(BackdropKind::None), DWMSBT_NONE);
        assert_eq!(system_type(BackdropKind::Mica), DWMSBT_MAINWINDOW);
        assert_eq!(system_type(BackdropKind::Acrylic), DWMSBT_TRANSIENTWINDOW);
    }

    #[test]
    fn a_material_covers_the_whole_window_and_none_puts_the_frame_back() {
        assert_eq!(margin(BackdropKind::Mica), SHEET);
        assert_eq!(margin(BackdropKind::Acrylic), SHEET);
        assert_eq!(margin(BackdropKind::None), 0);
    }

    #[test]
    fn without_system_backdrops_a_material_is_refused_and_none_has_nothing_to_clear() {
        let unsupported = Err(UiError::Refused(UNSUPPORTED));
        assert_eq!(outcome(BackdropKind::None, unsupported), Ok(()));
        for kind in [BackdropKind::Mica, BackdropKind::Acrylic] {
            assert_eq!(outcome(kind, unsupported), unsupported, "{kind:?}");
        }
    }

    #[test]
    fn other_answers_pass_through() {
        let other = Err(UiError::Refused(5));
        assert_eq!(outcome(BackdropKind::None, other), other);
        let gone = Err(UiError::NoWindow);
        assert_eq!(outcome(BackdropKind::None, gone), gone);
        assert_eq!(outcome(BackdropKind::Mica, Ok(())), Ok(()));
    }

    #[cfg(windows)]
    #[test]
    fn the_backdrop_table_matches_windows() {
        use windows::Win32::Foundation::E_INVALIDARG;
        use windows::Win32::Graphics::Dwm::{
            DWMSBT_MAINWINDOW as MICA, DWMSBT_NONE as NONE, DWMSBT_TRANSIENTWINDOW as ACRYLIC,
        };
        assert_eq!(DWMSBT_NONE, NONE.0);
        assert_eq!(DWMSBT_MAINWINDOW, MICA.0);
        assert_eq!(DWMSBT_TRANSIENTWINDOW, ACRYLIC.0);
        assert_eq!(UNSUPPORTED, E_INVALIDARG.0);
    }
}
