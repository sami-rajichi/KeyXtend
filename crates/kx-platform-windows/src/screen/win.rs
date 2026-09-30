//! The Windows side of the screens: monitors, their work areas and DPI, and the pointer.
//! Boxes are in physical pixels while the process is per-monitor DPI aware, as Qt makes it.

use kx_module_api::ports::Point;
use kx_platform::ui::{ScreenArea, Screens, UiError};
use windows::Win32::Foundation::{LPARAM, POINT, RECT};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITOR_DEFAULTTONEAREST, MONITORINFO,
    MonitorFromRect,
};
use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
use windows::core::BOOL;

use super::{dpi_or_default, is_primary, rect_of};
use crate::{last_refused, refused};

/// The Windows screens: every monitor's work area and the pointer.
#[derive(Clone, Copy, Debug, Default)]
pub struct WinScreens;

impl Screens for WinScreens {
    fn work_areas(&self) -> Vec<ScreenArea> {
        monitors().into_iter().filter_map(area).collect()
    }

    fn cursor(&self) -> Result<Point, UiError> {
        cursor()
    }
}

/// Every monitor, in Windows' order.
#[allow(unsafe_code, reason = "Enumerates the monitors into a local list.")]
fn monitors() -> Vec<HMONITOR> {
    unsafe extern "system" fn collect(mon: HMONITOR, _: HDC, _: *mut RECT, list: LPARAM) -> BOOL {
        let found = std::ptr::with_exposed_provenance_mut::<Vec<HMONITOR>>(list.0.cast_unsigned());
        // SAFETY: `list` is the address of the Vec passed below, alive and not borrowed during the call.
        unsafe { (*found).push(mon) };
        BOOL::from(true)
    }
    let mut found: Vec<HMONITOR> = Vec::new();
    let list = LPARAM(
        std::ptr::from_mut(&mut found)
            .expose_provenance()
            .cast_signed(),
    );
    // SAFETY: the callback touches only `found`, which outlives the call; a failure leaves it as it is.
    let _ = unsafe { EnumDisplayMonitors(None, None, Some(collect), list) };
    found
}

/// What Windows knows about monitor `mon`; `None` when it refuses.
#[allow(unsafe_code, reason = "A monitor query into a local struct.")]
fn info(mon: HMONITOR) -> Option<MONITORINFO> {
    let mut info = MONITORINFO {
        cbSize: u32::try_from(size_of::<MONITORINFO>()).ok()?,
        ..MONITORINFO::default()
    };
    // SAFETY: `info` is a local sized for the call, which fills it in.
    unsafe { GetMonitorInfoW(mon, &raw mut info) }
        .as_bool()
        .then_some(info)
}

/// The effective DPI of monitor `mon`, or the default when Windows cannot say.
#[allow(unsafe_code, reason = "A DPI query into locals.")]
fn monitor_dpi(mon: HMONITOR) -> u32 {
    let (mut x, mut y) = (0, 0);
    // SAFETY: plain query into two locals.
    let ok = unsafe { GetDpiForMonitor(mon, MDT_EFFECTIVE_DPI, &raw mut x, &raw mut y) }.is_ok();
    dpi_or_default(if ok { x } else { 0 })
}

/// Monitor `mon` as a screen area; `None` when Windows refuses.
fn area(mon: HMONITOR) -> Option<ScreenArea> {
    let info = info(mon)?;
    let work = info.rcWork;
    Some(ScreenArea {
        rect: rect_of([work.left, work.top, work.right, work.bottom]),
        dpi: monitor_dpi(mon),
        primary: is_primary(info.dwFlags),
    })
}

/// The monitor nearest to box `r`.
#[allow(unsafe_code, reason = "A plain monitor query.")]
fn nearest(r: &RECT) -> HMONITOR {
    // SAFETY: `r` is a live rect; the nearest monitor always exists.
    unsafe { MonitorFromRect(r, MONITOR_DEFAULTTONEAREST) }
}

/// The work area of the screen nearest to box `r`, as a Windows box.
///
/// # Errors
/// `Refused` when Windows will not describe that screen.
pub fn work_area(r: &RECT) -> Result<RECT, UiError> {
    info(nearest(r)).map(|i| i.rcWork).ok_or_else(last_refused)
}

/// The DPI of the screen nearest to box `r`, or the default when Windows cannot say.
#[must_use]
pub fn dpi_of(r: &RECT) -> u32 {
    monitor_dpi(nearest(r))
}

/// Where the pointer is.
///
/// # Errors
/// `Refused` when Windows cannot say, as on a locked desktop.
#[allow(unsafe_code, reason = "A pointer query into a local.")]
pub fn cursor() -> Result<Point, UiError> {
    let mut p = POINT::default();
    // SAFETY: a plain query into a local.
    unsafe { GetCursorPos(&raw mut p) }.map_err(|e| refused(&e))?;
    Ok(Point { x: p.x, y: p.y })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screen::DEFAULT_DPI;

    #[test]
    fn there_is_one_primary_screen_with_a_real_area() {
        let areas = WinScreens.work_areas();
        assert!(!areas.is_empty());
        assert_eq!(areas.iter().filter(|a| a.primary).count(), 1, "{areas:?}");
        assert!(
            areas
                .iter()
                .all(|a| a.rect.w > 0 && a.rect.h > 0 && a.dpi > 0)
        );
    }

    #[test]
    fn the_nearest_work_area_is_one_of_the_screens() {
        let origin = RECT::default();
        let work = work_area(&origin).unwrap();
        let rect = rect_of([work.left, work.top, work.right, work.bottom]);
        assert!(WinScreens.work_areas().iter().any(|a| a.rect == rect));
        assert!(dpi_of(&origin) >= DEFAULT_DPI / 2, "a real DPI");
    }

    #[test]
    fn the_pointer_is_read_or_refused() {
        match WinScreens.cursor() {
            Ok(_) | Err(UiError::Refused(_)) => {}
            Err(other) => panic!("{other:?}"),
        }
    }
}
