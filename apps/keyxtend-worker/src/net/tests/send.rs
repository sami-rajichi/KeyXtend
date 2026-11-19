//! One piece's request: tried again after a broken link, a stall or a service down, up to its tries; one wait for a
//! 429's `Retry-After`, then the rate fault; a used allowance, a final fault and a redirect at once; a reply past the
//! cap strange; the call's wait asked; the key in its header only and in no fault; the key's test; and every shape's
//! own adapter.

use super::eleven::own_form;
use super::*;
use crate::net::send::{Sending, piece, test};
use crate::net::shape;
use kx_ipc::{MAX_NET_REPLY_BYTES, NetFault};
use kx_platform_net::fake::Step;
use kx_platform_net::{NetError, Verb, wait_ms};
use std::sync::atomic::AtomicBool;

/// The sound the tests send.
const WAV: &[u8] = b"RIFF-and-samples";

/// What one piece's thread sends over `call` with `tries`.
fn sending(call: NetCall<'static>, tries: u8) -> Sending {
    Sending {
        shape: shape::of(call.shape.code()),
        call,
        model: Some(Cow::Borrowed(MODEL)),
        lang: Cow::Borrowed(LANG),
        tries,
        pace: None,
        dropped: Arc::new(AtomicBool::new(false)),
    }
}

/// Sends `WAV` with `tries` to a server doing `acts`: the outcome, the server and the clock.
fn post(
    acts: impl IntoIterator<Item = Act>,
    tries: u8,
) -> (Result<String, NetFault>, Arc<FakeHttp>, Arc<Still>) {
    let (http, clock) = (Arc::new(FakeHttp::new(acts)), Arc::new(Still::default()));
    let got = piece(&kit(&http, &clock), &sending(call(), tries), WAV);
    (got.map(|t| t.to_string()), http, clock)
}

/// An answer whose body stays quiet past the call's wait.
fn stalled() -> Act {
    Act::Answer(head(OK, &[]), vec![Step::Quiet(wait_ms(WAIT_S))])
}

#[test]
fn a_429_waits_once_for_its_retry_after_then_tries_again() {
    let (got, http, clock) = post([busy(&[("retry-after", "7")]), said("Bonjour")], 1);
    assert_eq!(got, Ok("Bonjour".into()));
    assert_eq!((clock.waits(), http.called().len()), (vec![7000], 2));
}

#[test]
fn a_second_429_is_the_rate_fault() {
    let acts = [
        busy(&[("retry-after", "1")]),
        busy(&[("retry-after", "1")]),
        said("x"),
    ];
    let (got, http, clock) = post(acts, 3);
    assert_eq!(got, Err(NetFault::Rate));
    assert_eq!((clock.waits(), http.called().len()), (vec![1000], 2));
}

#[test]
fn a_groq_rate_limit_waits_once_then_is_rate() {
    let hour = || busy_with(&[("retry-after", "3")], RATE_HOUR);
    let (got, http, clock) = post([hour(), said("Bonjour")], 1);
    assert_eq!(got, Ok("Bonjour".into()));
    assert_eq!((clock.waits(), http.called().len()), (vec![3000], 2));
    let (got, http, clock) = post([hour(), hour(), said("x")], 3);
    assert_eq!(got, Err(NetFault::Rate));
    assert_eq!((clock.waits(), http.called().len()), (vec![3000], 2));
}

#[test]
fn a_429_without_a_wait_or_past_the_calls_wait_is_rate_at_once() {
    let past = (WAIT_S + 1).to_string();
    for extra in [vec![], vec![("retry-after", past.as_str())]] {
        let (got, http, clock) = post([busy(&extra), said("x")], 3);
        assert_eq!(got, Err(NetFault::Rate), "{extra:?}");
        assert_eq!((clock.waits(), http.called().len()), (vec![], 1));
    }
    let most = WAIT_S.to_string();
    let (got, _, clock) = post([busy(&[("retry-after", most.as_str())]), said("x")], 1);
    assert_eq!(
        (got, clock.waits()),
        (Ok("x".into()), vec![u64::from(WAIT_S) * 1000])
    );
}

#[test]
fn a_used_allowance_is_told_at_once_with_its_renewal() {
    let used = [
        ("x-ratelimit-remaining-requests", "0"),
        ("x-ratelimit-reset-requests", "1h"),
    ];
    let (got, http, clock) = post([busy(&used), said("x")], 3);
    assert_eq!(
        got,
        Err(NetFault::Allowance {
            renews_s: Some(3600)
        })
    );
    assert_eq!((clock.waits(), http.called().len()), (vec![], 1));
}

