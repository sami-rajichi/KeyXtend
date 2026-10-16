//! The lines owed per tick, the cap on a long gap, and the restarts.

use super::*;
use kx_module_api::US_PER_MS;

/// How close two line counts must be: far below any wheel step.
pub(super) const EPS: f64 = 1e-9;

/// The default speeds and check: 3 and 10 lines a second, 250 ms.
fn cfg() -> GlideCfg {
    GlideCfg::from(&ScrollSettings::from_defaults().unwrap())
}

/// The point `n` ms after the start.
fn ms(n: u64) -> Mono {
    Mono::from_us(n * u64::from(US_PER_MS))
}

/// Whether `got` lines are `want` lines.
fn same(got: f64, want: f64) -> bool {
    (got - want).abs() < EPS
}

#[test]
fn the_settings_give_both_speeds_and_the_cap() {
    let c = cfg();
    assert_eq!((c.slow, c.fast, c.cap), (3, 10, Duration::from_millis(250)));
    assert_eq!((c.speed(false), c.speed(true)), (3, 10));
}

#[test]
fn nothing_is_owed_at_the_start() {
    let mut g = Glide::start(cfg(), false, ms(5));
    assert!(same(g.tick(ms(5)), 0.0));
}

#[test]
fn a_tick_owes_the_speed_times_its_gap() {
    let mut g = Glide::start(cfg(), false, ms(0));
    assert!(same(g.tick(ms(16)), 0.048));
    assert!(same(g.tick(ms(116)), 0.3));
}

#[test]
fn fast_owes_at_the_fast_speed() {
    let mut g = Glide::start(cfg(), true, ms(0));
    assert!(same(g.tick(ms(100)), 1.0));
}

#[test]
fn a_gap_of_one_check_owes_all_of_it() {
    let mut g = Glide::start(cfg(), false, ms(0));
    assert!(same(g.tick(ms(250)), 0.75));
}

#[test]
fn a_long_gap_owes_no_more_than_one_check() {
    let mut g = Glide::start(cfg(), true, ms(0));
    assert!(same(g.tick(ms(10_000)), 2.5));
    assert!(same(g.tick(ms(10_100)), 1.0));
}

#[test]
fn a_restart_settles_the_old_speed_then_goes_on_at_the_new() {
    let mut g = Glide::start(cfg(), false, ms(0));
    assert!(same(g.restart(true, ms(100)), 0.3));
    assert!(same(g.tick(ms(200)), 1.0));
    assert!(same(g.restart(false, ms(300)), 1.0));
    assert!(same(g.tick(ms(400)), 0.3));
}

#[test]
fn a_restart_right_after_a_tick_owes_nothing_more() {
    let mut g = Glide::start(cfg(), false, ms(0));
    assert!(same(g.tick(ms(16)), 0.048));
    assert!(same(g.restart(false, ms(16)), 0.0));
    assert!(same(g.tick(ms(32)), 0.048));
}

#[test]
fn a_restart_after_a_long_gap_is_capped_too() {
    let mut g = Glide::start(cfg(), false, ms(0));
    assert!(same(g.restart(true, ms(60_000)), 0.75));
    assert!(same(g.tick(ms(60_016)), 0.16));
}

#[test]
fn an_earlier_time_owes_nothing_and_never_counts_twice() {
    let mut g = Glide::start(cfg(), false, ms(100));
    assert!(same(g.tick(ms(50)), 0.0));
    assert!(same(g.restart(false, ms(20)), 0.0));
    assert!(same(g.tick(ms(200)), 0.3));
}

#[test]
fn the_largest_values_still_owe_a_finite_amount() {
    let huge = GlideCfg {
        slow: u32::MAX,
        fast: u32::MAX,
        cap: Duration::MAX,
    };
    let mut g = Glide::start(huge, true, Mono::from_us(0));
    let owed = g.tick(Mono::from_us(u64::MAX));
    assert!(owed.is_finite() && owed > 0.0, "{owed}");
}
