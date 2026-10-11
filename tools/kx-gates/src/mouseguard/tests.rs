//! Tests of the mouse guard's settings, its view of hook events, its waits and how each end is named.

use std::collections::BTreeSet;
use std::time::Duration;

use windows::Win32::UI::WindowsAndMessaging::{
    WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_RBUTTONDOWN, WM_XBUTTONDOWN,
    WM_XBUTTONUP, XBUTTON1,
};

use super::*;

/// Windows' longest wait for a low-level hook since Windows 10 1709, in ms (Microsoft's `LowLevelMouseProc` docs).
const HOOK_TIMEOUT_MS: u64 = 1_000;

fn shipped() -> MouseGuard {
    crate::config::load()
        .expect("kx-gates.toml loads")
        .mouse_guard
}

#[test]
fn the_shipped_guard_is_on_with_the_owners_limit_and_one_click_escape() {
    let g = shipped();
    assert!(g.on);
    assert_eq!(
        (g.limit_ms, g.escape_clicks, g.escape_window_ms),
        (360_000, 1, 3_000),
        "owner, 2026-10-11"
    );
    assert!(
        g.alive_ms >= 2 * HOOK_TIMEOUT_MS,
        "twice Windows' 1 s hook timeout"
    );
    assert_eq!(g.check(), Ok(()));
}

#[test]
fn a_zero_limit_no_escape_click_a_zero_window_or_a_zero_alive_time_is_refused() {
    let bad: [fn(&mut MouseGuard); 4] = [
        |g| g.limit_ms = 0,
        |g| g.escape_clicks = 0,
        |g| g.escape_window_ms = 0,
        |g| g.alive_ms = 0,
    ];
    for (i, f) in bad.iter().enumerate() {
        let mut g = shipped();
        f(&mut g);
        assert!(g.check().is_err(), "case {i}");
    }
}

#[test]
fn the_start_line_says_how_many_clicks_give_the_mouse_back_and_never_asks_for_a_hold() {
    let one = shipped();
    let two = MouseGuard {
        escape_clicks: 2,
        ..shipped()
    };
    assert!(
        one.line().contains("press any mouse button"),
        "{}",
        one.line()
    );
    let within = format!("2 times within {} ms", two.escape_window_ms);
    assert!(two.line().contains(&within), "{}", two.line());
    assert!(!one.line().contains("hold") && !two.line().contains("hold"));
}

#[test]
fn only_input_that_kx_gates_injected_and_tagged_counts_as_its_own() {
    let injected = LLMHF_INJECTED;
    assert!(ours(injected, TAG));
    assert!(ours(injected, SIM_TAG));
    assert!(!ours(injected, 0), "another program's injected input");
    assert!(!ours(0, TAG), "a tag without the injected flag");
    assert!(!ours(0, 0), "the hand");
}

#[test]
fn hook_messages_become_presses_releases_or_other_events() {
    let x2 = u32::from(XBUTTON1 + 1) << 16;
    let x1 = u32::from(XBUTTON1) << 16;
    assert_eq!(kind_of(WM_LBUTTONDOWN, 0), Kind::Down(LEFT));
    assert_eq!(kind_of(WM_LBUTTONUP, 0), Kind::Up(LEFT));
    assert_eq!(kind_of(WM_RBUTTONDOWN, 0), Kind::Down(RIGHT));
    assert_eq!(kind_of(WM_XBUTTONDOWN, x1), Kind::Down(X1));
    assert_eq!(kind_of(WM_XBUTTONUP, x2), Kind::Up(X2));
    assert_eq!(kind_of(WM_MOUSEMOVE, 0), Kind::Other);
    assert_eq!(kind_of(WM_MOUSEWHEEL, x1), Kind::Other);
}

#[test]
fn a_wait_rounds_up_to_whole_ms_and_stops_at_the_largest_wait() {
    assert_eq!(wait_ms(Duration::from_micros(1)), 1);
    assert_eq!(wait_ms(Duration::from_millis(7)), 7);
    assert_eq!(wait_ms(Duration::ZERO), 0);
    assert_eq!(wait_ms(Duration::from_secs(u64::MAX)), INFINITE - 1);
    let endless = Duration::from_millis(u64::from(INFINITE));
    assert_eq!(wait_ms(endless), INFINITE - 1, "never a wait without end");
}

#[test]
fn only_a_press_the_limit_or_a_lost_hook_stops_the_gate_and_each_end_says_its_own_why() {
    let all = End::ALL;
    let stops: Vec<_> = all.iter().filter(|e| e.stops()).collect();
    let lost = [Loss::Off, Loss::Prompt, Loss::Thread].map(End::Lost);
    assert_eq!(
        stops,
        [&End::Owner, &End::Limit, &lost[0], &lost[1], &lost[2]]
    );
    let whys: BTreeSet<_> = all.iter().map(|e| e.why()).collect();
    assert_eq!(whys.len(), all.len(), "each end says its own why");
    let names: BTreeSet<_> = all.iter().map(|e| e.name()).collect();
    assert_eq!(
        names.len(),
        all.len() - lost.len() + 1,
        "every loss is lost"
    );
    assert!(lost.iter().all(|e| e.name() == "lost"));
}

#[test]
fn a_press_is_named_without_saying_whose_it_was() {
    let why = End::Owner.why();
    assert!(why.starts_with("a mouse button was pressed"), "{why}");
    assert!(!why.contains("owner"), "another program may have clicked");
}
