//! Where things are before a copy's snips: target-window's screen and frame, the region and the editor's part in its
//! text box, the keyboard's box and the screen's scale, read once the copy's keyboard shows.
#![cfg(windows)]

use kx_platform_windows::screen;
use serde_json::{Value, json};
use windows::Win32::Foundation::RECT;

use super::judge;
use super::snip::Snip;
use super::wins;
use crate::product::Keyboard;
use crate::win;

/// The boxes G24 snips and judges, in physical pixels.
pub struct Scene {
    /// The whole screen holding target-window.
    pub screen: RECT,
    /// Target-window's frame.
    pub frame: RECT,
    /// The region in its text box.
    pub region: RECT,
    /// The editor's part in its text box, whose size lands between logical pixels.
    pub edit: RECT,
    /// The keyboard's box.
    pub keyboard: RECT,
    /// The screen's DPI.
    pub dpi: u32,
    /// Why the exact checks are skipped, when the screen is not at their scale.
    pub off: Option<String>,
}

/// The box `px` (x, y, width, height in logical px at `dpi`) from the top-left of `inside`.
fn boxed(inside: &RECT, px: [f32; 4], dpi: u32) -> RECT {
    let [left, top, wide, high] = px.map(|v| screen::physical(v, dpi));
    RECT {
        left: inside.left + left,
        top: inside.top + top,
        right: inside.left + left + wide,
        bottom: inside.top + top + high,
    }
}

impl Scene {
    /// The boxes as they stand for the copy of `s`.
    pub fn read(s: &Snip) -> Result<Self, String> {
        let frame = wins::frame(s.app.hwnd)?;
        let text = win::rect(win::focus(s.app.hwnd))?;
        let dpi = screen::dpi_of(&text);
        Ok(Self {
            screen: wins::screen(&frame)?,
            frame,
            region: boxed(&text, s.g.region_px, dpi),
            edit: boxed(&text, s.g.edit_px, dpi),
            keyboard: win::rect(Keyboard::find()?.hwnd)?,
            dpi,
            off: judge::off_scale(dpi, s.g.dpi),
        })
    }

    /// Whether target-window lies clear of the keyboard and holds the region and the editor's part, so neither covers
    /// the other's pixels.
    pub fn apart(&self) -> Value {
        let clear = !wins::meet(&self.frame, &self.keyboard);
        let inside = |r: &RECT| {
            [(r.left, r.top), (r.right - 1, r.bottom - 1)]
                .iter()
                .all(|&(x, y)| {
                    (self.frame.left..self.frame.right).contains(&x)
                        && (self.frame.top..self.frame.bottom).contains(&y)
                })
        };
        let holds = inside(&self.region) && inside(&self.edit);
        json!({ "clear": clear, "holds_region": holds, "ok": clear && holds })
    }
}
