//! The usable screen around a box: its monitor's work area, without the taskbar.

use windows::Win32::Foundation::RECT;
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromRect,
};

/// The work area of the monitor nearest to `r`; physical pixels in a per-monitor DPI-aware thread, like UIA boxes.
pub fn work_area(r: &RECT) -> Result<RECT, String> {
    let mut info = MONITORINFO {
        cbSize: size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    // SAFETY: plain queries; `info` is sized for the call.
    unsafe {
        let mon = MonitorFromRect(r, MONITOR_DEFAULTTONEAREST);
        if !GetMonitorInfoW(mon, &mut info).as_bool() {
            return Err("GetMonitorInfo failed".to_string());
        }
    }
    Ok(info.rcWork)
}
