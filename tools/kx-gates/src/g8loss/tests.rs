//! Tests of G8's frame rule, on synthetic `PresentMon` rows read with and without a byte order mark.

use kx_theme::Themes;

use super::*;
use crate::presentmon;

/// The product's program, as `PresentMon` names it.
const PROGRAM: &str = "keyxtend.exe";
/// The CSV's header: only the columns G8 reads.
const HEAD: &str =
    "Application,SwapChainAddress,CPUStartQPCTime,MsBetweenPresents,MsUntilDisplayed\n";
/// A 60 Hz screen: one refresh is 16.7 ms.
const HZ: f64 = 60.0;
/// G8's start, its one press with a 200 ms rise, and its one resize, on the QPC clock in ms.
const START_MS: f64 = 10_000.0;
const PRESS_MS: f64 = 11_000.0;
const RISE_MS: u32 = 200;
/// How long a slow key keeps the button down before G8 lets go, within `[g8] press_wait_ms`.
const HELD_MS: f64 = 300.0;
const RESIZE_MS: (f64, f64) = (12_000.0, 12_500.0);
/// The swap chains in the rows.
const CHAIN: &str = "0x1";
const OTHER: &str = "0x2";

/// QPC ms `ms` in µs.
#[allow(clippy::cast_possible_truncation, reason = "Whole test times.")]
fn us(ms: f64) -> i64 {
    (ms * 1_000.0) as i64
}

/// A present of the product on swap chain `chain` at QPC ms `ms`, shown or never shown.
fn row(chain: &str, ms: f64, shown: bool) -> String {
    let display = if shown { "6.0" } else { "NA" };
    format!("{PROGRAM},{chain},{ms},6.0,{display}\n")
}

/// The chain's first frame, shown, and a frame shown inside the press, so the times line up.
fn base() -> Vec<String> {
    vec![
        row(CHAIN, START_MS + 100.0, true),
        row(CHAIN, PRESS_MS + 10.0, true),
    ]
}

fn spans() -> Vec<Span> {
    vec![
        Span::press(us(PRESS_MS), us(PRESS_MS), u64::from(RISE_MS)),
        Span::resize(us(RESIZE_MS.0), us(RESIZE_MS.1)),
    ]
}

/// The losses in `rows` after `base`, read with and without a byte order mark, which must agree.
fn read(rows: &[String]) -> Result<Losses, String> {
    read_over(rows, &spans())
}

/// `read` against G8's steps `spans`.
fn read_over(rows: &[String], spans: &[Span]) -> Result<Losses, String> {
    let csv = format!("{HEAD}{}{}", base().concat(), rows.concat());
    let of = |csv: &str| {
        let f = presentmon::parse(csv, PROGRAM)?;
        losses(&f.presents, spans, Some(HZ), us(START_MS))
    };
    let (plain, marked) = (of(&csv), of(&format!("\u{FEFF}{csv}")));
    assert_eq!(plain, marked, "the byte order mark changes nothing");
    plain
}

fn loss(ms: f64, during: During) -> Loss {
    Loss {
        at_ms: ms - START_MS,
        during,
    }
}

#[test]
fn a_loss_while_a_pressed_key_rises_counts() {
    let at = PRESS_MS + 100.0;
    let got = read(&[row(CHAIN, at, false), row(CHAIN, at + 50.0, true)]).expect("lines up");
    assert_eq!(got.counted, [loss(at, During::Press)]);
    assert!(got.listed.is_empty());
}

#[test]
fn a_press_window_runs_from_the_press_to_its_release_plus_the_rise() {
    let up = PRESS_MS + HELD_MS;
    let held = [Span::press(us(PRESS_MS), us(up), u64::from(RISE_MS))];
    let end = up + f64::from(RISE_MS);
    let (rising, after) = (end - 10.0, end + 50.0);
    let rows = [
        row(CHAIN, rising, false),
        row(CHAIN, rising + 30.0, true),
        row(CHAIN, after, false),
        row(CHAIN, after + 30.0, true),
    ];
    let got = read_over(&rows, &held).expect("lines up");
    assert_eq!(
        got.counted,
        [loss(rising, During::Press)],
        "the rise starts at the release"
    );
    assert_eq!(got.listed, [loss(after, During::Outside)]);
}

#[test]
fn a_swap_chains_first_frame_never_counts_even_inside_a_press() {
    let at = PRESS_MS + 20.0;
    let got = read(&[row(OTHER, at, false), row(OTHER, at + 50.0, true)]).expect("lines up");
    assert!(got.counted.is_empty() && got.listed.is_empty(), "{got:?}");
    assert_eq!(got.skipped, 1);
}

#[test]
fn a_frame_a_newer_one_replaced_within_a_refresh_never_counts() {
    let at = PRESS_MS + 100.0;
    let replaced = [row(CHAIN, at, false), row(CHAIN, at + 10.0, true)];
    let got = read(&replaced).expect("lines up");
    assert!(got.counted.is_empty(), "replaced within 16.7 ms");
    assert_eq!(got.skipped, 1);
    let late = [row(CHAIN, at, false), row(CHAIN, at + 20.0, true)];
    assert_eq!(
        read(&late).expect("lines up").counted.len(),
        1,
        "a refresh went by"
    );
}

#[test]
fn a_loss_in_a_resize_is_listed_and_not_counted() {
    let at = RESIZE_MS.0 + 100.0;
    let got = read(&[row(CHAIN, at, false), row(CHAIN, at + 50.0, true)]).expect("lines up");
    assert!(got.counted.is_empty());
    assert_eq!(got.listed, [loss(at, During::Resize)]);
}

#[test]
fn a_loss_after_the_last_step_or_between_steps_is_listed_and_not_counted() {
    let (after, between) = (RESIZE_MS.1 + 500.0, PRESS_MS + 500.0);
    let rows = [
        row(CHAIN, between, false),
        row(CHAIN, between + 50.0, true),
        row(CHAIN, after, false),
        row(CHAIN, after + 50.0, true),
    ];
    let got = read(&rows).expect("lines up");
    assert!(got.counted.is_empty());
    let want = [loss(between, During::Outside), loss(after, During::Outside)];
    assert_eq!(got.listed, want);
}

#[test]
fn times_that_do_not_line_up_or_a_missing_refresh_rate_are_errors_not_passes() {
    let csv = format!("{HEAD}{}", row(CHAIN, START_MS, true));
    let f = presentmon::parse(&csv, PROGRAM).expect("reads");
    let e = losses(&f.presents, &spans(), Some(HZ), us(START_MS)).expect_err("apart");
    assert_eq!(e, APART);
    let lined = format!("{HEAD}{}", base().concat());
    let f = presentmon::parse(&lined, PROGRAM).expect("reads");
    let e = losses(&f.presents, &spans(), None, us(START_MS)).expect_err("no rate");
    assert_eq!(e, NO_HZ);
}

#[test]
fn the_shipped_rise_covers_the_themes_press_moves_and_g8_waits_it_out() {
    let g = crate::config::load().expect("kx-gates settings").g8;
    let moves = &Themes::shipped()
        .expect("the tokens read")
        .motion
        .full
        .moves;
    for name in ["press", "colour"] {
        assert!(g.rise_ms >= u64::from(moves[name].ms), "{name}");
    }
    assert!(g.rise_wait_ms >= g.rise_ms);
}
