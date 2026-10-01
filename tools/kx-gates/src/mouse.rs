//! The frozen face's window and its tools' centres, and real mouse moves and clicks through `SendInput`.
#![cfg(windows)]

use kx_mod_keyboard::Area;
use kx_platform_windows::keys::TAG;
use spike_core::config::SpikeConfig;
use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::Graphics::Gdi::ClientToScreen;
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_MOUSE, MOUSE_EVENT_FLAGS, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN,
    MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MOVE, MOUSEEVENTF_VIRTUALDESK, MOUSEINPUT,
};
use windows::Win32::UI::WindowsAndMessaging::{GetClientRect, USER_DEFAULT_SCREEN_DPI};

/// Largest absolute mouse coordinate; the virtual desktop maps onto 0..=this.
const ABS_MAX: f64 = 65535.0;

/// Where a keyboard window is: client origin in physical pixels, its DPI and client size.
#[derive(Debug, Clone, Copy)]
pub struct Face {
    /// Top-left corner of the client area, in physical pixels.
    pub origin: POINT,
    /// DPI of the face window.
    pub dpi: u32,
    /// Client width and height, in physical pixels.
    pub client: (i32, i32),
}

/// `v` rounded to a whole pixel.
#[allow(
    clippy::cast_possible_truncation,
    reason = "Screen coordinates are far inside `i32`."
)]
fn round_px(v: f64) -> i32 {
    v.round() as i32
}

/// The physical screen centre of `place` for a face at `origin` and `dpi`.
pub fn centre(place: &Area, origin: POINT, dpi: u32) -> POINT {
    let scale = f64::from(dpi) / f64::from(USER_DEFAULT_SCREEN_DPI);
    let mid = |start: f64, size: f64| round_px((start + size / 2.0) * scale);
    POINT {
        x: origin.x + mid(place.x, place.w),
        y: origin.y + mid(place.y, place.h),
    }
}

/// The running face window, found by its exact title from spike.toml; `name` only fills the error.
pub fn find_face(spike: &SpikeConfig, name: &str) -> Result<HWND, String> {
    let title = &spike.keyboard.title;
    crate::win::top_windows()
        .into_iter()
        .find(|&w| crate::win::title(w) == *title)
        .ok_or_else(|| format!("face window {title:?} not found; start the {name} face first"))
}

/// Reads where a keyboard window's client area is and its DPI.
#[allow(unsafe_code, reason = "Queries into locals on a live window.")]
pub fn face(hwnd: HWND) -> Result<Face, String> {
    let mut origin = POINT::default();
    let mut rect = RECT::default();
    // SAFETY: plain queries into locals on a live window handle.
    let dpi = unsafe {
        if !ClientToScreen(hwnd, &raw mut origin).as_bool() {
            return Err("ClientToScreen failed".to_string());
        }
        GetClientRect(hwnd, &raw mut rect).map_err(|e| format!("GetClientRect: {e}"))?;
        GetDpiForWindow(hwnd)
    };
    if dpi == 0 {
        return Err("GetDpiForWindow failed".to_string());
    }
    Ok(Face {
        origin,
        dpi,
        client: (rect.right - rect.left, rect.bottom - rect.top),
    })
}

/// Maps a pixel to the 0..=65535 absolute range over `start..start+size`.
pub fn normalise(v: i32, start: i32, size: i32) -> i32 {
    round_px(f64::from(v - start) * ABS_MAX / f64::from((size - 1).max(1)))
}

/// A mouse input at absolute (`dx`, `dy`) with `flags`, marked `tag`.
pub fn input(dx: i32, dy: i32, flags: MOUSE_EVENT_FLAGS, tag: usize) -> INPUT {
    INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx,
                dy,
                mouseData: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: tag,
            },
        },
    }
}

/// A wheel turn of `delta` on the wheel `flags` names, marked `tag`; it acts where the pointer is.
pub fn wheel(delta: i32, flags: MOUSE_EVENT_FLAGS, tag: usize) -> INPUT {
    INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: 0,
                dy: 0,
                // Windows reads the turn as a signed number in this unsigned field.
                mouseData: delta.cast_unsigned(),
                dwFlags: flags,
                time: 0,
                dwExtraInfo: tag,
            },
        },
    }
}

/// Move, press and release at the absolute spot (x, y) of the 0..=65535 desktop range.
fn click_events(x: i32, y: i32) -> [(i32, i32, MOUSE_EVENT_FLAGS); 3] {
    let at = MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK;
    [
        (x, y, at),
        (x, y, at | MOUSEEVENTF_LEFTDOWN),
        (x, y, at | MOUSEEVENTF_LEFTUP),
    ]
}

/// Moves the pointer to `p` (physical pixels) without pressing: the first of a click's events.
pub fn move_to(p: POINT) -> Result<(), String> {
    let [moved, ..] = events_at(p);
    send(&[moved])
}

/// Clicks at `p` (physical pixels) in one `SendInput` batch, which a hand on the mouse cannot split.
pub fn click_at(p: POINT) -> Result<(), String> {
    send(&events_at(p))
}

/// Presses the left button at `p` (physical pixels) and holds it.
pub fn press_at(p: POINT) -> Result<(), String> {
    let [moved, down, _] = events_at(p);
    send(&[moved, down])
}

/// Moves to `p` (physical pixels) and lets the left button go there.
pub fn release_at(p: POINT) -> Result<(), String> {
    let [moved, _, up] = events_at(p);
    send(&[moved, up])
}

/// A click's events at `p` (physical pixels), in the 0..=65535 absolute range of the desktop.
fn events_at(p: POINT) -> [(i32, i32, MOUSE_EVENT_FLAGS); 3] {
    let (left, top, width, height) = crate::win::desktop();
    click_events(normalise(p.x, left, width), normalise(p.y, top, height))
}

/// Sends `events`, each marked as ours, in one batch.
fn send(events: &[(i32, i32, MOUSE_EVENT_FLAGS)]) -> Result<(), String> {
    let inputs: Vec<INPUT> = events
        .iter()
        .map(|&(x, y, flags)| input(x, y, flags, TAG))
        .collect();
    crate::keys::send(&inputs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centres_scale_to_dpi_144() {
        let place = Area {
            x: 4.0,
            y: 4.0,
            w: 48.0,
            h: 48.0,
        };
        let origin = POINT { x: 100, y: 200 };
        let p = centre(&place, origin, 144);
        assert_eq!((p.x, p.y), (142, 242));
        let p = centre(&place, origin, 96);
        assert_eq!((p.x, p.y), (128, 228));
    }

    #[test]
    fn normalise_maps_the_desktop_edges() {
        assert_eq!(normalise(0, 0, 1920), 0);
        assert_eq!(normalise(1919, 0, 1920), 65535);
        assert_eq!(normalise(-1920, -1920, 3840), 0);
    }

    #[test]
    fn a_click_is_move_press_release_at_one_absolute_spot() {
        let at = MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK;
        assert_eq!(
            click_events(100, 200),
            [
                (100, 200, at),
                (100, 200, at | MOUSEEVENTF_LEFTDOWN),
                (100, 200, at | MOUSEEVENTF_LEFTUP)
            ]
        );
    }
}
