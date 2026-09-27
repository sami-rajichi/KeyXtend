//! The running face's tool buttons, pill and overlay: found by title, clicked only where they are on top.

use serde_json::{Value, json};
use spike_core::config::ToolButton;
use spike_core::hold::Pt;
use spike_core::place::{self, Place};
use spike_core::screen;
use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::UI::WindowsAndMessaging::{
    SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER, SetWindowPos,
};

use crate::apps::Ctx;
use crate::mouse::{self, Face};
use crate::win;

/// A running face window and where it is.
pub struct FaceWin {
    /// The face's window.
    pub hwnd: HWND,
    /// Its client origin and DPI.
    pub face: Face,
}

impl FaceWin {
    /// The running face called `name`.
    pub fn find(ctx: &Ctx, name: &str) -> Result<FaceWin, String> {
        let hwnd = mouse::find_face(ctx.spike, name)?;
        Ok(FaceWin {
            hwnd,
            face: mouse::face(hwnd)?,
        })
    }

    /// The running face `name`, moved to the bottom-right corner so it covers no test app, as a docked keyboard would.
    pub fn parked(ctx: &Ctx, name: &str) -> Result<FaceWin, String> {
        let hwnd = mouse::find_face(ctx.spike, name)?;
        let r = win::rect(hwnd)?;
        let p = corner(&r, &screen::work_area(&r)?);
        // SAFETY: plain call on our own face window; a failure is returned.
        unsafe {
            SetWindowPos(
                hwnd,
                None,
                p.x,
                p.y,
                0,
                0,
                SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
            )
        }
        .map_err(|e| format!("moving the face: {e}"))?;
        FaceWin::find(ctx, name)
    }

    /// The screen centre of tool button `b`.
    pub fn button(&self, ctx: &Ctx, b: ToolButton) -> POINT {
        let places = place::tools(&ctx.spike.keyboard, ToolButton::ALL.len());
        let at = places.get(b.index()).copied().unwrap_or(Place {
            x: 0.0,
            y: 0.0,
            w: 0.0,
            h: 0.0,
        });
        mouse::centre(&at, self.face.origin, self.face.dpi)
    }

    /// A spot on the status line, which types nothing; a click there gives the face the last input, as a user's click would.
    pub fn quiet_spot(&self, ctx: &Ctx) -> POINT {
        let kb = &ctx.spike.keyboard;
        let (_, top) = place::face_size(kb);
        let line = Place {
            x: kb.gap_px,
            y: top,
            w: kb.key_px,
            h: kb.font_px,
        };
        mouse::centre(&line, self.face.origin, self.face.dpi)
    }

    /// Clicks tool button `b`.
    pub fn press(&self, ctx: &Ctx, b: ToolButton) -> Result<(), String> {
        click_on(self.hwnd, self.button(ctx, b))
    }
}

/// Where window `face` goes to sit in the bottom-right corner of `area`.
fn corner(face: &RECT, area: &RECT) -> POINT {
    POINT {
        x: area.right - (face.right - face.left),
        y: area.bottom - (face.bottom - face.top),
    }
}

/// `p` as a Win32 point.
pub fn at(p: Pt) -> POINT {
    POINT { x: p.x, y: p.y }
}

/// Clicks `p` only when `owner` is the window on top there, so no other window is ever clicked.
pub fn click_on(owner: HWND, p: POINT) -> Result<(), String> {
    let top = win::root_at(p);
    if top != owner {
        return Err(format!("{} covers {},{}", win::describe(top), p.x, p.y));
    }
    mouse::click_at(p)
}

/// Closes the running face `name` and waits until its window is gone; needs the uiAccess harness.
pub fn close(ctx: &Ctx, name: &str) -> Result<Value, String> {
    let hwnd = mouse::find_face(ctx.spike, name)?;
    win::close(hwnd)?;
    let t = &ctx.cfg.timing;
    let closed = win::wait_gone(hwnd, t.close_wait_ms, t.poll_ms);
    Ok(json!({ "gate": "close", "face": name, "pass": closed }))
}

/// The window called `title` while it shows: visible and not cloaked.
pub fn shown(title: &str) -> Option<HWND> {
    win::seen_windows()
        .into_iter()
        .find(|&w| win::title(w) == title)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_face_parks_in_the_bottom_right_corner_of_the_work_area() {
        let face = RECT {
            left: 128,
            top: 128,
            right: 928,
            bottom: 508,
        };
        let area = RECT {
            left: 0,
            top: 0,
            right: 1920,
            bottom: 1032,
        };
        let p = corner(&face, &area);
        assert_eq!((p.x, p.y), (1120, 652));
    }
}
