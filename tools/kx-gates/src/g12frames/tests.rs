//! Tests of `g12frames`: the overlay's swap chain picked from fixture CSV text, gaps kept inside one span, the numbers
//! of a known series with each span's quiet head and tail, lost frames and the capture's cut.

use super::*;
use crate::presentmon;
use crate::stats::US_PER_MS_INT;

/// The test build's program, as `PresentMon` names it.
const PROGRAM: &str = "keyxtend.exe";
/// The overlay's swap chain.
const OVERLAY: &str = "0xA";
/// The keyboard's swap chain.
const KEYBOARD: &str = "0xB";
/// A 60 Hz screen.
const HZ: Option<f64> = Some(60.0);
/// The ring's time to show, in ms.
const SHOW_MS: u64 = 50;

/// One refresh of `HZ`, in µs.
fn refresh() -> f64 {
    refresh_us(HZ).expect("a known rate")
}

/// The ring's time to show, in µs.
fn show_us() -> i64 {
    us_of(SHOW_MS)
}

/// One row: a chain, its present time in ms on the QPC clock, and whether it showed.
type Row = (&'static str, f64, bool);

/// `PresentMon` 2.x CSV text under `--qpc_time_ms` holding `rows`, each with no CPU busy time.
fn csv(rows: &[Row]) -> String {
    let head = "Application,SwapChainAddress,CPUStartQPCTime,CPUBusy,FrameTime,DisplayedTime\n";
    let row = |&(chain, at_ms, shown): &Row| {
        let shown = if shown { "6.0" } else { "NA" };
        format!("{PROGRAM},{chain},{at_ms},0.0,16.0,{shown}\n")
    };
    format!("{head}{}", rows.iter().map(row).collect::<String>())
}

/// The presents of `rows`, read as G12 reads its CSV.
fn presents(rows: &[Row]) -> Vec<Present> {
    presentmon::parse(&csv(rows), PROGRAM)
        .expect("reads")
        .presents
}

/// A ring span from `from_ms` to `to_ms`.
fn span(from_ms: i64, to_ms: i64) -> Span {
    Span {
        from_us: from_ms * US_PER_MS_INT,
        to_us: to_ms * US_PER_MS_INT,
    }
}

#[test]
fn a_keyboard_chain_busy_outside_the_spans_loses_to_the_overlays_chain() {
    let mut rows: Vec<Row> = (0..20)
        .map(|i| (KEYBOARD, f64::from(i) * 16.0, true))
        .collect();
    rows.extend([
        (OVERLAY, 1010.0, true),
        (KEYBOARD, 1020.0, true),
        (OVERLAY, 1026.0, true),
        (OVERLAY, 1042.0, true),
    ]);
    let all = presents(&rows);
    assert_eq!(chain(&all, &[span(1000, 1050)]).as_deref(), Some(OVERLAY));
    assert_eq!(
        chain(&all, &[span(5000, 6000)]),
        None,
        "no present in a span"
    );
}

#[test]
fn gaps_never_cross_two_spans() {
    let at = [10.0, 20.0, 30.0, 60.0, 100.0, 110.0];
    let rows: Vec<Row> = at.iter().map(|&ms| (OVERLAY, ms, true)).collect();
    let r = ring(
        &presents(&rows),
        OVERLAY,
        &[span(5, 35), span(95, 115)],
        refresh(),
        show_us(),
    );
    assert_eq!(
        (r.frames, r.gaps),
        (5, 3),
        "60 ms lies outside, and 30 to 100 crosses two spans"
    );
    assert_eq!(r.p99_ms, Some(10.0));
}

#[test]
fn fps_and_p99_come_out_of_a_known_series() {
    let rows = [
        (OVERLAY, 0.0, true),
        (OVERLAY, 10.0, true),
        (KEYBOARD, 15.0, true),
        (OVERLAY, 20.0, true),
        (OVERLAY, 40.0, true),
    ];
    let r = ring(
        &presents(&rows),
        OVERLAY,
        &[span(0, 50)],
        refresh(),
        show_us(),
    );
    assert_eq!((r.chain.as_str(), r.frames, r.gaps), (OVERLAY, 4, 3));
    assert_eq!(r.fps, Some(60.0), "three gaps in 40 ms, then 10 ms quiet");
    assert_eq!(r.p99_ms, Some(20.0), "the slowest of three gaps");
    let one = ring(
        &presents(&rows[..1]),
        OVERLAY,
        &[span(0, 50)],
        refresh(),
        show_us(),
    );
    assert_eq!((one.gaps, one.fps, one.p99_ms), (0, Some(0.0), None));
}

#[test]
fn a_ring_that_draws_twice_then_stalls_reads_far_below_its_rate() {
    let rows = [(OVERLAY, 10.0, true), (OVERLAY, 26.0, true)];
    let r = ring(
        &presents(&rows),
        OVERLAY,
        &[span(0, 600)],
        refresh(),
        show_us(),
    );
    assert_eq!(
        (r.gaps, r.drawn, r.p99_ms),
        (1, 1, Some(16.0)),
        "its one gap looks fine"
    );
    let slow = Some(f64::from(MS_PER_S) / 590.0);
    assert_eq!(
        (r.quiet_ms, r.fps),
        (574.0, slow),
        "one gap over the whole span, with its stall after the last frame"
    );
    let g = crate::config::load().expect("kx-gates.toml loads").g12;
    assert!(r.fps.is_some_and(|f| f < g.min_fps), "{r:?}");
}

#[test]
fn a_ring_drawn_at_60_hz_to_its_release_keeps_the_rate() {
    let at = |i: u32| (OVERLAY, 20.0 + f64::from(i) * 1000.0 / 60.0, true);
    let rows: Vec<Row> = (0..35).map(at).collect();
    let r = ring(
        &presents(&rows),
        OVERLAY,
        &[span(0, 600)],
        refresh(),
        show_us(),
    );
    let g = crate::config::load().expect("kx-gates.toml loads").g12;
    assert!(r.fps.is_some_and(|f| f >= g.min_fps), "{r:?}");
}

#[test]
fn a_first_frame_later_than_the_show_counts_its_wait_as_quiet() {
    let rows = [(OVERLAY, 80.0, true), (OVERLAY, 96.0, true)];
    let r = ring(
        &presents(&rows),
        OVERLAY,
        &[span(0, 100)],
        refresh(),
        show_us(),
    );
    let quiet = (r.quiet_ms, r.fps);
    assert_eq!(
        quiet,
        (34.0, Some(20.0)),
        "30 ms past the show, 4 ms after the last"
    );
    let none = ring(
        &presents(&rows),
        OVERLAY,
        &[span(200, 300)],
        refresh(),
        show_us(),
    );
    assert_eq!(
        (none.quiet_ms, none.fps),
        (50.0, Some(0.0)),
        "a span with no frame"
    );
}

#[test]
fn a_present_never_shown_counts_as_lost_by_g8s_rule() {
    let rows = [
        (OVERLAY, 10.0, false),
        (OVERLAY, 30.0, true),
        (OVERLAY, 50.0, false),
        (OVERLAY, 80.0, true),
        (KEYBOARD, 85.0, false),
        (OVERLAY, 100.0, false),
        (OVERLAY, 110.0, true),
        (OVERLAY, 300.0, false),
    ];
    let r = ring(
        &presents(&rows),
        OVERLAY,
        &[span(0, 200)],
        refresh(),
        show_us(),
    );
    // The chain's first at 10 ms and the one replaced 10 ms later at 100 ms are skipped; 300 ms is outside.
    assert_eq!((r.frames, r.lost), (6, 1), "only the one at 50 ms is lost");
}

#[test]
fn the_overlays_first_present_inside_the_first_span_is_not_lost() {
    let rows = [
        (OVERLAY, 10.0, false),
        (OVERLAY, 26.0, true),
        (OVERLAY, 42.0, true),
    ];
    let r = judge(&presents(&rows), &[span(0, 50)], HZ, SHOW_MS).expect("judged");
    assert_eq!((r.lost, r.gaps), (0, 2));
    let e = judge(&presents(&rows), &[span(0, 50)], None, SHOW_MS).expect_err("no refresh rate");
    assert!(e.contains("refresh"), "{e}");
}

/// The overlay's spans 0 to 40, 100 to 140 and 200 to 240 ms.
fn three() -> [Span; 3] {
    [span(0, 40), span(100, 140), span(200, 240)]
}

#[test]
fn a_span_with_no_gap_of_the_chain_is_not_drawn() {
    let rows = [
        (OVERLAY, 10.0, true),
        (OVERLAY, 26.0, true),
        (OVERLAY, 110.0, true),
        (OVERLAY, 126.0, true),
        (OVERLAY, 210.0, true),
    ];
    let r = ring(&presents(&rows), OVERLAY, &three(), refresh(), show_us());
    assert_eq!(
        (r.frames, r.drawn),
        (5, 2),
        "one present alone draws no gap"
    );
    let r = ring(
        &presents(&rows[..4]),
        OVERLAY,
        &three(),
        refresh(),
        show_us(),
    );
    assert_eq!(r.drawn, 2, "the last span holds no present");
}

#[test]
fn a_swap_chain_that_changed_address_leaves_its_later_spans_undrawn() {
    let at = [10.0, 20.0, 30.0, 110.0, 120.0, 130.0];
    let mut rows: Vec<Row> = at.iter().map(|&ms| (OVERLAY, ms, true)).collect();
    rows.extend([(KEYBOARD, 210.0, true), (KEYBOARD, 220.0, true)]);
    let r = judge(&presents(&rows), &three(), HZ, SHOW_MS).expect("judged");
    assert_eq!((r.chain.as_str(), r.drawn), (OVERLAY, 2));
}

#[test]
fn the_overlays_frames_are_judged_and_none_in_a_span_is_an_error() {
    let rows = [(OVERLAY, 10.0, true), (OVERLAY, 26.0, true)];
    let r = judge(&presents(&rows), &[span(0, 40)], HZ, SHOW_MS).expect("judged");
    assert_eq!((r.chain.as_str(), r.gaps), (OVERLAY, 1));
    let e = judge(&presents(&rows), &[span(100, 140)], HZ, SHOW_MS).expect_err("apart");
    assert!(e.contains("ring span"), "{e}");
}

#[test]
fn a_span_past_the_captures_end_errors() {
    let spans = [span(0, 40), span(100, 140)];
    let end = 140 * US_PER_MS_INT;
    assert_eq!(cut(&spans, end), Ok(()));
    let e = cut(&spans, end - 1).expect_err("the last span ends after the capture");
    assert!(e.contains("[g12] presentmon_secs"), "{e}");
}
