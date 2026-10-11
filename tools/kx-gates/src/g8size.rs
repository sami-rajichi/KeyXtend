//! G8's resizes through the top bar's size buttons: how long each takes, and the presses that size the keyboard back.
#![cfg(windows)]

use std::time::{Duration, Instant};

use serde::Serialize;
use spike_core::clock::now_us;
use windows::Win32::Foundation::HWND;

use crate::apps::Ctx;
use crate::budget::ms;
use crate::g8cfg::G8;
use crate::g8loss::Span;
use crate::keymap::Reader;
use crate::product;
use crate::win::{self, sleep_ms};

/// One resize: the button pressed and the ms until the keyboard's box settled; none when it did not change.
#[derive(Debug, Serialize)]
pub struct Resize {
    /// The button, by UI Automation id.
    pub button: String,
    /// Press to the box's last change.
    pub ms: Option<f64>,
    /// From the press until the box settled or the wait ran out, for G8's frame rule.
    #[serde(skip)]
    pub span: Span,
}

/// One of the top bar's size buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Size {
    /// One size step bigger.
    Grow,
    /// One size step smaller.
    Shrink,
}

impl Size {
    /// Its button's UI Automation id, from `[g8]`.
    pub fn button(self, g: &G8) -> &str {
        match self {
            Self::Grow => &g.grow,
            Self::Shrink => &g.shrink,
        }
    }
}

/// The presses after a grow that end at the first size: a shrink, or, when the grow changed nothing as the keyboard was
/// at its largest, a shrink and a grow.
pub fn after_grow(grew: bool) -> &'static [Size] {
    if grew {
        &[Size::Shrink]
    } else {
        &[Size::Shrink, Size::Grow]
    }
}

/// A press of a button by UI Automation id: `product::invoke`, or `product::restore` for a step that puts back.
type Press = fn(&Reader, &str) -> Result<(), String>;

/// Presses `size`'s button and waits for keyboard window `hwnd` to settle at its new box.
pub fn resize(ctx: &Ctx, reader: &Reader, hwnd: HWND, size: Size) -> Result<Resize, String> {
    resize_with(ctx, reader, hwnd, size, product::invoke)
}

/// `resize`, pressing with `press`.
fn resize_with(
    ctx: &Ctx,
    reader: &Reader,
    hwnd: HWND,
    size: Size,
    press: Press,
) -> Result<Resize, String> {
    let g = &ctx.cfg.g8;
    let id = size.button(g);
    let (wait, settle) = (
        Duration::from_millis(g.resize_wait_ms),
        Duration::from_millis(g.resize_settle_ms),
    );
    let mut last = win::rect(hwnd)?;
    let (start, from_us) = (Instant::now(), now_us());
    press(reader, id)?;
    let mut changed: Option<Duration> = None;
    while start.elapsed() < wait {
        let now = win::rect(hwnd)?;
        if now != last {
            (last, changed) = (now, Some(start.elapsed()));
        } else if changed.is_some_and(|c| start.elapsed().saturating_sub(c) >= settle) {
            break;
        }
        sleep_ms(g.look_ms);
    }
    Ok(Resize {
        button: id.to_string(),
        ms: changed.map(ms),
        span: Span::resize(from_us, now_us()),
    })
}

/// The keyboard window's width, in px.
pub fn width(hwnd: HWND) -> Result<i32, String> {
    win::rect(hwnd).map(|r| r.right - r.left)
}

/// The button that sizes a keyboard `now` px wide back toward `first` px; none within `slack` px.
pub fn mend(first: i32, now: i32, slack: i32) -> Option<Size> {
    if (now - first).abs() <= slack {
        None
    } else if now > first {
        Some(Size::Shrink)
    } else {
        Some(Size::Grow)
    }
}

/// Sizes keyboard `hwnd` back toward `first` px wide when it is off by more than `[g8] resize_slack_px`; the press,
/// which still goes out after the mouse guard stopped the gate.
pub fn size_back(
    ctx: &Ctx,
    reader: &Reader,
    hwnd: HWND,
    first: i32,
) -> Result<Option<Resize>, String> {
    match mend(first, width(hwnd)?, ctx.cfg.g8.resize_slack_px) {
        Some(size) => resize_with(ctx, reader, hwnd, size, product::restore).map(Some),
        None => Ok(None),
    }
}

/// One grow, then the presses of `after_grow` back to the first size.
fn steps(ctx: &Ctx, reader: &Reader, hwnd: HWND) -> Result<Vec<Resize>, String> {
    let mut all = vec![resize(ctx, reader, hwnd, Size::Grow)?];
    for &size in after_grow(all[0].ms.is_some()) {
        all.push(resize(ctx, reader, hwnd, size)?);
    }
    Ok(all)
}

/// G8's `steps`, then one more press if a press silently did nothing or the steps broke off.
pub fn resizes(ctx: &Ctx, hwnd: HWND) -> Result<Vec<Resize>, String> {
    let (reader, first) = (Reader::new()?, width(hwnd)?);
    let steps = steps(ctx, &reader, hwnd);
    let back = size_back(ctx, &reader, hwnd, first);
    let mut all = steps?;
    all.extend(back?);
    Ok(all)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Widths of the keyboard on this PC at 125 %, one size step apart, and a sample slack for DPI rounding.
    const WIDTHS: [i32; 3] = [765, 850, 935];
    const SLACK: i32 = 2;

    #[test]
    fn a_keyboard_off_its_first_width_is_sized_back_toward_it() {
        let [small, first, big] = WIDTHS;
        assert_eq!(mend(first, big, SLACK), Some(Size::Shrink));
        assert_eq!(mend(first, small, SLACK), Some(Size::Grow));
        assert_eq!(mend(first, first, SLACK), None);
        assert_eq!(mend(first, first + 1, SLACK), None, "DPI rounding");
    }

    #[test]
    fn a_grow_that_changed_nothing_is_undone_by_a_shrink_and_a_grow() {
        assert_eq!(after_grow(true), [Size::Shrink]);
        assert_eq!(
            after_grow(false),
            [Size::Shrink, Size::Grow],
            "at the largest size a lone shrink would leave it one step smaller"
        );
    }
}
