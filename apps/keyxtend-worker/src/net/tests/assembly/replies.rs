//! `AssemblyAI`'s answers read as its pages say [A1][A2]: its text beside its words, an empty transcript left to the
//! run, and each problem (RFC 9457) by its status, a 400 as the language only when its title or detail names it and
//! never as the key, a 402 as no credit or the account's step, and a 429 or a 503 waiting by its `Retry-After`.
//! The fixtures under `replies/assemblyai/` are made up and hold no key; the undocumented credit reply is assumed.

use super::super::table::{Extra, Got, read_with, tested_with};
use super::*;
use kx_ipc::NetFault::{Account, Credit, Key, Language, Network, Strange, TooBig};

/// The text with its words, as the reference gives a reply [A1].
pub(super) const TEXT: &[u8] = include_bytes!("../replies/assemblyai/text.json");
/// An empty transcript, as for silence.
const EMPTY: &[u8] = include_bytes!("../replies/assemblyai/empty.json");
const KEY_BAD: &[u8] = include_bytes!("../replies/assemblyai/key.json");
/// A code outside the language set [A1].
const LANGUAGE: &[u8] = include_bytes!("../replies/assemblyai/language.json");
/// Malformed audio, whose words hold "decode" but name no language.
const AUDIO: &[u8] = include_bytes!("../replies/assemblyai/audio.json");
/// The same, its problem type's address naming a language, which is not its words.
const TYPED: &[u8] = include_bytes!("../replies/assemblyai/typed.json");
const FORMAT: &[u8] = include_bytes!("../replies/assemblyai/format.json");
const TOO_LONG: &[u8] = include_bytes!("../replies/assemblyai/too-long.json");
const RATE: &[u8] = include_bytes!("../replies/assemblyai/rate.json");
/// A cold start or the concurrency cap.
const COLD: &[u8] = include_bytes!("../replies/assemblyai/busy.json");
const MODEL: &[u8] = include_bytes!("../replies/assemblyai/model.json");
/// The 30-second server deadline passed [A2].
const DEADLINE: &[u8] = include_bytes!("../replies/assemblyai/deadline.json");
const CREDIT: &[u8] = include_bytes!("../replies/assemblyai/credit.json");
const PAYMENT: &[u8] = include_bytes!("../replies/assemblyai/payment.json");
/// A wait given as a date, which names no seconds.
const DATED: &str = "Wed, 21 Oct 2026 07:28:00 GMT";

/// What `AssemblyAI`'s shape reads in a piece's reply of `status` with the headers `extra` and `body`.
fn read(status: u16, extra: Extra<'_>, body: &[u8]) -> Got {
    read_with(&Assembly, &assembly(ADDRESS), status, extra, body)
}

#[test]
fn a_reply_holds_its_text_beside_its_words_and_an_empty_one_is_the_runs_to_judge() {
    assert_eq!(read(OK, &[], TEXT), Got::Text(SAID.to_owned()));
    assert_eq!(read(OK, &[], EMPTY), Got::Text(String::new()));
    for body in [HOTSPOT, KEY_BAD, b"", br#"{"text":null}"#, br#"["text"]"#] {
        let shown = String::from_utf8_lossy(body);
        assert_eq!(read(OK, &[], body), Got::Fault(Strange), "{shown}");
    }
}

#[test]
fn each_problem_is_its_fault_by_its_status_as_its_pages_say() {
    let cases: [(u16, &[u8], Got); 20] = [
        (401, KEY_BAD, Got::Fault(Key)),
        (401, b"", Got::Fault(Key)),
        (401, HOTSPOT, Got::Fault(Key)),
        (402, CREDIT, Got::Fault(Credit)),
        (402, b"", Got::Fault(Credit)),
        (402, PAYMENT, Got::Fault(Account)),
        (413, TOO_LONG, Got::Fault(TooBig)),
        (400, LANGUAGE, Got::Fault(Language)),
        (400, AUDIO, Got::Fault(Strange)),
        (400, TYPED, Got::Fault(Strange)),
        (400, KEY_BAD, Got::Fault(Strange)),
        (400, b"", Got::Fault(Strange)),
        (415, FORMAT, Got::Fault(Strange)),
        (BUSY, RATE, Got::Wait(None)),
        (500, MODEL, Got::Fault(Network)),
        (503, COLD, Got::Fault(Network)),
        (504, DEADLINE, Got::Fault(Network)),
        (502, HOTSPOT, Got::Fault(Strange)),
        (403, HOTSPOT, Got::Fault(Strange)),
        (302, b"", Got::Fault(Strange)),
    ];
    for (status, body, want) in cases {
        let shown = String::from_utf8_lossy(body);
        assert_eq!(read(status, &[], body), want, "{status} {shown}");
    }
}

#[test]
fn a_429_waits_by_its_retry_after_and_a_503_only_when_it_names_seconds() {
    let wait = |s| [("retry-after", s)];
    assert_eq!(read(BUSY, &wait("3"), RATE), Got::Wait(Some(3)));
    assert_eq!(read(BUSY, &wait(DATED), RATE), Got::Wait(None));
    assert_eq!(read(503, &wait("5"), COLD), Got::Wait(Some(5)));
    let dated = read(503, &wait(DATED), COLD);
    assert_eq!(dated, Got::Fault(Network), "tried again as a 500 is");
}

#[test]
fn a_400s_language_and_a_402s_payment_method_are_read_only_in_its_title_or_detail() {
    let problem = |title: &str, detail: &str| {
        let body = serde_json::json!({ "status": 400, "title": title, "detail": detail });
        body.to_string().into_bytes()
    };
    let titled = problem("Unsupported Language", "kx");
    assert_eq!(read(400, &[], &titled), Got::Fault(Language));
    let detailed = problem("Bad Request", "language_codes holds kx");
    assert_eq!(read(400, &[], &detailed), Got::Fault(Language));
    let elsewhere = br#"{"status":400,"title":"Bad Request","error":"language"}"#;
    assert_eq!(read(400, &[], elsewhere), Got::Fault(Strange));
    let method = problem("Payment Required", "Add a PAYMENT METHOD");
    assert_eq!(read(402, &[], &method), Got::Fault(Account), "in any case");
}

#[test]
fn a_key_test_reply_reads_through_the_same_table() {
    let call = assembly(ADDRESS);
    let tested = |status, body: &[u8]| tested_with(&Assembly, &call, status, &[], body);
    assert_eq!(tested(401, KEY_BAD), Got::Fault(Key));
    assert_eq!(tested(402, PAYMENT), Got::Fault(Account));
    assert_eq!(tested(OK, HOTSPOT), Got::Fault(Strange));
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
        let call = assembly(ADDRESS);
        let _ = Assembly.read(&call, &head, &body);
        let _ = Assembly.tested(&call, &head, &body);
    }

    #[test]
    fn only_a_401_is_the_key_and_only_a_402_no_credit_or_the_accounts_step(
        status in 300u16..600,
        title in ".{0,24}",
        detail in prop_oneof![Just("payment method".to_owned()), Just("language".to_owned()), ".{0,48}"],
    ) {
        let body = serde_json::json!({ "status": status, "title": title, "detail": detail });
        let got = read(status, &[], body.to_string().as_bytes());
        prop_assert_eq!(got == Got::Fault(Key), status == 401);
        prop_assert_eq!(matches!(got, Got::Fault(Credit | Account)), status == 402);
        let language = got == Got::Fault(Language);
        prop_assert!(!language || status == 400, "{status}: only a 400 names the language");
    }
}
