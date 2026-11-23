//! A service's key page through the hub: the key as dots with Paste and Test and the test's answer, a refusal by the
//! note of its reason with a used allowance's renewal, where keys are made and that the key is sealed, a refused
//! address or model or a busy Paste each with its reason (carries 1, 5), the keep-nothing switch with a model no host
//! of which keeps nothing (edge 17), and a typed address's lines in their order.

use super::super::{button, buttons, heard, panel, panel_json};
use super::{groq, marks, pairs};
use crate::Rig;
use kx_module_api::Redacted;
use kx_module_api::ports::{
    AddressFault, CloudButton, CloudLine, KeyCheck, LocalTime, PasteFault, Renewal, ServiceMark,
    ServiceName, ServiceSheet, VoiceSheet,
};
use kx_ui::voice_acts::{KEEP, PASTE, SAVE, SERVICE};
use serde_json::{Value, json};

/// Groq's key page: a 56-character key that works, nothing typed on the model line.
pub(super) fn groq_lines() -> ServiceSheet {
    ServiceSheet {
        service: groq(),
        mark: ServiceMark::Ready,
        key: Redacted::new(56),
        typed: Redacted::new(0),
        check: KeyCheck::Works,
        renews_at: None,
        address: None,
        address_fault: None,
        model: Redacted::new(String::new()),
        model_na: false,
        model_refused: false,
        keep: None,
        site: Some("console.groq.com".to_owned()),
        tests: true,
        acts: CloudButton::ALL.to_vec(),
        lines: vec![CloudLine::Key, CloudLine::Model],
        typing: CloudLine::Key,
        caret: Redacted::new(0),
        saves: Vec::new(),
        pasted: None,
    }
}

/// The Custom row's page: an address that is not HTTPS, a refused model id, and a key Paste the clipboard did not
/// answer, whose note shows in place of the test's answer.
fn custom_lines() -> ServiceSheet {
    let mut lines = groq_lines();
    (lines.service, lines.mark, lines.check) = (
        ServiceName::Custom,
        ServiceMark::NeedsAddress,
        KeyCheck::Untested,
    );
    lines.pasted = Some((CloudLine::Key, PasteFault::Busy));
    lines.address = Some(Redacted::new("http://lan.example".to_owned()));
    lines.address_fault = Some(AddressFault::Https);
    lines.model = Redacted::new("no such/model".to_owned());
    (lines.model_refused, lines.site) = (true, None);
    lines.lines = vec![CloudLine::Address, CloudLine::Model, CloudLine::Key];
    lines
}

/// The panel QML reads for `lines`.
fn lines_json(lines: ServiceSheet) -> Value {
    let rig = Rig::new();
    let heard = heard(&rig);
    panel_json(&rig, &heard, panel(VoiceSheet::Service(lines)))
}

/// `v` as a list of one, for `buttons` and `marks`.
fn one(v: &Value) -> Value {
    Value::Array(vec![v.clone()])
}

#[test]
fn the_key_page_shows_the_key_as_dots_with_paste_and_test() {
    let p = lines_json(groq_lines());
    assert_eq!(
        (p["page"].as_str(), p["sheet"]["is"].as_str()),
        (Some("service"), Some("service"))
    );
    assert_eq!(
        (p["title"].as_str(), p["sub"].as_str()),
        (Some("Groq"), Some("Ready"))
    );
    let key = &p["sheet"]["key"];
    assert_eq!(
        (key["caption"].as_str(), key["hint"].as_str()),
        (Some("Key"), Some("Type or paste your key"))
    );
    assert_eq!(
        (key["dots"].as_u64(), key["text"].as_str()),
        (Some(56), Some(""))
    );
    assert_eq!(
        key["mask"], "••••••••",
        "the mask PanelLine draws its dots from"
    );
    let save = button(SAVE, "key", "Save", false, "plain");
    let paste = button(PASTE, "key", "Paste", true, "plain");
    let test = button(SERVICE, "test", "Test", true, "plain");
    assert_eq!(buttons(&key["buttons"]), [paste, save, test]);
    assert_eq!(
        marks(&one(&p["sheet"]["check"])),
        pairs(&[("ok", "Key works")])
    );
    assert_eq!(p["sheet"]["address"], Value::Null);
    let model = &p["sheet"]["model"];
    assert_eq!(
        (model["caption"].as_str(), model["fault"].as_null()),
        (Some("Other model"), Some(()))
    );
    assert_eq!(model["mask"], Value::Null, "only the key hides");
    assert_eq!(p["sheet"]["keep"], Value::Null);
}

#[test]
fn a_row_with_no_test_call_shows_paste_and_save_with_no_test() {
    let mut lines = groq_lines();
    (lines.tests, lines.check) = (false, KeyCheck::NoTest);
    let p = lines_json(lines);
    let kinds: Vec<_> = (p["sheet"]["key"]["buttons"].as_array().unwrap().iter())
        .map(|b| b["kind"].as_str().unwrap_or_default().to_owned())
        .collect();
    assert_eq!(kinds, [PASTE, SAVE], "Paste, then Save");
}

#[test]
fn the_key_page_says_where_keys_are_made_and_that_the_key_is_sealed() {
    let p = lines_json(groq_lines());
    let link = &p["sheet"]["link"];
    assert_eq!(link["text"], "Make a key at console.groq.com");
    let open = button(SERVICE, "link", "Open", true, "plain");
    assert_eq!(buttons(&one(&link["button"])), [open]);
    assert_eq!(
        p["line"],
        "The key is kept encrypted on this PC and never shown."
    );
    let foot = [
        button(SERVICE, "facts", "Facts", true, "plain"),
        button(SERVICE, "remove", "Remove key", true, "danger"),
        button(SERVICE, "off", "Switch off", true, "plain"),
    ];
    assert_eq!(buttons(&p["foot"]), foot);
}

