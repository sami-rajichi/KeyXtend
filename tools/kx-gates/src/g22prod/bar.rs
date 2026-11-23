//! The copy's dictation bar: found by Stop's gate id among the copy's shown windows, never a click-through one, its
//! look checked as the voice design asks (never activated, on top, above the keyboard and beside it), and Stop clicked
//! with the real pointer.
#![cfg(windows)]

use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::UI::WindowsAndMessaging::{WS_EX_NOACTIVATE, WS_EX_TOPMOST, WS_EX_TRANSPARENT};

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
    let shown = product::shown_by(pid).into_iter().filter(|&w| w != kb);
    let windows: Vec<(HWND, u32)> = shown.map(|w| (w, g9win::ex_of(w))).collect();
    chosen(&windows, |w| {
        let boxes = live::kept(live::boxes(reader, w, id), || live::shown(w))?;
        Ok(boxes.and_then(|b| b.first().copied()))
    })
}

/// The first of `windows`, each a key and its extended style, topmost first, that is not click-through and on which
/// `holds` finds Stop, with what it found; a click-through window, such as the bar's shadow, is never asked.
pub(super) fn chosen<K: Copy, T, E>(
    windows: &[(K, u32)],
    mut holds: impl FnMut(K) -> Result<Option<T>, E>,
) -> Result<Option<(K, T)>, E> {
    for &(w, ex) in windows {
        if ex & WS_EX_TRANSPARENT.0 != 0 {
            continue;
        }
        if let Some(found) = holds(w)? {
            return Ok(Some((w, found)));
        }
    }
    Ok(None)
}

/// The bar's look now, beside the keyboard `kb`, and the class of the window over its box when one covers it.
pub(super) fn look(bar: HWND, kb: HWND) -> Result<(BarLook, Option<String>), String> {
    let order: Vec<isize> = win::seen_windows().into_iter().map(win::key).collect();
    let (b, k) = (
        (win::key(bar), win::rect(bar)?),
        (win::key(kb), win::rect(kb)?),
    );
    let over = winfind::over(bar);
    let look = BarLook::of(g9win::ex_of(bar), &order, b, k, over.map(win::key));
    Ok((look, top_class(look, || over.map(win::class))))
}

/// The class `class` reads of the window over the bar's box, only when `look` says one covers it.
pub(super) fn top_class(look: BarLook, class: impl FnOnce() -> Option<String>) -> Option<String> {
    (!look.uncovered).then(class).flatten()
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
