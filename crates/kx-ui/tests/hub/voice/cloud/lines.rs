//! A service's key page through the hub: the key as dots with Paste and Test and the test's answer, where keys are
//! made and that the key is sealed, a refused address or model or a busy Paste each with its reason (carries 1, 5), and
//! the keep-nothing switch with a model no host of which keeps nothing (edge 17).

use super::super::{button, buttons, heard, panel, panel_json};
use super::{groq, marks, pairs};
use crate::Rig;
use kx_module_api::Redacted;
use kx_module_api::ports::{
    AddressFault, CloudButton, KeyCheck, ServiceMark, ServiceName, ServiceSheet, VoiceSheet,
};
use kx_ui::voice_acts::{KEEP, SERVICE};
use serde_json::Value;

/// Groq's key page: a 56-character key that works, nothing typed on the model line.
fn groq_lines() -> ServiceSheet {
    ServiceSheet {
        service: groq(),
        mark: ServiceMark::Ready,
        key: Redacted::new(56),
        check: KeyCheck::Works,
        address: None,
        address_fault: None,
        model: Redacted::new(String::new()),
        model_na: false,
        model_refused: false,
        keep: None,
        site: Some("console.groq.com".to_owned()),
        acts: CloudButton::ALL.to_vec(),
    }
}

/// The Custom row's page: an address that is not HTTPS, a refused model id, and a Paste the clipboard did not answer.
fn custom_lines() -> ServiceSheet {
    let mut lines = groq_lines();
    (lines.service, lines.mark, lines.check) = (
        ServiceName::Custom,
        ServiceMark::NeedsAddress,
        KeyCheck::Busy,
    );
    lines.address = Some(Redacted::new("http://lan.example".to_owned()));
    lines.address_fault = Some(AddressFault::Https);
    lines.model = Redacted::new("no such/model".to_owned());
    (lines.model_refused, lines.site) = (true, None);
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
    let paste = button(SERVICE, "paste", "Paste", true, "plain");
    assert_eq!(
        buttons(&key["buttons"]),
        [paste, button(SERVICE, "test", "Test", true, "plain")]
    );
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

#[test]
fn a_refused_address_model_or_busy_paste_each_says_why_on_its_own_line() {
    let p = lines_json(custom_lines());
    let sheet = &p["sheet"];
    assert_eq!(
        (p["title"].as_str(), p["sub"].as_str()),
        (Some("Custom address"), Some("Needs an address"))
    );
    let busy = "The clipboard did not answer. Try Paste again.";
    assert_eq!(marks(&one(&sheet["check"])), pairs(&[("busy", busy)]));
    let address = &sheet["address"];
    assert_eq!(address["text"], "http://lan.example");
    let hint = "Your service’s address, such as https://example.com/v1/audio/transcriptions";
    assert_eq!(address["hint"], hint);
    assert_eq!(address["fault"], "The address must start with https://");
    assert_eq!(sheet["model"]["text"], "no such/model");
    let refused = "This model id is too long or holds a character it may not.";
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