/// A used allowance's renewal: 17:00 UTC on 14 November 2026.
const RENEWS_MS: i64 = 1_794_675_600_000;

/// Each test answer with a reason, its id and its line, which is that reason's cloud note; and no answer's and a
/// management key's own lines.
const ANSWERS: [(KeyCheck, &str, &str); 8] = [
    (
        KeyCheck::Credit,
        "credit",
        "The account has no credit left.",
    ),
    (
        KeyCheck::Account,
        "account",
        "The service asks for a step on its site, such as a verification, a payment method or a spending cap.",
    ),
    (
        KeyCheck::Region,
        "region",
        "The service is not offered in your country or region.",
    ),
    (
        KeyCheck::Allowance,
        "allowance",
        "The free allowance is used up for now.",
    ),
    (
        KeyCheck::Rate,
        "rate",
        "Too many requests. Wait a moment and try again.",
    ),
    (
        KeyCheck::Strange,
        "strange",
        "The reply was not the service’s, maybe a sign-in page. Check the network.",
    ),
    (
        KeyCheck::Failed,
        "failed",
        "No answer about the key. Check the network and test again.",
    ),
    (
        KeyCheck::Admin,
        "admin",
        "This is a management key, which cannot be used for dictation. Make an API key on the service’s page.",
    ),
];

/// The key test's answer `check` on Groq's key page, renewing at `renews_at`, as its id and line.
fn answer(check: KeyCheck, renews_at: Option<Renewal>) -> Vec<(String, String)> {
    let mut lines = groq_lines();
    (lines.check, lines.renews_at) = (check, renews_at);
    marks(&one(&lines_json(lines)["sheet"]["check"]))
}

#[test]
fn a_tests_refusal_reads_as_the_cloud_note_of_its_reason_and_no_answer_as_its_own() {
    for (check, id, text) in ANSWERS {
        assert_eq!(answer(check, None), pairs(&[(id, text)]), "{check:?}");
    }
}

#[test]
fn a_used_allowance_on_test_names_its_renewal_when_the_service_told() {
    let at = LocalTime::at(RENEWS_MS, 0).unwrap();
    let today = "The free allowance is used up. It renews at 17:00.";
    let got = answer(KeyCheck::Allowance, Some(Renewal { at, today: true }));
    assert_eq!(got, pairs(&[("allowance", today)]));
    let later = "The free allowance is used up. It renews on 14/11/2026 at 17:00.";
    let got = answer(KeyCheck::Allowance, Some(Renewal { at, today: false }));
    assert_eq!(got, pairs(&[("allowance", later)]), "a later day");
}

#[test]
fn a_refused_address_model_or_busy_paste_each_says_why_on_its_own_line() {
    let p = lines_json(custom_lines());
    let sheet = &p["sheet"];
    assert_eq!(
        (p["title"].as_str(), p["sub"].as_str()),
        (Some("Custom address"), Some("Needs an address"))
    );
    let busy = "The clipboard did not answer. Try Paste again.";
    assert_eq!(sheet["key"]["fault"], busy, "the key Paste's note");
    assert_eq!(sheet["check"], Value::Null, "in the answer's place");
    let address = &sheet["address"];
    assert_eq!(address["text"], "http://lan.example");
    let hint = "Your service’s address, such as https://example.com/v1/audio/transcriptions";
    assert_eq!(address["hint"], hint);
    let https = "The address must use HTTPS, such as https://example.com";
    assert_eq!(address["fault"], https);
    assert_eq!(sheet["model"]["text"], "no such/model");
    let refused = "This model id is too long or holds a character that is not allowed.";
    assert_eq!(sheet["model"]["fault"], refused);
    assert_eq!(sheet["link"], Value::Null);
}

#[test]
fn keep_nothing_says_why_a_model_with_no_such_host_cannot_be_used() {
    let mut lines = custom_lines();
    (lines.model_refused, lines.model_na, lines.keep) = (false, true, Some(true));
    let p = lines_json(lines);
    let none = "No host of this model promises to keep nothing.";
    assert_eq!(p["sheet"]["model"]["fault"], none, "edge 17");
    let keep = &p["sheet"]["keep"];
    assert_eq!(
        (keep["label"].as_str(), keep["on"].as_bool()),
        (Some("Only hosts that keep nothing"), Some(true))
    );
    assert_eq!(keep["kind"], KEEP);
}

#[test]
fn a_typed_addresss_key_page_shows_its_address_model_and_key_in_that_order() {
    let p = lines_json(custom_lines());
    let sheet = &p["sheet"];
    assert_eq!(sheet["order"], json!(["address", "model", "key"]));
    for id in ["address", "model", "key"] {
        assert!(sheet[id].is_object(), "{id} on the one page");
    }
    let p = lines_json(groq_lines());
    assert_eq!(p["sheet"]["order"], json!(["key", "model"]), "Groq's");
}

/// The test's answer shows lit only when the key works, as a warning band when it warns, and plain while it tests or the
/// row has none, so QML holds no list of answers.
#[test]
fn each_key_tests_answer_tells_qml_whether_it_works_or_warns() {
    for check in KeyCheck::ALL {
        let mut lines = groq_lines();
        lines.check = check;
        let c = &lines_json(lines)["sheet"]["check"];
        let want = match check {
            KeyCheck::Untested => (None, None),
            KeyCheck::Works => (Some(true), Some(false)),
            KeyCheck::Testing | KeyCheck::NoTest => (Some(false), Some(false)),
            _ => (Some(false), Some(true)),
        };
        assert_eq!(
            (c["works"].as_bool(), c["warns"].as_bool()),
            want,
            "{check:?}"
        );
    }
}
