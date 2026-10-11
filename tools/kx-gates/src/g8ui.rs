//! G8's on-screen timings, read off the screen: how soon a pressed key lights up.
//! Each press is let go away from its key, so no click lands and nothing is typed.
#![cfg(windows)]

use std::time::{Duration, Instant};

use serde::Serialize;
use spike_core::clock::now_us;
use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::Graphics::Dwm::{DWM_TIMING_INFO, DwmGetCompositionTimingInfo};

use crate::apps::Ctx;
use crate::budget::ms;
use crate::g8loss::Span;
use crate::product::Keyboard;
use crate::win::{self, size32, sleep_ms};
use crate::{mouse, shot, simuser};

/// What the presses measured, in ms.
#[derive(Debug, Default, Serialize)]
pub struct Presses {
    /// Press to light, per press; none when the key never lit.
    pub lit_ms: Vec<Option<f64>>,
    /// How long each screen copy took.
    pub copy_ms: Vec<f64>,
    /// Each press and its key's rise, for G8's frame rule.
    #[serde(skip)]
    pub spans: Vec<Span>,
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
pub fn away_from(kb: RECT, on: POINT, desktop_top: i32) -> POINT {
    let y = if kb.top > desktop_top {
        kb.top - 1
    } else {
        kb.bottom
    };
    POINT { x: on.x, y }
}

/// `released`, the result of letting the button go; on an error `release` lets every button go, so none stays down.
fn or_release(
    released: Result<(), String>,
    release: impl FnOnce() -> Result<bool, String>,
) -> Result<(), String> {
    released.map_err(|e| format!("{e}; buttons let go: {:?}", release()))
}

/// Presses `[g8] press_key` again and again and times how soon it lights; the pointer rests on it first, so the
/// hover is already drawn, and the last key's rise ends before it returns.
pub fn presses(ctx: &Ctx, kb: &Keyboard) -> Result<Presses, String> {
    let g = &ctx.cfg.g8;
    let (r, on) = (kb.rect(&g.press_key)?, kb.at(&g.press_key)?);
    let away = away_from(win::rect(kb.hwnd)?, on, win::desktop().1);
    let mut out = Presses::default();
    for _ in 0..g.presses {
        mouse::move_to(on)?;
        sleep_ms(g.hover_settle_ms);
        let before = picture(&r)?;
        let (start, at_us) = (Instant::now(), now_us());
        mouse::press_at(on)?;
        out.spans.push(Span::press(at_us, g.rise_ms));
        let lit = watch(&r, &before, start, g.press_wait_ms, &mut out.copy_ms);
        or_release(mouse::release_at(away), simuser::release_all)?;
        out.lit_ms.push(lit?);
    }
    sleep_ms(g.rise_wait_ms);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_failed_release_lets_every_button_go_and_says_so() {
        let mut freed = false;
        let e = or_release(Err("SendInput refused".into()), || {
            freed = true;
            Ok(true)
        })
        .expect_err("the release failed");
        assert!(freed && e.contains("SendInput refused"), "{e}");
        let mut called = false;
        let ok = or_release(Ok(()), || {
            called = true;
            Ok(true)
        });
        assert!(
            ok.is_ok() && !called,
            "a release that worked needs nothing more"
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
