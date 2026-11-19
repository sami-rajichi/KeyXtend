//! `ElevenLabs`' answers read as its error page says [E8]: its text, a bad key, a plain 401 still the key, a used quota
//! or no credit by the exact code its `detail` names, each 429 a wait, too big, a language refused, a service down, and
//! anything else strange; its key test on the subscription [E9]. The fixtures under `replies/elevenlabs/` hold made-up
//! ids and no key; [E8] names the codes only, so each error body is assumed, for the live check.

use super::super::table::{Extra, Got, read_with, tested_with};
use super::*;
use kx_ipc::NetFault::{Credit, Key, Language, Network, Strange, TooBig};
use proptest::prelude::*;

/// The text with its word timings, as the reference's sample gives it [E3].
pub(super) const TEXT: &[u8] = include_bytes!("../replies/elevenlabs/text.json");
/// A bad key, in the newer `detail.code` shape.
const KEY_BAD: &[u8] = include_bytes!("../replies/elevenlabs/key.json");
/// A used quota, in the older `detail.status` shape.
const QUOTA: &[u8] = include_bytes!("../replies/elevenlabs/quota.json");
const PAYMENT: &[u8] = include_bytes!("../replies/elevenlabs/payment.json");
const CREDITS: &[u8] = include_bytes!("../replies/elevenlabs/credits.json");
const CONCURRENT: &[u8] = include_bytes!("../replies/elevenlabs/concurrent.json");
const TOO_MANY: &[u8] = include_bytes!("../replies/elevenlabs/too-many.json");
const SYSTEM: &[u8] = include_bytes!("../replies/elevenlabs/busy.json");
/// A 422 naming `language_code` in its `detail[]`, assumed from its validation errors.
const LANGUAGE_CODE: &[u8] = include_bytes!("../replies/elevenlabs/language.json");
/// The subscription the key test reads, with the quota's renewal [E9].
const SUBSCRIPTION: &[u8] = include_bytes!("../replies/elevenlabs/subscription.json");
/// The codes [E8] gives for a used quota or no credit.
const CREDIT_CODES: [&str; 3] = ["quota_exceeded", "payment_required", "insufficient_credits"];

/// What `ElevenLabs`' form reads in a piece's reply of `status` with the headers `extra` and `body`.
fn read(status: u16, extra: Extra<'_>, body: &[u8]) -> Got {
    read_with(&Eleven, &eleven(ADDRESS), status, extra, body)
}

#[test]
fn a_reply_holds_its_text_beside_its_word_timings() {
    assert_eq!(read(OK, &[], TEXT), Got::Text(SAID.to_owned()));
    assert_eq!(
        read(OK, &[], BLANK),
        Got::Text("  ".into()),
        "the run judges blanks"
    );
    for body in [
        HOTSPOT,
        SUBSCRIPTION,
        b"",
        br#"{"text":null}"#,
        br#"["text"]"#,
    ] {
        let shown = String::from_utf8_lossy(body);
        assert_eq!(read(OK, &[], body), Got::Fault(Strange), "{shown}");
    }
}

#[test]
fn each_error_status_is_its_fault_as_its_error_page_says() {
    let cases: [(u16, &[u8], Got); 17] = [
        (401, KEY_BAD, Got::Fault(Key)),
        (401, b"", Got::Fault(Key)),
        (401, HOTSPOT, Got::Fault(Key)),
        (401, QUOTA, Got::Fault(Credit)),
        (402, PAYMENT, Got::Fault(Credit)),
        (402, CREDITS, Got::Fault(Credit)),
        (402, b"", Got::Fault(Credit)),
        (BUSY, CONCURRENT, Got::Wait(None)),
        (BUSY, TOO_MANY, Got::Wait(None)),
        (BUSY, SYSTEM, Got::Wait(None)),
        (413, b"", Got::Fault(TooBig)),
        (422, LANGUAGE_CODE, Got::Fault(Language)),
        (400, KEY_BAD, Got::Fault(Strange)),
        (403, HOTSPOT, Got::Fault(Strange)),
        (500, DOWN, Got::Fault(Network)),
        (503, SYSTEM, Got::Fault(Network)),
        (302, b"", Got::Fault(Strange)),
    ];
    for (status, body, want) in cases {
        let shown = String::from_utf8_lossy(body);
        assert_eq!(read(status, &[], body), want, "{status} {shown}");
    }
}

