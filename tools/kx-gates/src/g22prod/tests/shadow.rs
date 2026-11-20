//! The bar's click-through shadow, which owns the bar and whose tree may list Stop: never taken as the bar, and its box
//! reads as a bar over the keys with a window on top.

use std::cell::RefCell;

use windows::Win32::Foundation::RECT;
use windows::Win32::UI::WindowsAndMessaging::{WS_EX_NOACTIVATE, WS_EX_TOPMOST, WS_EX_TRANSPARENT};

use super::super::bar::{BarLook, chosen};

/// The bar's, its shadow's and the keyboard's window keys.
const BAR: isize = 7;
const SHADOW: isize = 8;
const KB: isize = 3;
/// Stop's box, as the bar's tree gives it.
const STOP: RECT = RECT {
    left: 10,
    top: 4,
    right: 40,
    bottom: 34,
};
/// The bar's extended style, and its shadow's, which is click-through as well.
const BAR_EX: u32 = WS_EX_NOACTIVATE.0 | WS_EX_TOPMOST.0;
const SHADOW_EX: u32 = BAR_EX | WS_EX_TRANSPARENT.0;
/// The bar's size, its gap above the keyboard (`panel.gap_px`), its shadow's glow margin and the keyboard's height.
const WIDTH: i32 = 400;
const HEIGHT: i32 = 38;
const GAP: i32 = 10;
const GLOW: i32 = 44;
const KB_HEIGHT: i32 = 300;

/// The chosen window among `windows`, where those in `stop` hold Stop, and each window asked.
fn choose(windows: &[(isize, u32)], stop: &[isize]) -> (Option<isize>, Vec<isize>) {
    let asked = RefCell::new(Vec::new());
    let holds = |w: isize| {
        asked.borrow_mut().push(w);
        Ok::<_, String>(stop.contains(&w).then_some(STOP))
    };
    let found = chosen(windows, holds).expect("no read fails");
    (found.map(|(w, _)| w), asked.into_inner())
}

#[test]
fn the_bar_is_chosen_over_its_shadow_in_either_order_and_the_shadow_is_never_asked() {
    let both = [BAR, SHADOW];
    let bar_first = choose(&[(BAR, BAR_EX), (SHADOW, SHADOW_EX)], &both);
    assert_eq!(bar_first, (Some(BAR), vec![BAR]));
    let shadow_first = choose(&[(SHADOW, SHADOW_EX), (BAR, BAR_EX)], &both);
    assert_eq!(shadow_first, (Some(BAR), vec![BAR]));
}

#[test]
fn only_the_shadow_holding_stop_chooses_nothing_so_the_poll_waits() {
    let got = choose(&[(BAR, BAR_EX), (SHADOW, SHADOW_EX)], &[SHADOW]);
    assert_eq!(got, (None, vec![BAR]));
}

#[test]
fn a_lone_window_holding_stop_is_chosen_with_its_box() {
    let holds = |_| Ok::<_, String>(Some(STOP));
    assert_eq!(chosen(&[(BAR, BAR_EX)], holds), Ok(Some((BAR, STOP))));
    assert_eq!(
        choose(&[(BAR, BAR_EX)], &[]),
        (None, vec![BAR]),
        "Stop not there yet"
    );
}

fn rect(left: i32, top: i32, right: i32, bottom: i32) -> RECT {
    RECT {
        left,
        top,
        right,
        bottom,
    }
}

#[test]
fn the_shadows_box_reads_as_over_the_keys_and_covered_by_the_bar() {
    let bar = rect(0, 0, WIDTH, HEIGHT);
    let kb_top = HEIGHT + GAP;
    let kb = rect(0, kb_top, WIDTH, kb_top + KB_HEIGHT);
    let shadow = rect(-GLOW, -GLOW, WIDTH + GLOW, HEIGHT + GLOW);
    let order = [BAR, SHADOW, KB];
    let bar_look = BarLook::of(BAR_EX, &order, (BAR, bar), (KB, kb), Some(BAR));
    assert!(bar_look.ok(), "{bar_look:?}");
    let shadow_look = BarLook::of(SHADOW_EX, &order, (SHADOW, shadow), (KB, kb), Some(BAR));
    let want = BarLook {
        apart: false,
        uncovered: false,
        ..bar_look
    };
    assert_eq!(shadow_look, want, "the failed Arabic lines' look");
}
