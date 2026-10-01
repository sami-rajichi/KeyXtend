//! Tests of G8's budget arithmetic on recorded samples.

use super::*;

/// The product's own start-up lines, recorded from the debug app on 2026-10-01.
const LOG: &str = "\
2026-10-01T18:29:22.924463Z  INFO keyxtend::run: KeyXtend started version=\"0.0.0\" ui_access=false
2026-10-01T18:29:23.998726Z  INFO kx_ui::hub::face: windows guarded guarded=2
";

#[test]
fn a_log_stamp_reads_as_microseconds_since_1970() {
    assert_eq!(stamp_us(LOG), Some(1_790_879_362_924_463));
    assert_eq!(stamp_us("1970-01-01T00:00:00Z x"), Some(0));
    assert_eq!(
        stamp_us("2000-03-01T00:00:00.5Z"),
        Some(951_868_800_500_000),
        "a short fraction"
    );
    assert_eq!(
        stamp_us("2024-02-29T23:59:59.000001Z"),
        Some(1_709_251_199_000_001),
        "a leap day"
    );
}

#[test]
fn a_line_without_a_utc_stamp_reads_as_nothing() {
    for bad in [
        "",
        "KeyXtend started",
        "2026-10-01T18:29:22.924463 INFO",
        "2026-1x-01T18:29:22Z",
        "2026-10-01",
    ] {
        assert_eq!(stamp_us(bad), None, "{bad:?}");
    }
}

#[test]
fn the_start_up_span_runs_from_the_start_line_to_the_next_guard_line() {
    let (from, to) = span_us(LOG, "KeyXtend started", "windows guarded").expect("both lines");
    assert!((ms_between(from, to) - 1074.263).abs() < 1e-9);
    let guard_first = format!(
        "{}\n{}",
        LOG.lines().nth(1).unwrap_or_default(),
        LOG.lines().next().unwrap_or_default()
    );
    assert_eq!(
        span_us(&guard_first, "KeyXtend started", "windows guarded"),
        None,
        "a guard line from before the start does not count"
    );
}

#[test]
fn a_budget_passes_only_when_measured_and_within_its_limit() {
    assert!(Part::at_most("start_ms", Some(500.0), 500.0).pass);
    assert!(!Part::at_most("start_ms", Some(500.1), 500.0).pass);
    assert!(!Part::at_most("start_ms", None, 500.0).pass, "not measured");
    let parts = [
        Part::at_most("a", Some(1.0), 2.0),
        Part::at_most("b", Some(3.0), 2.0),
    ];
    assert!(!all_pass(&parts));
    assert!(all_pass(&parts[..1]));
    assert!(!all_pass(&[]), "nothing measured passes nothing");
}

#[test]
fn idle_cpu_is_a_share_of_the_cores_the_product_may_use() {
    assert_eq!(cpu_pct(10.0, 10_000.0, 1.0), Some(0.1));
    assert_eq!(cpu_pct(80.0, 10_000.0, 8.0), Some(0.1));
    assert_eq!(cpu_pct(1.0, 0.0, 8.0), None);
    assert!((usable_cores(8, None) - 8.0).abs() < 1e-9);
    let near = |got: f64, want: f64| (got - want).abs() < 1e-9;
    assert!(
        near(usable_cores(8, Some((2, 30))), 2.0),
        "weak: 2 cores, under 30 % of 8"
    );
    assert!(
        near(usable_cores(8, Some((4, 25))), 2.0),
        "25 % of the whole PC is 2 of 8 cores"
    );
    assert!(
        near(usable_cores(1, Some((2, 50))), 0.5),
        "never more than the PC has, then its share"
    );
}

#[test]
fn a_press_may_take_its_frames_plus_one_screen_copy() {
    let got = lit_within_ms(165.0, 2.0, &[3.0, 1.0, 2.0]).expect("a budget");
    assert!((got - (2000.0 / 165.0 + 2.0)).abs() < 1e-9, "{got}");
    assert_eq!(lit_within_ms(0.0, 2.0, &[1.0]), None, "no refresh rate");
    assert_eq!(lit_within_ms(60.0, 2.0, &[]), None, "no copy timed");
}
