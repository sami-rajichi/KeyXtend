//! Gemini's answers [N1][N5][N7]: its transcript from `output_text`, else from its steps' content in order; an empty
//! one is the run's to judge; a bad key by its exact reason and a leaked one by its exact status; a free tier asking to
//! verify and an unpaid balance by their reasons; each 429 a wait; a 503 retried as any 5xx; a refused location and a
//! refused language; anything else strange, a plain 400 or another page's 403 too; and its key test on the models list
//! [N8]. The fixtures under `replies/gemini/` are made up in Google's error format with no key; the reasons beside
//! [N7]'s are assumed, for the live check.

use super::super::table::{Extra, Got, read_with, tested_with};
use super::*;
use kx_ipc::NetFault::{Credit, Key, Language, Network, Region, Strange};
use proptest::prelude::*;

/// What the text fixtures say.
pub(super) const SAID: &str = "Bonjour tout le monde.";
/// The transcript under `output_text`, as its SDK reads it [N1].
pub(super) const TEXT: &[u8] = include_bytes!("../replies/gemini/text.json");
/// The transcript in its steps' content, as the sample's raw reply places it [N1], in two parts after the sound's.
const STEPS: &[u8] = include_bytes!("../replies/gemini/steps.json");
/// An empty transcript, and steps with no text.
pub(super) const EMPTY: &[u8] = include_bytes!("../replies/gemini/empty.json");
pub(super) const NO_STEP_TEXT: &[u8] = include_bytes!("../replies/gemini/no-step-text.json");
/// A bad key, by its exact reason.
const KEY_BAD: &[u8] = include_bytes!("../replies/gemini/key.json");
/// A leaked key's 403 [N7].
const LEAKED: &[u8] = include_bytes!("../replies/gemini/leaked.json");
/// The free tier asking the account to verify [N7], and an unpaid balance.
const VERIFY: &[u8] = include_bytes!("../replies/gemini/verify.json");
const BILLING: &[u8] = include_bytes!("../replies/gemini/billing.json");
/// A request it cannot read.
const SYNTAX: &[u8] = include_bytes!("../replies/gemini/syntax.json");
/// A location it does not serve.
const LOCATION: &[u8] = include_bytes!("../replies/gemini/location.json");
/// A language code it refuses.
const LANGUAGE_CODE: &[u8] = include_bytes!("../replies/gemini/language.json");
/// A used quota [N5], and the model overloaded.
const QUOTA: &[u8] = include_bytes!("../replies/gemini/quota.json");
const OVERLOADED: &[u8] = include_bytes!("../replies/gemini/overloaded.json");
/// The models list the key test reads [N8].
const MODELS_LIST: &[u8] = include_bytes!("../replies/gemini/models.json");
/// The exact reasons [N7] gives, and the one assumed, for no credit.
const PAY: [&str; 2] = ["FREE_TIER_VERIFICATION_REQUIRED", "BILLING_DISABLED"];

/// What Gemini's shape reads in a piece's reply of `status` with the headers `extra` and `body`.
fn read(status: u16, extra: Extra<'_>, body: &[u8]) -> Got {
    read_with(&Gemini, &gemini(ADDRESS), status, extra, body)
}

#[test]
fn a_reply_holds_its_text_in_output_text_else_in_its_steps() {
    assert_eq!(read(OK, &[], TEXT), Got::Text(SAID.to_owned()));
    assert_eq!(
        read(OK, &[], STEPS),
        Got::Text(SAID.to_owned()),
        "parts in order"
    );
    let both = br#"{"output_text":"first","steps":[{"content":[{"text":"second"}]}]}"#;
    assert_eq!(read(OK, &[], both), Got::Text("first".into()));
    let one = br#"{"steps":[{"content":{"type":"text","text":"one part"}}]}"#;
    assert_eq!(
        read(OK, &[], one),
        Got::Text("one part".into()),
        "content as one part"
    );
    let null = br#"{"output_text":null,"steps":[{"content":[{"text":"kept"}]}]}"#;
    assert_eq!(read(OK, &[], null), Got::Text("kept".into()));
}

#[test]
fn an_empty_transcript_is_the_runs_to_judge() {
    for body in [EMPTY, NO_STEP_TEXT, br#"{"steps":[]}"#] {
        let shown = String::from_utf8_lossy(body);
        assert_eq!(read(OK, &[], body), Got::Text(String::new()), "{shown}");
    }
}

#[test]
fn a_reply_that_is_not_its_json_is_strange() {
    let bodies: [&[u8]; 7] = [
        HOTSPOT,
        MODELS_LIST,
        b"",
        br#"{"text":"the form's"}"#,
        br#"["output_text"]"#,
        br#"{"output_text":7}"#,
        br#"{"steps":[{"content":"words"}]}"#,
    ];
    for body in bodies {
        let shown = String::from_utf8_lossy(body);
        assert_eq!(read(OK, &[], body), Got::Fault(Strange), "{shown}");
    }
}

