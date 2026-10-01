//! G8's on-screen timings, read off the screen: how soon a pressed key lights up, and how long a resize takes.
//! Each press is let go away from its key, so no click lands and nothing is typed.
#![cfg(windows)]

use std::time::{Duration, Instant};

use serde::Serialize;
use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::Graphics::Dwm::{DWM_TIMING_INFO, DwmGetCompositionTimingInfo};

use crate::apps::Ctx;
use crate::budget::ms;
use crate::g8cfg::G8;
use crate::keymap::Reader;
use crate::product::{self, Keyboard};
use crate::win::{self, size32, sleep_ms};
use crate::{mouse, shot};

/// What the presses measured, in ms.
#[derive(Debug, Default, Serialize)]
pub struct Presses {
    /// Press to light, per press; none when the key never lit.
    pub lit_ms: Vec<Option<f64>>,
    /// How long each screen copy took.
    pub copy_ms: Vec<f64>,
}

/// One resize: the button pressed and the ms until the keyboard's box settled; none when it did not change.
#[derive(Debug, Serialize)]
pub struct Resize {
    /// The button, by UI Automation id.
    pub button: String,
    /// Press to the box's last change.
    pub ms: Option<f64>,
}

/// The screen's refresh rate in Hz, from the compositor.
#[allow(unsafe_code, reason = "Fills a local with its size set.")]
pub fn refresh_hz() -> Option<f64> {
    let mut info = DWM_TIMING_INFO {
        cbSize: size32::<DWM_TIMING_INFO>(),
        ..Default::default()
    };
    // SAFETY: `info` is a live local with `cbSize` set, as the call needs.
    unsafe { DwmGetCompositionTimingInfo(HWND::default(), &raw mut info) }.ok()?;
    let rate = info.rateRefresh;
    (rate.uiDenominator > 0).then(|| f64::from(rate.uiNumerator) / f64::from(rate.uiDenominator))
}

/// The pixels of screen box `r`.
fn picture(r: &RECT) -> Result<Vec<u8>, String> {
    Ok(shot::of_box(r)?.bgra)
}

/// Copies box `r` until it differs from `before`, adding each copy's time to `copies`; the ms from `start`.
fn watch(
    r: &RECT,
    before: &[u8],
    start: Instant,
    wait_ms: u64,
    copies: &mut Vec<f64>,
) -> Result<Option<f64>, String> {
    let wait = Duration::from_millis(wait_ms);
    loop {
        let copy = Instant::now();
        let now = picture(r)?;
        copies.push(ms(copy.elapsed()));
        if now != before {
            return Ok(Some(ms(start.elapsed())));
        }
        if start.elapsed() >= wait {
            return Ok(None);
        }
    }
}

/// A point just above the keyboard's box, else just below it, where letting go clicks nothing.
fn away_from(kb: RECT, on: POINT, desktop_top: i32) -> POINT {
    let y = if kb.top > desktop_top {
        kb.top - 1
    } else {
        kb.bottom
    };
    POINT { x: on.x, y }
}

/// Presses `[g8] press_key` again and again and times how soon it lights; the pointer rests on it first, so the
/// hover is already drawn.
pub fn presses(ctx: &Ctx, kb: &Keyboard) -> Result<Presses, String> {
    let g = &ctx.cfg.g8;
    let (r, on) = (kb.rect(&g.press_key)?, kb.at(&g.press_key)?);
    let away = away_from(win::rect(kb.hwnd)?, on, win::desktop().1);
    let mut out = Presses::default();
    for _ in 0..g.presses {
        mouse::move_to(on)?;
        sleep_ms(g.hover_settle_ms);
        let before = picture(&r)?;
        let start = Instant::now();
        mouse::press_at(on)?;
        let lit = watch(&r, &before, start, g.press_wait_ms, &mut out.copy_ms);
        mouse::release_at(away)?;
        out.lit_ms.push(lit?);
    }
    Ok(out)
}

/// One of the top bar's size buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Size {
    Grow,
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

/// Presses `size`'s button and waits for keyboard window `hwnd` to settle at its new box.
pub fn resize(ctx: &Ctx, reader: &Reader, hwnd: HWND, size: Size) -> Result<Resize, String> {
    let g = &ctx.cfg.g8;
    let id = size.button(g);
    let (wait, settle) = (
        Duration::from_millis(g.resize_wait_ms),
        Duration::from_millis(g.resize_settle_ms),
    );
    let mut last = win::rect(hwnd)?;
    let start = Instant::now();
    product::invoke(reader, id)?;
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
    })
}

/// One grow, then the presses of `after_grow` back to the first size.
pub fn resizes(ctx: &Ctx, hwnd: HWND) -> Result<Vec<Resize>, String> {
    let reader = Reader::new()?;
    let grew = resize(ctx, &reader, hwnd, Size::Grow)?;
    let back = after_grow(grew.ms.is_some());
    let mut all = vec![grew];
    for &size in back {
        all.push(resize(ctx, &reader, hwnd, size)?);
    }
    Ok(all)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_grow_that_changed_nothing_is_undone_by_a_shrink_and_a_grow() {
        assert_eq!(after_grow(true), [Size::Shrink]);
        assert_eq!(
            after_grow(false),
            [Size::Shrink, Size::Grow],
            "at the largest size a lone shrink would leave it one step smaller"
        );
    }

    #[test]
    fn letting_go_happens_above_the_keyboard_or_below_it_at_the_top_of_the_screen() {
        let kb = RECT {
            left: 540,
            top: 775,
            right: 1390,
            bottom: 1032,
        };
        let on = POINT { x: 641, y: 913 };
        assert_eq!(away_from(kb, on, 0), POINT { x: 641, y: 774 });
        let top = RECT { top: 0, ..kb };
        assert_eq!(away_from(top, on, 0), POINT { x: 641, y: 1032 });
    }
}
