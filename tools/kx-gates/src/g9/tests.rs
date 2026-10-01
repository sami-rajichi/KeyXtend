//! Tests of G9's flag rule, its size steps and its pass rule on recorded styles.

use kx_platform_windows::window::KEPT;

use super::*;

/// The keyboard window's style, recorded from the debug app on 2026-10-01: no-activate, layered, tool, topmost.
const KEYBOARD_EX: u32 = 0x0808_0088;
/// Widths of the keyboard on this PC at 125 %, one size step apart, and the slack the settings allow.
const WIDTHS: [i32; 3] = [765, 850, 935];
const SLACK: i32 = 2;

fn seen(step: &'static str, ex: u32, first: u32, front_kept: bool) -> Seen {
    Seen {
        step,
        window: "kb".into(),
        ex: format!("{ex:#010x}"),
        missing: missing(ex, wants(first)),
        front_kept,
    }
}

#[test]
fn the_names_cover_the_guards_flags_and_click_through() {
    let kept = FLAGS[..3].iter().fold(0, |all, (_, bit)| all | bit);
    let through = FLAGS[3..].iter().fold(0, |all, (_, bit)| all | bit);
    assert_eq!((kept, through), (KEPT, THROUGH));
}

#[test]
fn layered_alone_is_not_wanted_back_after_a_strip_as_the_guard_keeps_only_both() {
    let stripped = kept_ex(KEYBOARD_EX, KEYBOARD_EX & !(KEPT | THROUGH));
    assert!(
        missing(stripped, wants(KEYBOARD_EX)).is_empty(),
        "{stripped:#x}"
    );
    let glow = KEYBOARD_EX | WS_EX_TRANSPARENT.0;
    let kept = kept_ex(glow, glow & !(KEPT | THROUGH));
    assert!(
        missing(kept, wants(glow)).is_empty(),
        "both click-through flags stay"
    );
}

#[test]
fn the_recorded_keyboard_keeps_every_flag_and_a_lost_one_is_named() {
    assert!(missing(KEYBOARD_EX, wants(KEYBOARD_EX)).is_empty());
    assert_eq!(
        missing(KEYBOARD_EX & !WS_EX_TOPMOST.0, wants(KEYBOARD_EX)),
        ["topmost"]
    );
    let glow = KEYBOARD_EX | WS_EX_TRANSPARENT.0;
    assert_eq!(
        missing(KEYBOARD_EX, wants(glow)),
        ["transparent"],
        "a click-through window stays so"
    );
    assert!(
        missing(glow, wants(KEYBOARD_EX)).is_empty(),
        "extra flags are fine"
    );
}

#[test]
fn the_size_steps_after_a_grow_end_at_the_first_size() {
    let steps = |grew| back(grew).collect::<Vec<_>>();
    assert_eq!(steps(true), [(Size::Shrink, SHRINK)]);
    assert_eq!(
        steps(false),
        [(Size::Shrink, SHRINK), (Size::Grow, GROW_BACK)],
        "the grow changed nothing"
    );
}

#[test]
fn a_run_that_broke_off_is_sized_back_toward_its_first_width() {
    let [small, first, big] = WIDTHS;
    assert_eq!(mend(first, big, SLACK), Some(Size::Shrink));
    assert_eq!(mend(first, small, SLACK), Some(Size::Grow));
    assert_eq!(mend(first, first, SLACK), None);
    assert_eq!(mend(first, first + 1, SLACK), None, "DPI rounding");
}

#[test]
fn click_through_is_turned_on_only_where_it_was_as_the_guard_never_drops_it() {
    assert!(!was_through(KEYBOARD_EX));
    assert!(was_through(KEYBOARD_EX | WS_EX_TRANSPARENT.0));
    let stuck = kept_ex(KEYBOARD_EX | THROUGH, KEYBOARD_EX);
    assert_eq!(
        stuck & THROUGH,
        THROUGH,
        "on the keyboard it would ignore clicks until it restarts"
    );
}

#[test]
fn g9_passes_only_when_every_step_looked_kept_every_flag_and_the_focus() {
    let steps = ["start", "topmost"];
    let good = [
        seen("start", KEYBOARD_EX, KEYBOARD_EX, true),
        seen("topmost", KEYBOARD_EX, KEYBOARD_EX, true),
    ];
    assert!(passed(&good, &steps));
    assert!(!passed(&good[..1], &steps), "a step saw no window");
    let lost = [
        good[0].clone(),
        seen("topmost", KEYBOARD_EX & !WS_EX_TOPMOST.0, KEYBOARD_EX, true),
    ];
    assert!(!passed(&lost, &steps));
    let focus = [
        good[0].clone(),
        seen("topmost", KEYBOARD_EX, KEYBOARD_EX, false),
    ];
    assert!(!passed(&focus, &steps), "the target lost the focus");
}
