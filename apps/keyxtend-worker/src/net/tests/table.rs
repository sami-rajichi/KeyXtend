//! The form's status table over Groq's documented answers: the text in any script, a blank text, a reply that is not
//! the form's JSON, each error status, the wait a 429 asks for even when its message names billing, credit told only by
//! its exact code, a used allowance with its renewal, and the key's test.

use super::*;
use crate::net::form::Form;
use crate::net::shape::{Shape, Verdict};
use kx_ipc::NetFault;

/// A verdict with its text as plain text, so a test can compare it.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Got {
    Text(String),
    Wait(Option<u32>),
    Fault(NetFault),
}

/// The headers of an answer, as names and values.
pub(super) type Extra<'a> = &'a [(&'a str, &'a str)];

/// What `shape` reads in a piece's reply of `status` over `call`, with the headers `extra` and `body`.
pub(super) fn read_with(
    shape: &dyn Shape,
    call: &NetCall<'_>,
    status: u16,
    extra: Extra<'_>,
    body: &[u8],
) -> Got {
    match shape.read(call, &head(status, extra), body) {
        Verdict::Said(text) => Got::Text(text.to_string()),
        Verdict::Busy(s) => Got::Wait(s),
        Verdict::Fault(f) => Got::Fault(f),
    }
}

/// What `shape` reads in a key test's reply over `call`.
pub(super) fn tested_with(
    shape: &dyn Shape,
    call: &NetCall<'_>,
    status: u16,
    extra: Extra<'_>,
    body: &[u8],
) -> Got {
    match shape.tested(call, &head(status, extra), body) {
        Verdict::Said(()) => Got::Text(String::new()),
        Verdict::Busy(s) => Got::Wait(s),
        Verdict::Fault(f) => Got::Fault(f),
    }
}

/// What the form reads in a piece's reply of `status` over `call`, with the headers `extra` and `body`.
pub(super) fn read_by(call: &NetCall<'_>, status: u16, extra: Extra<'_>, body: &[u8]) -> Got {
    read_with(&Form, call, status, extra, body)
}

/// What the form reads in a key test's reply over `call`.
pub(super) fn tested_by(call: &NetCall<'_>, status: u16, extra: Extra<'_>, body: &[u8]) -> Got {
    tested_with(&Form, call, status, extra, body)
}

/// What the form reads in a piece's reply from Groq.
fn read(status: u16, extra: Extra<'_>, body: &[u8]) -> Got {
    read_by(&call(), status, extra, body)
}

/// What the form reads in a key test's reply from Groq.
fn tested(status: u16, extra: Extra<'_>, body: &[u8]) -> Got {
    tested_by(&call(), status, extra, body)
}