#[test]
fn a_429_waits_for_its_retry_after_and_a_quota_in_one_is_credit() {
    let wait = [("retry-after", "3")];
    assert_eq!(read(BUSY, &wait, CONCURRENT), Got::Wait(Some(3)));
    assert_eq!(
        read(BUSY, &wait, QUOTA),
        Got::Fault(Credit),
        "its exact code"
    );
}

#[test]
fn credit_is_told_only_by_its_exact_code() {
    let near = br#"{"detail":{"status":"quota_exceeded_soon","message":"quota_exceeded"}}"#;
    assert_eq!(
        read(401, &[], near),
        Got::Fault(Key),
        "a near code stays the key"
    );
    let upper = br#"{"detail":{"status":"QUOTA_EXCEEDED"}}"#;
    assert_eq!(read(401, &[], upper), Got::Fault(Credit), "in any case");
    let elsewhere = br#"{"error":{"code":"quota_exceeded"}}"#;
    assert_eq!(
        read(401, &[], elsewhere),
        Got::Fault(Key),
        "only under detail"
    );
}

#[test]
fn a_key_test_works_on_the_subscription_and_fails_as_a_piece_does() {
    let test = eleven(TEST);
    let tested = |status, body: &[u8]| tested_with(&Eleven, &test, status, &[], body);
    assert_eq!(tested(OK, SUBSCRIPTION), Got::Text(String::new()));
    assert_eq!(tested(OK, HOTSPOT), Got::Fault(Strange));
    assert_eq!(tested(401, KEY_BAD), Got::Fault(Key));
    assert_eq!(tested(401, QUOTA), Got::Fault(Credit));
    assert_eq!(tested(BUSY, SYSTEM), Got::Wait(None));
    assert_eq!(tested(503, DOWN), Got::Fault(Network));
}

/// An error body with `code` under `detail.<under>`.
fn detail(under: &str, code: &str) -> Vec<u8> {
    let mut inner = serde_json::Map::new();
    inner.insert(under.to_owned(), code.into());
    serde_json::json!({ "detail": inner })
        .to_string()
        .into_bytes()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn a_random_reply_never_panics_its_readers(
        status in any::<u16>(),
        body in proptest::collection::vec(any::<u8>(), 0..512),
        value in ".{0,24}",
    ) {
        let head = head(status, &[("retry-after", value.as_str())]);
        let call = eleven(ADDRESS);
        let _ = Eleven.read(&call, &head, &body);
        let _ = Eleven.tested(&call, &head, &body);
    }

    #[test]
    fn a_401_or_429_is_credit_only_by_an_exact_code_and_a_402_always(
        code in prop_oneof![
            Just("quota_exceeded".to_owned()),
            Just("payment_required".to_owned()),
            Just("insufficient_credits".to_owned()),
            Just("invalid_api_key".to_owned()),
            ".{0,24}",
        ],
        under in prop_oneof![Just("status"), Just("code"), Just("type"), Just("message")],
        status in prop_oneof![Just(401u16), Just(402), Just(BUSY)],
    ) {
        let coded = matches!(under, "status" | "code");
        let credit = coded && CREDIT_CODES.iter().any(|c| c.eq_ignore_ascii_case(&code));
        let want = match status {
            401 if !credit => Got::Fault(Key),
            BUSY if !credit => Got::Wait(None),
            _ => Got::Fault(Credit),
        };
        prop_assert_eq!(read(status, &[], &detail(under, &code)), want);
    }
}
