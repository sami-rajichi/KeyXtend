//! The checks that tie `[g19_product]` to the selection module's settings, the theme's gap and its own sizes.

use super::super::limits::default_gap;
use super::{refused, selection_defaults};
use crate::config::load;

/// The module's look, its settle then its read, in ms.
fn look() -> u64 {
    let s = selection_defaults().expect("defaults");
    u64::from(s.settle_ms) + u64::from(s.read_ms)
}

/// The module's deadline for an edit, in ms.
fn edit() -> u64 {
    u64::from(selection_defaults().expect("defaults").edit_ms)
}

/// The theme's gap at the keyboard's default size.
fn gap() -> i32 {
    default_gap().expect("the gap")
}

#[test]
fn a_wait_the_module_outlasts_or_an_edit_deadline_could_end_is_refused() {
    let g = load().expect("loads").g19_product;
    assert!(
        g.quiet_ms > look() && g.gone_wait_ms < edit(),
        "the shipped waits"
    );
    refused(&[
        (|g| g.quiet_ms = look(), "g19_product.quiet_ms"),
        (|g| g.gone_wait_ms = edit(), "g19_product.gone_wait_ms"),
    ]);
}

#[test]
fn a_slack_or_near_bottom_distance_as_wide_as_the_gap_is_refused() {
    let g = load().expect("loads").g19_product;
    assert!(g.place_slack_px < gap() && i32::from(g.near_px) < gap());
    refused(&[
        (|g| g.place_slack_px = gap(), "g19_product.place_slack_px"),
        (|g| g.place_slack_px = -1, "g19_product.place_slack_px"),
        (|g| g.near_px = 0, "g19_product.near_px"),
        (
            |g| g.near_px = u16::try_from(gap()).expect("small"),
            "g19_product.near_px",
        ),
    ]);
}

#[test]
fn a_ping_program_that_could_run_a_command_is_refused() {
    refused(&[
        (
            |g| g.ping_exe = "ping.exe;calc".into(),
            "g19_product.ping_exe",
        ),
        (|g| g.ping_exe = "ping exe".into(), "g19_product.ping_exe"),
        (|g| g.ping_exe = "$p.exe".into(), "g19_product.ping_exe"),
        (|g| g.ping_exe = "ping`.exe".into(), "g19_product.ping_exe"),
    ]);
}

#[test]
fn a_huge_repeat_is_refused_before_the_line_is_built() {
    let g = load().expect("loads").g19_product;
    assert_eq!(g.long_chars(), g.long_line().chars().count());
    refused(&[(
        |g| g.arabic_repeat = usize::MAX,
        "g19_product.arabic_repeat",
    )]);
}