#[test]
fn a_broken_link_a_stall_or_a_service_down_is_tried_again_up_to_the_tries() {
    let (got, http, _) = post([Act::Fail(NetError::Net), said("x")], 2);
    assert_eq!((got, http.called().len()), (Ok("x".into()), 2));
    let cases = [
        (
            [Act::Fail(NetError::Net), Act::Fail(NetError::Net)],
            NetFault::Network,
        ),
        ([stalled(), stalled()], NetFault::Timeout),
        ([reply(503, DOWN), reply(500, DOWN)], NetFault::Network),
        (
            [Act::Fail(NetError::Stall), reply(498, b"")],
            NetFault::Network,
        ),
    ];
    for (acts, fault) in cases {
        let (got, http, _) = post(acts.into_iter().chain([said("late")]), 2);
        assert_eq!((got, http.called().len()), (Err(fault), 2), "{fault:?}");
    }
    let (got, http, _) = post([Act::Fail(NetError::Net), said("x")], 1);
    assert_eq!((got, http.called().len()), (Err(NetFault::Network), 1));
}

#[test]
fn a_final_fault_is_never_tried_again() {
    let cases = [
        (reply(401, KEY_BAD), NetFault::Key),
        (reply(402, b""), NetFault::Credit),
        (reply(413, TOO_BIG), NetFault::TooBig),
        (reply(OK, HOTSPOT), NetFault::Strange),
        (reply(OK, NO_TEXT), NetFault::Strange),
    ];
    for (act, fault) in cases {
        let (got, http, _) = post([act, said("x")], 3);
        assert_eq!((got, http.called().len()), (Err(fault), 1), "{fault:?}");
    }
}

#[test]
fn a_redirect_is_strange_and_nothing_goes_to_its_host() {
    let (got, http, _) = post([Act::moved("https://evil.example/take"), said("x")], 3);
    assert_eq!(got, Err(NetFault::Strange));
    let called = http.called();
    assert_eq!((called.len(), called[0].url.as_str()), (1, URL));
}

#[test]
fn a_reply_past_the_cap_is_strange() {
    let over = Act::with(OK, &vec![b' '; MAX_NET_REPLY_BYTES + 1]);
    let (got, _, _) = post([over], 1);
    assert_eq!(got, Err(NetFault::Strange));
}

#[test]
fn the_request_waits_the_calls_seconds_and_carries_the_key_in_its_header_only() {
    let (_, http, _) = post([said("x")], 1);
    let called = &http.called()[0];
    assert_eq!(
        (called.verb, called.url.as_str(), called.stall_s),
        (Verb::Post, URL, WAIT_S)
    );
    let bearer = [&b"Bearer "[..], KEY].concat();
    let keyed: Vec<_> = called
        .fields
        .iter()
        .filter(|(_, v)| v.windows(KEY.len()).any(|w| w == KEY))
        .collect();
    assert_eq!(keyed, [&("Authorization".to_owned(), bearer)]);
    assert!(!called.body.windows(KEY.len()).any(|w| w == KEY));
}

#[test]
fn no_fault_names_the_key() {
    let failing = [
        reply(401, KEY_BAD),
        Act::Fail(NetError::Net),
        reply(OK, HOTSPOT),
        busy(&[]),
    ];
    for act in failing {
        let (got, _, _) = post([act], 1);
        let shown = format!("{got:?}");
        assert!(got.is_err() && !shown.contains(KEY_TEXT), "{shown}");
    }
}

#[test]
fn a_dropped_run_sends_nothing() {
    let (http, clock) = (
        Arc::new(FakeHttp::new([said("x")])),
        Arc::new(Still::default()),
    );
    let s = sending(call(), 3);
    s.dropped.store(true, Ordering::SeqCst);
    assert_eq!(
        piece(&kit(&http, &clock), &s, WAV).err(),
        Some(NetFault::Gone)
    );
    assert!(http.called().is_empty());
}

#[test]
fn a_key_test_says_whether_the_key_works() {
    let cases = [
        (reply(OK, LIST), Ok(())),
        (reply(401, KEY_BAD), Err(NetFault::Key)),
        (reply(OK, HOTSPOT), Err(NetFault::Strange)),
    ];
    for (act, want) in cases {
        let (http, clock) = (Arc::new(FakeHttp::new([act])), Arc::new(Still::default()));
        let s = sending(call_to(MODELS, NetShape::Form(form())), 1);
        assert_eq!(test(&kit(&http, &clock), &s), want);
        let called = &http.called()[0];
        assert_eq!(
            (called.verb, called.url.as_str(), called.body.len()),
            (Verb::Get, MODELS, 0)
        );
    }
}

#[test]
fn every_shape_has_its_own_adapter() {
    let shapes = [
        NetShape::Form(form()),
        NetShape::Eleven(own_form()),
        NetShape::Gemini,
    ];
    for shape in shapes {
        let call = call_to(MODELS, shape);
        // Each adapter refuses another shape's call, so a test request shows the code reached its own.
        let req = shape::of(call.shape.code()).test(&call);
        assert!(req.is_ok(), "{:?}", call.shape.code());
    }
}
