//! The copy's dictation bar: found by Stop's gate id among the copy's shown windows, its look checked as the voice
//! design asks (never activated, on top, above the keyboard and beside it), and Stop clicked with the real pointer.
#![cfg(windows)]

use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::UI::WindowsAndMessaging::{WS_EX_NOACTIVATE, WS_EX_TOPMOST};

use crate::apps::Opened;
use crate::g23::live;
use crate::keymap::Reader;
use crate::ownkeys::{self, Real};
use crate::{g9win, mouse, mouseguard, product, win, winfind};

/// The bar's look beside the keyboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "Each flag is one part of the look the gate prints and judges."
)]
pub(super) struct BarLook {
    /// It never takes the activation.
    pub(super) no_activate: bool,
    /// It sits in the topmost band.
    pub(super) topmost: bool,
    /// It is above the keyboard in the z-order.
    pub(super) above: bool,
    /// Its box covers none of the keyboard's.
    pub(super) apart: bool,
    /// No window lies over its box.
    pub(super) uncovered: bool,
}

impl BarLook {
    /// The look of the bar `(key, box)` with extended style `ex` beside the keyboard `(key, box)`, by the z-order
    /// `order`, topmost first, where `top` is the window on top over the bar's box.
    pub(super) fn of(
        ex: u32,
        order: &[isize],
        bar: (isize, RECT),
        kb: (isize, RECT),
        top: Option<isize>,
    ) -> Self {
        let at = |k: isize| order.iter().position(|&w| w == k);
        let above = matches!((at(bar.0), at(kb.0)), (Some(b), Some(k)) if b < k);
        let (b, k) = (bar.1, kb.1);
        let overlap = b.left < k.right && k.left < b.right && b.top < k.bottom && k.top < b.bottom;
        Self {
            no_activate: ex & WS_EX_NOACTIVATE.0 != 0,
            topmost: ex & WS_EX_TOPMOST.0 != 0,
            above,
            apart: !overlap,
            uncovered: top == Some(bar.0),
        }
    }

    /// Every part of the look holds.
    pub(super) fn ok(self) -> bool {
        self.no_activate && self.topmost && self.above && self.apart && self.uncovered
    }
}

/// The copy's bar: process `pid`'s shown window, other than the keyboard `kb`, holding Stop's gate id `id`, and Stop's
/// box; none while no such window shows.
pub(super) fn find(
    reader: &Reader,
    pid: u32,
    kb: HWND,
    id: &str,
) -> Result<Option<(HWND, RECT)>, String> {
    for w in product::shown_by(pid).into_iter().filter(|&w| w != kb) {
        let boxes = live::kept(live::boxes(reader, w, id), || live::shown(w))?;
        if let Some(stop) = boxes.and_then(|b| b.first().copied()) {
            return Ok(Some((w, stop)));
        }
    }
    Ok(None)
}

/// The bar's look now, beside the keyboard `kb`.
pub(super) fn look(bar: HWND, kb: HWND) -> Result<BarLook, String> {
    let order: Vec<isize> = win::seen_windows().into_iter().map(win::key).collect();
    let (b, k) = (
        (win::key(bar), win::rect(bar)?),
        (win::key(kb), win::rect(kb)?),
    );
    let top = winfind::over(bar).map(win::key);
    Ok(BarLook::of(g9win::ex_of(bar), &order, b, k, top))
}

/// Clicks the middle of Stop's box `stop` on `bar`, only while `app` is in front and the bar is on top there.
pub(super) fn stop(app: &Opened, bar: HWND, stop: RECT) -> Result<(), String> {
    let at = POINT {
        x: stop.left + (stop.right - stop.left) / 2,
        y: stop.top + (stop.bottom - stop.top) / 2,
    };
    ownkeys::click(&Real, app, &|| {
        mouseguard::allowed()?;
        let top = win::root_at(at);
        if top != bar {
            return Err(format!("{} covers Stop", win::describe(top)));
        }
        mouse::click_at(at)
    })
}
