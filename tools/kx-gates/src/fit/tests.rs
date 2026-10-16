//! Tests of the run lengths checked at load: each guarded run, G8's capture and hold, and G12's capture.

use kx_module_api::MS_PER_S;

use super::*;
use crate::config::load;

#[test]
fn a_click_run_takes_each_hold_and_gap() {
    let g2 = load().expect("loads").g2;
    let each = g2.press_ms + g2.click_gap_ms;
    assert_eq!(clicks_ms(&g2, 1000), 1000 * each);
    assert_eq!(clicks_ms(&g2, 0), 0);
}

#[test]
fn the_shipped_guarded_runs_fit_the_guards_limit() {
    let c = load().expect("loads");
    assert_eq!(check(&c), Ok(()));
    let g2 = clicks_ms(&c.g2, c.g2.clicks) + c.fit.guard_headroom_ms;
    assert!(g2 <= c.mouse_guard.limit_ms, "G2's {g2} ms");
}

/// The shipped settings with `change` made.
fn with(change: impl FnOnce(&mut GatesConfig)) -> GatesConfig {
    let mut c = load().expect("loads");
    change(&mut c);
    c
}

#[test]
fn a_click_run_or_g8_that_would_pass_the_guards_limit_is_refused() {
    let past = |c: &GatesConfig| {
        let each = c.g2.press_ms + c.g2.click_gap_ms;
        usize::try_from(c.mouse_guard.limit_ms / each).expect("fits")
    };
    let g2 = with(|c| c.g2.clicks = past(c));
    let e = check(&g2).expect_err("G2 runs too long");
    assert!(e.contains("g2"), "{e}");
    let g4 = with(|c| c.g4.clicks = past(c));
    assert!(check(&g4).expect_err("G4 too").contains("g4"));
    let g8 = with(|c| c.g8.presentmon_secs = c.mouse_guard.limit_ms / u64::from(MS_PER_S));
    assert!(check(&g8).expect_err("G8 too").contains("g8"));
}

#[test]
fn g8s_capture_covers_its_steps_through_its_g2_clicks() {
    let c = load().expect("loads");
    let need = g8_captured_ms(&c) + c.fit.capture_headroom_ms;
    assert!(ms_of(c.g8.presentmon_secs) >= need, "{need} ms to cover");
    assert!(
        g8_captured_ms(&c) > clicks_ms(&c.g2, c.g8.clicks),
        "the clicks are counted"
    );
    let more = with(|c| c.g8.clicks *= 10);
    let e = check(&more).expect_err("the capture would end before the clicks");
    assert!(e.contains("presentmon_secs"), "{e}");
}

#[test]
fn a_click_count_past_the_guards_limit_is_refused_for_its_gate() {
    let c = load().expect("loads");
    assert_eq!(clicks_fit(&c, "g4", c.g4.clicks), Ok(()));
    let each = c.g2.press_ms + c.g2.click_gap_ms;
    let past = usize::try_from(c.mouse_guard.limit_ms / each).expect("fits");
    let e = clicks_fit(&c, "g2", past).expect_err("too long");
    assert!(e.contains("g2"), "{e}");
    let off = with(|c| c.mouse_guard.on = false);
    assert_eq!(clicks_fit(&off, "g2", past), Ok(()), "nothing is held");
}

#[test]
fn g8s_use_step_follows_its_g2_clicks_inside_the_capture_and_comes_before_the_churn() {
    let long = with(|c| c.g8.presentmon_secs = c.mouse_guard.limit_ms);
    assert_eq!(
        g8_ms(&long, 10),
        g8_ms(&long, 0),
        "the capture's wait covers it"
    );
    let short = with(|c| c.g8.presentmon_secs = 0);
    let (rest, used) = (
        short.g8.use_hover_ms,
        g8_captured_ms(&short) + short.g8.warm_ms,
    );
    assert_eq!(g8_ms(&short, 10) - g8_ms(&short, 0), 10 * rest);
    assert!(g8_ms(&short, 0) > used, "then the churn");
    let slow = with(|c| c.g8.use_hover_ms = c.mouse_guard.limit_ms);
    assert!(check(&slow).expect_err("G8 too long").contains("g8"));
}

#[test]
fn churn_rounds_past_any_count_are_refused_without_a_panic() {
    let c = with(|c| (c.g8.churn_warm_rounds, c.g8.churn_rounds) = (usize::MAX, 1));
    assert!(check(&c).expect_err("G8 too long").contains("g8"));
}

/// The default hold time, in ms.
fn hold_ms() -> u64 {
    let m = MouseSettings::from_defaults().expect("the mouse defaults");
    u64::from(m.hold_ms)
}

#[test]
fn g12s_capture_covers_every_press_at_its_longest() {
    let c = load().expect("loads");
    let g = &c.g12;
    let clicks = u64::from(g.grid[0] * g.grid[1]);
    let each = c.sim.step_ms + hold_ms() - g.spare_ms + g.settle_ms + c.timing.poll_ms;
    let need = clicks * each + c.g8.presentmon_wait_ms;
    assert_eq!(g12_ms(&c, hold_ms()), need);
    assert!(ms_of(g.presentmon_secs) >= need, "{need} ms to cover");
    let short = with(|c| c.g12.presentmon_secs = 1);
    let e = check(&short).expect_err("the capture ends before the presses");
    assert!(e.contains("g12.presentmon_secs"), "{e}");
}

#[test]
fn a_g5_round_takes_each_case_with_its_hold_and_its_hang() {
    let c = load().expect("loads");
    let (g, s) = (&c.g5, &c.sim);
    let short = 2 * s.step_ms + s.click_ms + g.settle_ms;
    let long = 2 * s.step_ms + hold_ms() + s.hold_margin_ms + g.settle_ms;
    // Click, drag and the short move; the hold, the wiggle and the held drag; the hung hold.
    let round = 3 * short + 3 * long + (long + g.hang_ms);
    assert_eq!(rounds_ms(&c, 1, hold_ms()), round);
    assert_eq!(rounds_ms(&c, 4, hold_ms()), 4 * round);
}

#[test]
fn g12_g5_or_g8s_hook_part_past_the_guards_limit_is_refused() {
    let wide = with(|c| {
        c.g12.grid = [200, 100];
        c.g12.presentmon_secs = c.mouse_guard.limit_ms;
    });
    assert!(check(&wide).expect_err("G12 too long").contains("g12"));
    let g5 = with(|c| c.g5.rounds = 1000);
    assert!(check(&g5).expect_err("G5 too long").contains("g5"));
    let hook = with(|c| c.g8_hook.rounds = 1000);
    let e = check(&hook).expect_err("G8's hook part too long");
    assert!(e.contains("g8 hook"), "{e}");
}

#[test]
fn with_the_guard_off_nothing_is_cut_short() {
    let c = with(|c| {
        c.mouse_guard.on = false;
        c.g2.clicks = usize::MAX;
    });
    assert_eq!(check(&c), Ok(()));
}