#[test]
fn each_error_status_is_its_fault_as_its_error_pages_say() {
    let cases: [(u16, &[u8], Got); 17] = [
        (400, KEY_BAD, Got::Fault(Key)),
        (403, LEAKED, Got::Fault(Key)),
        (403, HOTSPOT, Got::Fault(Strange)),
        (401, b"", Got::Fault(Strange)),
        (
            403,
            br#"{"error":{"status":"PERMISSION_DENIED_SOON"}}"#,
            Got::Fault(Strange),
        ),
        (403, VERIFY, Got::Fault(Credit)),
        (403, BILLING, Got::Fault(Credit)),
        (400, LOCATION, Got::Fault(Region)),
        (400, LANGUAGE_CODE, Got::Fault(Language)),
        (BUSY, QUOTA, Got::Wait(None)),
        (500, DOWN, Got::Fault(Network)),
        (503, OVERLOADED, Got::Fault(Network)),
        (400, SYNTAX, Got::Fault(Strange)),
        (400, b"", Got::Fault(Strange)),
        (404, b"", Got::Fault(Strange)),
        (413, b"", Got::Fault(Strange)),
        (302, b"", Got::Fault(Strange)),
    ];
    for (status, body, want) in cases {
        let shown = String::from_utf8_lossy(body);
        assert_eq!(read(status, &[], body), want, "{status} {shown}");
    }
}

#[test]
fn a_429_waits_for_its_retry_after_and_a_503_is_tried_again() {
    let wait = [("retry-after", "3")];
    assert_eq!(read(BUSY, &wait, QUOTA), Got::Wait(Some(3)));
    let sending = Sending {
        shape: &Gemini,
        call: gemini(ADDRESS),
        model: Some(Cow::Borrowed(TRANSCRIBE)),
        lang: Cow::Borrowed(MSA),
        tries: 2,
        pace: None,
        dropped: Arc::new(AtomicBool::new(false)),
    };
    let http = Arc::new(FakeHttp::new([reply(503, OVERLOADED), reply(OK, TEXT)]));
    let clock = Arc::new(Still::default());
    let got = send::piece(&kit(&http, &clock), &sending, WAV);
    assert_eq!(got.map(|t| t.to_string()), Ok(SAID.to_owned()));
    assert_eq!(http.called().len(), 2);
}

#[test]
fn a_key_test_works_on_the_models_list_and_fails_as_a_piece_does() {
    let test = gemini(TEST);
    let tested = |status, body: &[u8]| tested_with(&Gemini, &test, status, &[], body);
    assert_eq!(tested(OK, MODELS_LIST), Got::Text(String::new()));
    assert_eq!(tested(OK, HOTSPOT), Got::Fault(Strange));
    assert_eq!(tested(400, KEY_BAD), Got::Fault(Key));
    assert_eq!(tested(403, VERIFY), Got::Fault(Credit));
    assert_eq!(tested(BUSY, QUOTA), Got::Wait(None));
    assert_eq!(tested(503, OVERLOADED), Got::Fault(Network));
}

/// A Google error body of `status`, with the canonical status Google gives it and `reason` as its one detail's reason.
fn reasoned(status: u16, reason: &str) -> Vec<u8> {
    let canonical = match status {
        403 => "PERMISSION_DENIED",
        BUSY => "RESOURCE_EXHAUSTED",
        _ => "INVALID_ARGUMENT",
    };
    let detail = json!({ "@type": "type.googleapis.com/google.rpc.ErrorInfo", "reason": reason });
    let error =
        json!({ "code": status, "message": "kx test", "status": canonical, "details": [detail] });
    json!({ "error": error }).to_string().into_bytes()
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
        let call = gemini(ADDRESS);
        let _ = Gemini.read(&call, &head, &body);
        let _ = Gemini.tested(&call, &head, &body);
    }

    #[test]
    fn credit_and_a_400_key_are_told_only_by_their_exact_reasons(
        reason in prop_oneof![
            Just("FREE_TIER_VERIFICATION_REQUIRED".to_owned()),
            Just("BILLING_DISABLED".to_owned()),
            Just("API_KEY_INVALID".to_owned()),
            Just("api_key_invalid".to_owned()),
            Just("INVALID_ARGUMENT".to_owned()),
            "[A-Z_]{0,32}",
        ],
        status in prop_oneof![Just(400u16), Just(403), Just(BUSY)],
    ) {
        let pay = PAY.iter().any(|p| p.eq_ignore_ascii_case(&reason));
        let key = reason.eq_ignore_ascii_case("API_KEY_INVALID");
        // A 403 is PERMISSION_DENIED, the key's, unless its reason asks for payment; a 400 is the key's only by reason.
        let want = match status {
            _ if pay => Got::Fault(Credit),
            _ if key => Got::Fault(Key),
            403 => Got::Fault(Key),
            BUSY => Got::Wait(None),
            _ => Got::Fault(Strange),
        };
        prop_assert_eq!(read(status, &[], &reasoned(status, &reason)), want);
    }

    #[test]
    fn any_transcript_reads_back_as_itself_from_either_place(text in "\\PC{0,64}") {
        let said = json!({ "output_text": text }).to_string();
        prop_assert_eq!(read(OK, &[], said.as_bytes()), Got::Text(text.clone()));
        let steps = json!({ "steps": [{ "content": [{ "type": "text", "text": text }] }] });
        prop_assert_eq!(read(OK, &[], steps.to_string().as_bytes()), Got::Text(text));
    }
}
