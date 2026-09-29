//! Tests of the hook thread's state: the hold timer and the wait behind our own input.

use super::*;

/// How long a hold must last, in ms; the tests never reach it.
const HOLD_MS: u64 = 1_000;
/// Moves up to this many px count as still.
const STILL_PX: i32 = 4;
/// The echo wait, in ms: far longer than a test runs.
const REPLY_MS: u64 = 60_000;

/// A host with nothing queued and the echo wait `REPLY_MS`.
fn host() -> Host {
    let setup = Setup {
        hold_ms: HOLD_MS,
        still_px: STILL_PX,
        source: Source::Simulated,
        own: Vec::new(),
        reply_ms: REPLY_MS,
    };
    Host::new(setup, 0, Arc::default())
}

#[test]
fn timer_ms_rounds_up_and_never_overflows() {
    assert_eq!(timer_ms(1_500_000, 0), 1501);
    assert_eq!(timer_ms(1_500_999, 0), 1501);
    assert_eq!(timer_ms(0, 10), 1);
    assert_eq!(timer_ms(i64::MAX, i64::MIN), u32::MAX);
}

#[test]
fn input_we_sent_holds_later_input_back() {
    let mut host = host();
    assert!(!host.behind());
    host.sent(now_us());
    assert!(host.behind());
}

#[test]
fn a_failed_injection_ends_the_wait_and_keeps_the_error() {
    let mut host = host();
    host.sent(now_us());
    host.failed("injection failed".to_string());
    assert!(!host.behind());
    assert_eq!(host.into_report().errors, ["injection failed"]);
}

#[test]
fn input_that_never_comes_back_stops_the_wait_and_is_reported() {
    let mut host = host();
    host.sent(now_us() - host.echo_wait_us - 1);
    assert!(!host.behind());
    assert_eq!(host.into_report().errors.len(), 1);
}
