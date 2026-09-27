//! Key centres on screen from spike.toml, and real mouse moves and clicks through `SendInput`.

use spike_core::config::{KeyboardConfig, SpikeConfig};
use spike_core::place::{Place, places};
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

/// Where the face is: client origin in physical pixels, its DPI and client size.
#[derive(Debug, Clone, Copy)]
pub struct Face {
    /// Top-left corner of the client area, in physical pixels.
    pub origin: POINT,
    /// DPI of the face window.
    pub dpi: u32,
    /// Client width and height, in physical pixels.
    pub client: (i32, i32),
}

/// Every key's scan code and box, row by row, as the faces lay them out.
pub fn key_places(kb: &KeyboardConfig) -> Vec<(u32, Place)> {
    let codes = kb.rows.iter().flatten().copied();
    codes.zip(places(kb).into_iter().flatten()).collect()
}

/// The physical screen centre of `place` for a face at `origin` and `dpi`.
pub fn centre(place: &Place, origin: POINT, dpi: u32) -> POINT {
    let scale = f64::from(dpi) / f64::from(USER_DEFAULT_SCREEN_DPI);
    let mid =
        |start: f32, size: f32| ((f64::from(start) + f64::from(size) / 2.0) * scale).round() as i32;
    POINT {
        x: origin.x + mid(place.x, place.w),
        y: origin.y + mid(place.y, place.h),
    }
}

/// The running face called `name`, found by its exact title from spike.toml.
pub fn find_face(spike: &SpikeConfig, name: &str) -> Result<HWND, String> {
    let title = spike
        .keyboard
        .titles
        .get(name)
        .ok_or_else(|| format!("unknown face {name}"))?;
    crate::win::top_windows()
        .into_iter()
        .find(|&w| crate::win::title(w) == *title)
        .ok_or_else(|| format!("face window {title:?} not found; start the {name} face first"))
}

/// Reads where the face window's client area is and its DPI.
pub fn face(hwnd: HWND) -> Result<Face, String> {
    let mut origin = POINT::default();
    let mut rect = RECT::default();
    // SAFETY: plain queries into locals on a live window handle.
    let dpi = unsafe {
        if !ClientToScreen(hwnd, &mut origin).as_bool() {
            return Err("ClientToScreen failed".to_string());
        }
        GetClientRect(hwnd, &mut rect).map_err(|e| format!("GetClientRect: {e}"))?;
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
    (f64::from(v - start) * ABS_MAX / f64::from((size - 1).max(1))).round() as i32
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

/// Move, press and release at the absolute spot (x, y) of the 0..=65535 desktop range.
fn click_events(x: i32, y: i32) -> [(i32, i32, MOUSE_EVENT_FLAGS); 3] {
    let at = MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK;
    [
        (x, y, at),
        (x, y, at | MOUSEEVENTF_LEFTDOWN),
        (x, y, at | MOUSEEVENTF_LEFTUP),
    ]
}

/// Clicks at `p` (physical pixels) in one `SendInput` batch, which a hand on the mouse cannot split.
pub fn click_at(p: POINT) -> Result<(), String> {
    let (x0, y0, w, h) = crate::win::desktop();
    let events = click_events(normalise(p.x, x0, w), normalise(p.y, y0, h));
    let tag = spike_core::inject::TAG;
    spike_core::inject::send(&events.map(|(x, y, f)| input(x, y, f, tag)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kb() -> KeyboardConfig {
        KeyboardConfig {
            titles: Default::default(),
            key_px: 48.0,
            gap_px: 4.0,
            font_px: 16.0,
            guard_delay_ms: 0,
            guard_tries: 1,
            relabel_ms: 0,
            rows: vec![vec![0x29, 0x02], vec![0x0F, 0x10], vec![0x39, 0xE038]],
            widths: vec![(0x0F, 1.5), (0x39, 6.25)],
        }
    }

    #[test]
    fn key_places_pair_each_code_with_its_box() {
        let got: Vec<(u32, f32, f32)> = key_places(&kb())
            .into_iter()
            .map(|(code, p)| (code, p.x, p.y))
            .collect();
        assert_eq!(got[0], (0x29, 4.0, 4.0));
        assert_eq!(got[2], (0x0F, 4.0, 56.0));
        assert_eq!(got[5], (0xE038, 329.0, 108.0));
        assert_eq!(got.len(), 6);
    }

    #[test]
    fn centres_scale_to_dpi_144() {
        let keys = key_places(&kb());
        let origin = POINT { x: 100, y: 200 };
        let got: Vec<(i32, i32)> = keys
            .iter()
            .map(|(_, p)| centre(p, origin, 144))
            .map(|p| (p.x, p.y))
            .collect();
        assert_eq!(
            got,
            vec![
                (142, 242),
                (220, 242),
                (162, 320),
                (259, 320),
                (347, 398),
                (630, 398)
            ]
        );
        let p = centre(&keys[0].1, origin, 96);
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
