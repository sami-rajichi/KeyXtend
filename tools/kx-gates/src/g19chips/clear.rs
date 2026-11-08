//! What the strip keeps clear of, as kx-ui's `SelStrip.boxes()` lists it: the keyboard's plate while it shows, from
//! its window and its resize corner, else the minimise bubble; and the keyboard's size, which the gap grows with.
//! Our panels' panes are left out, as no strip shows while one is open.
#![cfg(windows)]

use std::cell::Cell;

use kx_module_api::ports::{Point, Rect};
use kx_platform_windows::screen;
use serde_json::{Value, json};
use windows::Win32::Foundation::{HWND, POINT};
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromPoint,
};

use crate::apps::Ctx;
use crate::g19chips::place;
use crate::keymap::Reader;
use crate::product::{self, Keyboard};
use crate::win::{self, size32};

/// The top-left of the whole screen nearest `p`, which Qt keeps as that screen's logical origin too.
#[allow(unsafe_code, reason = "A monitor query into a local.")]
pub fn origin(p: Point) -> Option<(i32, i32)> {
    let mut info = MONITORINFO {
        cbSize: size32::<MONITORINFO>(),
        ..Default::default()
    };
    // SAFETY: plain queries; `info` is a local with `cbSize` set, and the nearest monitor always exists.
    let ok = unsafe {
        let mon = MonitorFromPoint(POINT { x: p.x, y: p.y }, MONITOR_DEFAULTTONEAREST);
        GetMonitorInfoW(mon, &raw mut info).as_bool()
    };
    ok.then_some((info.rcMonitor.left, info.rcMonitor.top))
}

/// The boxes and the size one placed case is judged with.
pub struct Clear {
    /// The plate's or the bubble's box, in physical px.
    pub boxes: Vec<Rect>,
    /// The keyboard's size.
    pub size: f64,
    /// What the boxes are, for the result line.
    pub note: Value,
}

/// A reader of the test build's keyboard, which keeps the last size it read for a minimised keyboard.
pub struct Board<'a> {
    ctx: &'a Ctx<'a>,
    pid: u32,
    corner_px: f32,
    size: Cell<f64>,
}

impl<'a> Board<'a> {
    /// A reader of process `pid`'s keyboard, with the theme's corner width and the size the data folder starts with.
    pub fn new(ctx: &'a Ctx<'a>, pid: u32, corner_px: f32, size: f64) -> Self {
        Self {
            ctx,
            pid,
            corner_px,
            size: Cell::new(size),
        }
    }

    /// The plate and the size, while the keyboard shows with its corner.
    fn plate(&self) -> Option<(Rect, f64)> {
        let kb = Keyboard::find().ok()?;
        let corner = place::rect(&kb.rect(&self.ctx.cfg.g19_product.corner).ok()?);
        let r = win::rect(kb.hwnd).ok()?;
        let size = place::size_of(corner.w, self.corner_px, screen::dpi_of(&r));
        Some((place::plate(place::rect(&r), corner), size))
    }

    /// The bubble's window box, while it shows.
    fn bubble(&self, reader: &Reader) -> Option<Rect> {
        let id = &self.ctx.cfg.g9.bring_back;
        let holds = |w: &HWND| reader.read(*w).is_ok_and(|k| k.rect(id).is_some());
        let w = product::shown_by(self.pid).into_iter().find(holds)?;
        win::rect(w).ok().map(|r| place::rect(&r))
    }

    /// What the strip keeps clear of now.
    pub fn clear(&self, reader: &Reader) -> Clear {
        if let Some((plate, size)) = self.plate() {
            self.size.set(size);
            let note = json!({ "plate": [plate.x, plate.y, plate.w, plate.h], "size": size });
            return Clear {
                boxes: vec![plate],
                size,
                note,
            };
        }
        let size = self.size.get();
        let bubble = self.bubble(reader);
        let note = json!({ "bubble": bubble.map(|b| [b.x, b.y, b.w, b.h]), "size": size });
        Clear {
            boxes: bubble.into_iter().collect(),
            size,
            note,
        }
    }
}
