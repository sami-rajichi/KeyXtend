//! Tests of G9's flag rule, its size steps and its pass rule on recorded styles.

use kx_platform_windows::window::{KEPT, THROUGH};

use super::*;

/// The keyboard window's style, recorded from the debug app on 2026-10-01: no-activate, layered, tool, topmost.
const KEYBOARD_EX: u32 = 0x0808_0088;

fn seen(step: &'static str, ex: u32, first: u32, front_kept: bool) -> Seen {
    Seen {
        step,
        window: "kb".into(),
        ex: format!("{ex:#010x}"),
        missing: missing(ex, wants(first)),
        extra: extra(ex, first),
        front_kept,
        front: None,
        guarded: true,
    }
}

fn resized(ms: Option<f64>) -> Resize {
    Resize {
        button: "bar.bigger".into(),
        ms,
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
    let stripped = kept_ex(KEYBOARD_EX & !(KEPT | THROUGH), is_through(KEYBOARD_EX));
    assert!(
        missing(stripped, wants(KEYBOARD_EX)).is_empty(),
        "{stripped:#x}"
    );
    let glow = KEYBOARD_EX | WS_EX_TRANSPARENT.0;
    let kept = kept_ex(glow & !(KEPT | THROUGH), is_through(glow));
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
fn a_resize_step_fails_when_no_press_changed_the_keyboard() {
    assert!(any_changed(&[resized(None), resized(Some(120.0))]));
    assert!(
        !any_changed(&[resized(None), resized(None)]),
        "the size buttons did nothing"
    );
}

#[test]
fn click_through_written_to_a_window_that_took_clicks_must_be_refused() {
    let written = KEYBOARD_EX | WS_EX_TRANSPARENT.0;
    assert_eq!(extra(written, KEYBOARD_EX), ["transparent"]);
    assert!(extra(KEYBOARD_EX, KEYBOARD_EX).is_empty());
    assert!(extra(written, written).is_empty(), "a click-through glow");
    let steps = ["click_through"];
    let kept = [seen("click_through", written, KEYBOARD_EX, true)];
    assert!(!passed(&kept, &steps), "the guard let it through");
}

#[test]
fn click_through_is_wanted_only_where_it_was_as_the_guard_refuses_it_elsewhere() {
    assert!(!is_through(KEYBOARD_EX));
    assert!(is_through(KEYBOARD_EX | WS_EX_TRANSPARENT.0));
    assert_eq!(wants(KEYBOARD_EX) & THROUGH, 0);
    let refused = kept_ex(KEYBOARD_EX | THROUGH, is_through(KEYBOARD_EX));
    assert_eq!(
        refused & WS_EX_TRANSPARENT.0,
        0,
        "the keyboard keeps taking clicks"
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

#[test]
fn the_window_in_front_is_named_only_when_it_is_not_the_target() {
    let (target, other) = (HWND(std::ptr::without_provenance_mut(1)), HWND::default());
    assert_eq!(in_front(target, target), None);
    assert_eq!(in_front(other, target).as_deref(), Some("(none)"));
}

#[test]
fn a_window_counts_as_put_back_only_when_its_writes_worked_and_its_style_is_the_first() {
    use crate::g9win::put_back;
    assert_eq!(put_back(Ok(()), KEYBOARD_EX, KEYBOARD_EX, "kb"), Ok(()));
    let off =
        put_back(Ok(()), KEYBOARD_EX & !KEPT, KEYBOARD_EX, "kb").expect_err("a flag is still off");
    assert!(off.starts_with("kb") && off.contains("0x08080088"), "{off}");
    let refused = put_back(
        Err("Windows refused".into()),
        KEYBOARD_EX,
        KEYBOARD_EX,
        "kb",
    );
    assert_eq!(
        refused,
        Err("Windows refused".into()),
        "a refused write is never counted"
    );
}

#[test]
fn every_window_g9_wrote_is_kept_once_for_the_repair() {
    let mut written = Vec::new();
    let (a, b) = (
        HWND(std::ptr::without_provenance_mut(1)),
        HWND(std::ptr::without_provenance_mut(2)),
    );
    note_written(&mut written, &[a, b]);
    note_written(&mut written, &[b]);
    assert_eq!(written, [a, b]);
}
