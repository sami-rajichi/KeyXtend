//! G8's on-screen timings, read off the screen: how soon a pressed key lights up.
//! Each press types its key's character at the press (ADR-0024) into the window G8 keeps in front, and is let go away
//! from the key.
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
use crate::{mouse, mouseguard, shot, simuser};

/// Where G8 presses: the key's screen box and centre, and its keyboard window's box, in physical pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spot {
    /// The key's box, which G8 copies to see it light.
    pub key: RECT,
    /// Where the pointer presses it.
    pub at: POINT,
    /// The keyboard window's box, which the release goes past.
    pub window: RECT,
}

impl Spot {
    /// Key `id` of the product keyboard `kb`.
    pub fn of(kb: &Keyboard, id: &str) -> Result<Self, String> {
        Ok(Self {
            key: kb.rect(id)?,
            at: kb.at(id)?,
            window: win::rect(kb.hwnd)?,
        })
    }
}

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
    /// How many characters reached target-window.
    pub arrived: usize,
    /// True when exactly one press key character per press arrived there.
    pub typed_ok: bool,
    /// What the clean-up left open.
    pub left_open: Vec<String>,
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

/// Looks with `lit` until the key lights, `wait` from `start` is over, or `go_on` refuses because the mouse guard
/// stopped the gate; the ms from `start` to the light.
fn watch(
    start: Instant,
    wait: Duration,
    mut lit: impl FnMut() -> Result<bool, String>,
    go_on: impl Fn() -> Result<(), String>,
) -> Result<Option<f64>, String> {
    loop {
        go_on()?;
        if lit()? {
            return Ok(Some(ms(start.elapsed())));
        }
        if start.elapsed() >= wait {
            return Ok(None);
        }
    }
}

/// Copies box `r` and says whether it differs from `before`, adding the copy's time to `copies`.
fn changed(r: &RECT, before: &[u8], copies: &mut Vec<f64>) -> Result<bool, String> {
    let copy = Instant::now();
    let now = picture(r)?;
    copies.push(ms(copy.elapsed()));
    Ok(now != before)
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

/// Presses the key at `spot` `[g8] presses` times and times how soon it lights; the pointer rests on it first, so the
/// hover is already drawn, and `front` keeps the window it types into in front before each press. The last key's rise
/// ends before it returns, and a stop by the mouse guard lets go away from the key at once.
pub fn presses(
    ctx: &Ctx,
    spot: &Spot,
    front: impl Fn() -> Result<(), String>,
) -> Result<Presses, String> {
    let g = &ctx.cfg.g8;
    let (r, on) = (spot.key, spot.at);
    let away = away_from(spot.window, on, win::desktop().1);
    let (mut out, wait) = (Presses::default(), Duration::from_millis(g.press_wait_ms));
    for _ in 0..g.presses {
        mouse::move_to(on)?;
        sleep_ms(g.hover_settle_ms);
        front()?;
        let before = picture(&r)?;
        let (start, at_us) = (Instant::now(), now_us());
        mouse::press_at(on)?;
        let look = || changed(&r, &before, &mut out.copy_ms);
        let lit = watch(start, wait, look, mouseguard::allowed);
        or_release(mouse::release_at(away), simuser::release_all)?;
        out.spans.push(Span::press(at_us, now_us(), g.rise_ms));
        out.lit_ms.push(lit?);
    }
    sleep_ms(g.rise_wait_ms);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    #[test]
    fn the_light_wait_ends_at_once_when_the_mouse_guard_stops_the_gate() {
        let looks = Cell::new(0);
        let look = || {
            looks.set(looks.get() + 1);
            Ok(false)
        };
        let go_on = || match looks.get() {
            0 | 1 => Ok(()),
            _ => Err("stopped by the mouse guard".to_string()),
        };
        let e = watch(Instant::now(), Duration::MAX, look, go_on).expect_err("stopped");
        assert!(e.contains("mouse guard"), "{e}");
        assert_eq!(looks.get(), 2, "no look after the stop");
    }

    #[test]
    fn the_light_wait_gives_the_time_to_the_light_or_none_once_its_wait_is_over() {
        let looks = Cell::new(0);
        let look = || {
            looks.set(looks.get() + 1);
            Ok(looks.get() == 3)
        };
        let lit = watch(Instant::now(), Duration::MAX, look, || Ok(()));
        assert!(matches!(lit, Ok(Some(_))) && looks.get() == 3, "{lit:?}");
        let never = watch(Instant::now(), Duration::ZERO, || Ok(false), || Ok(()));
        assert_eq!(never, Ok(None));
    }

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