/// A 429's headers when the day's requests are used, renewing in `reset`.
fn used(reset: &str) -> [(&'static str, String); 2] {
    [
        ("x-ratelimit-remaining-requests", "0".to_owned()),
        ("x-ratelimit-reset-requests", reset.to_owned()),
    ]
}

/// The fault a used allowance renewing in `reset` gives.
fn allowance(reset: &str) -> Got {
    let headers = used(reset);
    let extra: Vec<(&str, &str)> = headers.iter().map(|(n, v)| (*n, v.as_str())).collect();
    read(BUSY, &extra, RATE)
}

#[test]
fn a_reply_holds_its_text_in_any_script() {
    assert_eq!(
        read(OK, &[], TEXT),
        Got::Text(" Bonjour tout le monde.".into())
    );
    assert_eq!(read(OK, &[], ARABIC), Got::Text("مرحبًا بالعالم".into()));
    assert_eq!(
        read(OK, &[], BLANK),
        Got::Text("  ".into()),
        "the run judges blanks"
    );
}

#[test]
fn a_reply_that_is_not_the_forms_json_is_strange() {
    let bodies: [&[u8]; 8] = [
        HOTSPOT,
        NO_TEXT,
        LIST,
        b"",
        br#"{"text":null}"#,
        br#"{"text":5}"#,
        br#"["text"]"#,
        br#"{"text":"cut"#,
    ];
    for body in bodies {
        let shown = String::from_utf8_lossy(body);
        assert_eq!(
            read(OK, &[], body),
            Got::Fault(NetFault::Strange),
            "{shown}"
        );
    }
}

#[test]
fn each_error_status_is_its_fault_as_groqs_page_says() {
    let cases: [(u16, &[u8], NetFault); 17] = [
        (401, KEY_BAD, NetFault::Key),
        (403, KEY_BAD, NetFault::Key),
        (402, b"", NetFault::Credit),
        (400, CREDIT, NetFault::Credit),
        (BUSY, CREDIT, NetFault::Credit),
        (413, TOO_BIG, NetFault::TooBig),
        (413, HOTSPOT, NetFault::TooBig),
        (400, LANGUAGE, NetFault::Language),
        (422, LANGUAGE, NetFault::Language),
        (400, BAD, NetFault::Strange),
        (422, BAD, NetFault::Strange),
        (404, HOTSPOT, NetFault::Strange),
        (498, b"", NetFault::Network),
        (500, DOWN, NetFault::Network),
        (503, HOTSPOT, NetFault::Network),
        (302, b"", NetFault::Strange),
        (301, HOTSPOT, NetFault::Strange),
    ];
    for (status, body, fault) in cases {
        assert_eq!(read(status, &[], body), Got::Fault(fault), "{status}");
    }
}

#[test]
fn a_429_asks_to_wait_for_its_retry_after_in_whole_seconds() {
    assert_eq!(
        read(BUSY, &[("retry-after", "7")], RATE),
        Got::Wait(Some(7))
    );
    assert_eq!(
        read(BUSY, &[("Retry-After", " 0 ")], RATE),
        Got::Wait(Some(0))
    );
    assert_eq!(read(BUSY, &[], RATE), Got::Wait(None));
    for odd in [
        "soon",
        "7.5",
        "-1",
        "4294967296",
        "Wed, 18 Nov 2026 07:28:00 GMT",
    ] {
        assert_eq!(
            read(BUSY, &[("retry-after", odd)], RATE),
            Got::Wait(None),
            "{odd}"
        );
    }
    let left = [
        ("retry-after", "2"),
        ("x-ratelimit-remaining-requests", "3"),
    ];
    assert_eq!(
        read(BUSY, &left, RATE),
        Got::Wait(Some(2)),
        "requests are left today"
    );
}

#[test]
fn a_used_allowance_tells_its_renewal_within_the_frames_range() {
    let renews = |s| Got::Fault(NetFault::Allowance { renews_s: s });
    assert_eq!(allowance("2m59.56s"), renews(Some(180)));
    assert_eq!(allowance("1h"), renews(Some(3600)));
    assert_eq!(allowance("1h2m3s"), renews(Some(3723)));
    assert_eq!(allowance("1m0.5s"), renews(Some(61)));
    assert_eq!(allowance("745h"), renews(None), "past a month is not told");
    assert_eq!(allowance("soon"), renews(None));
    assert_eq!(allowance(""), renews(None));
    let no_reset = [("x-ratelimit-remaining-requests", "0")];
    assert_eq!(read(BUSY, &no_reset, RATE), renews(None));
}

#[test]
fn no_request_left_until_a_reset_within_a_minute_is_a_wait() {
    assert_eq!(allowance("7.66s"), Got::Wait(Some(8)), "a minute's limit");
    assert_eq!(allowance("250ms"), Got::Wait(Some(1)));
    assert_eq!(allowance("0s"), Got::Wait(Some(0)));
    assert_eq!(allowance("1m"), Got::Wait(Some(60)), "the pace's span");
}

#[test]
fn a_key_test_works_on_the_model_list_and_fails_as_a_piece_does() {
    assert_eq!(tested(OK, &[], LIST), Got::Text(String::new()));
    assert_eq!(tested(OK, &[], HOTSPOT), Got::Fault(NetFault::Strange));
    assert_eq!(tested(OK, &[], b""), Got::Fault(NetFault::Strange));
    assert_eq!(tested(401, &[], KEY_BAD), Got::Fault(NetFault::Key));
    assert_eq!(
        tested(BUSY, &[("retry-after", "3")], RATE),
        Got::Wait(Some(3))
    );
    assert_eq!(tested(503, &[], DOWN), Got::Fault(NetFault::Network));
}

#[test]
fn a_groq_rate_limit_that_names_billing_is_a_wait_never_credit() {
    let wait = [("retry-after", "63")];
    assert_eq!(read(BUSY, &wait, RATE_HOUR), Got::Wait(Some(63)));
    assert_eq!(read(BUSY, &[], RATE_HOUR), Got::Wait(None));
    let headers = used("59m0.5s");
    let extra: Vec<(&str, &str)> = headers.iter().map(|(n, v)| (*n, v.as_str())).collect();
    let renews = NetFault::Allowance {
        renews_s: Some(3541),
    };
    assert_eq!(read(BUSY, &extra, RATE_HOUR), Got::Fault(renews));
    let words = br#"{"error":{"message":"Add credit or a payment in billing.","type":"requests","code":"rate_limit_exceeded"}}"#;
    assert_eq!(read(BUSY, &[], words), Got::Wait(None));
    assert_eq!(
        read(400, &[], words),
        Got::Fault(NetFault::Strange),
        "words never mean credit"
    );
}

#[test]
fn credit_and_a_spending_cap_are_told_only_by_their_exact_code_or_type() {
    let codes = [
        ("insufficient_quota", NetFault::Credit),
        ("credit_balance_exhausted", NetFault::Credit),
        ("organization_usage_limit_exceeded", NetFault::Credit),
        ("organization_spend_limit_exceeded", NetFault::Account),
        ("project_spend_limit_exceeded", NetFault::Account),
    ];
    for (code, fault) in codes {
        let by_code =
            serde_json::json!({"error": {"message": "m", "type": "requests", "code": code}});
        let by_type = serde_json::json!({"error": {"message": "m", "type": code}});
        for body in [by_code.to_string(), by_type.to_string()] {
            for status in [BUSY, 400] {
                let got = read(status, &[], body.as_bytes());
                assert_eq!(got, Got::Fault(fault), "{status} {body}");
            }
        }
    }
    let near = serde_json::json!({"error": {"message": "insufficient_quota", "code": "insufficient_quota_soon"}});
    assert_eq!(
        read(BUSY, &[], near.to_string().as_bytes()),
        Got::Wait(None)
    );
}
