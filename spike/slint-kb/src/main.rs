//! P1 spike: the stage-1 keyboard drawn with Slint. Throwaway, deleted at the end of P1.

// Release builds open no console window; debug builds keep one for errors.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fmt::Display;
use std::rc::Rc;
use std::time::Duration;

use slint::{ComponentHandle, ModelRc, Timer, TimerMode, VecModel, Weak};
use spike_core::config::{self, KeyboardConfig};
use spike_core::layout::{self, Follow, KeyCap};
use spike_core::status::{self, Guard};
use spike_core::{inject, place, uiaccess, window};

slint::include_modules!();

/// Face name that picks this window's title in `spike.toml`.
const FACE: &str = "slint";
/// Number of the first guard try; `status::guard` counts from 1.
const FIRST_TRY: u32 = 1;

/// The status line: its fixed part and the window that shows it.
#[derive(Clone)]
struct Line {
    ui: Weak<Keyboard>,
    base: Rc<str>,
}

impl Line {
    /// Shows `note` after the fixed part.
    fn show(&self, note: &str) {
        if let Some(ui) = self.ui.upgrade() {
            ui.set_status(status::with(&self.base, note).into());
        }
    }
}

/// Guard timing from spike.toml.
#[derive(Clone, Copy)]
struct Tries {
    delay: Duration,
    max: u32,
}

fn main() {
    let cfg = config::load().unwrap_or_else(|err| fail(err));
    let prev = window::foreground();
    let ui = Keyboard::new().unwrap_or_else(|err| fail(err));
    let kb = &cfg.keyboard;
    ui.set_caption(cfg.title(FACE).into());
    set_sizes(&ui, kb);
    let (_relabel, keys) = follow_layout(&ui, kb.clone());
    let base = status::base(uiaccess::active(), keys).into();
    let line = Line {
        ui: ui.as_weak(),
        base,
    };
    line.show(status::GUARD_PENDING);
    on_tap(&ui, line.clone());
    let tries = Tries {
        delay: Duration::from_millis(kb.guard_delay_ms),
        max: kb.guard_tries,
    };
    guard_later(line, tries, FIRST_TRY, move || window::give_back(prev));
    ui.run().unwrap_or_else(|err| fail(err));
}

/// Reports `err` (console and TEMP file) and exits with code 1.
fn fail(err: impl Display) -> ! {
    config::report_error(FACE, &err.to_string());
    std::process::exit(1);
}

/// Gives the UI the key block size, gap and font size.
fn set_sizes(ui: &Keyboard, kb: &KeyboardConfig) {
    let (width, height) = place::block_size(kb);
    ui.set_block_width(width);
    ui.set_block_height(height);
    ui.set_gap_px(kb.gap_px);
    ui.set_font_px(kb.font_px);
}

/// Sends each tapped key to the app in front; errors go to the status line.
fn on_tap(ui: &Keyboard, line: Line) {
    ui.on_tapped(move |code| {
        if let Err(err) = inject::tap(code as u32) {
            line.show(&err);
        }
    });
}

/// Guards our windows after the delay, retrying while none is visible; then runs `give_back` once.
fn guard_later<F: FnOnce() + 'static>(line: Line, tries: Tries, attempt: u32, give_back: F) {
    Timer::single_shot(tries.delay, move || {
        match status::guard(window::guard_own_windows(), attempt, tries.max) {
            Guard::Retry => guard_later(line, tries, attempt + 1, give_back),
            Guard::Done(note) | Guard::GiveUp(note) => {
                give_back();
                line.show(&note);
            }
        }
    });
}

/// Shows the keys for the layout in front, then follows it; returns the timer and key count.
fn follow_layout(ui: &Keyboard, kb: KeyboardConfig) -> (Timer, usize) {
    let first = layout::foreground_layout();
    let keys = set_keys(ui, &layout::rows(&kb, first));
    let mut follow = Follow::new(first);
    let weak = ui.as_weak();
    let timer = Timer::default();
    let every = Duration::from_millis(kb.relabel_ms);
    timer.start(TimerMode::Repeated, every, move || {
        if let Some(hkl) = follow.changed()
            && let Some(ui) = weak.upgrade()
        {
            set_keys(&ui, &layout::rows(&kb, hkl));
        }
    });
    (timer, keys)
}

/// Replaces the UI keys with `caps`; returns the key count.
fn set_keys(ui: &Keyboard, caps: &[Vec<KeyCap>]) -> usize {
    let keys: Vec<Key> = caps.iter().flatten().map(key).collect();
    let count = keys.len();
    ui.set_keys(ModelRc::new(VecModel::from(keys)));
    count
}

/// One UI key from a core key cap, keeping its box.
fn key(cap: &KeyCap) -> Key {
    let p = cap.place;
    Key {
        label: cap.label.as_str().into(),
        code: cap.code as i32,
        x: p.x,
        y: p.y,
        w: p.w,
        h: p.h,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spike_core::place::Place;

    #[test]
    fn key_keeps_the_core_box() {
        let place = Place {
            x: 4.0,
            y: 56.0,
            w: 74.0,
            h: 48.0,
        };
        let cap = KeyCap {
            code: 0xE038,
            label: "Alt".into(),
            place,
        };
        let k = key(&cap);
        assert_eq!((k.code, k.label.as_str()), (0xE038, "Alt"));
        assert_eq!((k.x, k.y, k.w, k.h), (4.0, 56.0, 74.0, 48.0));
    }
}
