//! Typing on the key page's lines through the hub (ADR-0049): the key line draws the typed key's dots and caret, never
//! its text; only the line taking the letters has a caret; each line's Save dims while it is empty; a click on a line
//! or its Save sends that line; a press on a line picks it as a click does; each Paste and Save names its line, and an
//! empty or busy Paste says so under its line.

use super::super::{button, buttons, heard, listening_bar, page, panel, panel_json};
use super::lines::groq_lines;
use crate::Rig;
use kx_module_api::Redacted;
use kx_module_api::ports::{
    AddressFault, CloudLine, PasteFault, VoiceAct, VoiceClick, VoicePanel, VoiceSheet,
};
use kx_ui::voice_acts::{CARET, PASTE, PICK, SAVE, SERVICE};
use serde_json::Value;

/// The round of the view the tests click on.
const ROUND: u32 = 9;

/// Groq's key page with `typed` dots typed, the caret `at` on line `typing`, and `saves`.
pub(in crate::voice) fn typed_panel(
    typed: u32,
    typing: CloudLine,
    at: u32,
    saves: &[CloudLine],
) -> VoicePanel {
    let mut lines = groq_lines();
    lines.address = Some(Redacted::new("https://a.example.com/v1".to_owned()));
    lines.lines = vec![CloudLine::Address, CloudLine::Model, CloudLine::Key];
    (lines.typed, lines.typing, lines.caret) = (Redacted::new(typed), typing, Redacted::new(at));
    lines.saves = saves.to_vec();
    panel(VoiceSheet::Service(lines))
}

/// The panel QML reads for that key page.
fn typed(typed: u32, typing: CloudLine, at: u32, saves: &[CloudLine]) -> Value {
    let rig = Rig::new();
    let heard = heard(&rig);
    panel_json(&rig, &heard, typed_panel(typed, typing, at, saves))
}

/// Line `line`'s Save, acting or dimmed.
fn save(line: &str, on: bool) -> [String; 5] {
    button(SAVE, line, "Save", on, "plain")
}

/// Line `line`'s Paste, which always acts.
fn paste(line: &str) -> [String; 5] {
    button(PASTE, line, "Paste", true, "plain")
}

#[test]
fn the_key_line_draws_the_typed_keys_dots_and_caret_and_never_its_text() {
    let p = typed(5, CloudLine::Key, 3, &[CloudLine::Key]);
    let key = &p["sheet"]["key"];
    let drawn = (
        key["typed"].as_u64(),
        key["at"].as_u64(),
        key["typing"].as_bool(),
    );
    assert_eq!(drawn, (Some(5), Some(3), Some(true)));
    assert_eq!(
        (key["dots"].as_u64(), key["text"].as_str()),
        (Some(56), Some("")),
        "the sealed key stays as dots"
    );
    let test = button(SERVICE, "test", "Test", true, "plain");
    let want = [paste("key"), save("key", true), test];
    assert_eq!(buttons(&key["buttons"]), want);
}

#[test]
fn only_the_line_taking_the_letters_has_a_caret() {
    let p = typed(5, CloudLine::Address, 7, &[]);
    let sheet = &p["sheet"];
    let line = |id: &str| (sheet[id]["typing"].as_bool(), sheet[id]["at"].as_u64());
    assert_eq!(line("address"), (Some(true), Some(7)));
    assert_eq!(line("key"), (Some(false), Some(0)));
    assert_eq!(line("model"), (Some(false), Some(0)));
    assert_eq!(
        sheet["address"]["typed"].as_u64(),
        Some(0),
        "only the key counts dots"
    );
}

#[test]
fn each_line_has_its_paste_and_a_save_that_dims_while_it_is_empty() {
    let p = typed(0, CloudLine::Key, 0, &[CloudLine::Address]);
    let sheet = &p["sheet"];
    assert!(buttons(&sheet["key"]["buttons"]).contains(&save("key", false)));
    let address = buttons(&sheet["address"]["buttons"]);
    assert_eq!(address, [paste("address"), save("address", true)]);
    let model = buttons(&sheet["model"]["buttons"]);
    assert_eq!(model, [paste("model"), save("model", false)]);
}

#[test]
fn a_click_on_a_line_places_its_caret_and_its_save_sends_the_line() {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard.shown(&rig, page(ROUND, listening_bar(), None));
    let caret = |line, at| VoiceClick::Caret {
        line,
        at: Redacted::new(at),
    };
    let cases = [
        (CARET, "address", 4, caret(CloudLine::Address, 4)),
        (CARET, "key", 0, caret(CloudLine::Key, 0)),
        (PICK, "model", 3, caret(CloudLine::Model, 3)),
        (SAVE, "key", 0, VoiceClick::Save(CloudLine::Key)),
        (SAVE, "model", 0, VoiceClick::Save(CloudLine::Model)),
        (PASTE, "address", 0, VoiceClick::Paste(CloudLine::Address)),
        (PASTE, "key", 0, VoiceClick::Paste(CloudLine::Key)),
    ];
    for (kind, id, arg, want) in cases {
        rig.hub.voice_act(kind, id, arg, i64::from(ROUND));
        let click = VoiceAct {
            round: ROUND,
            click: want,
        };
        assert_eq!(heard.sent.acts(), [click], "{kind} {id}");
    }
    for (kind, id, arg) in [
        (CARET, "address", -1),
        (CARET, "site", 0),
        (PICK, "model", -1),
        (SAVE, "keep", 0),
        (PASTE, "site", 0),
    ] {
        rig.hub.voice_act(kind, id, arg, i64::from(ROUND));
        assert!(heard.sent.acts().is_empty(), "{kind} {id} {arg}");
    }
}

#[test]
fn each_lines_paste_and_save_has_its_own_name_for_readers() {
    let p = typed(0, CloudLine::Key, 0, &[]);
    let sheet = &p["sheet"];
    let named = |line: &str, kind: &str| {
        let all = sheet[line]["buttons"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        let it = all.into_iter().find(|b| b["kind"] == kind).unwrap();
        (
            it["label"].as_str().map(str::to_owned),
            it["name"].as_str().map(str::to_owned),
        )
    };
    for (kind, label) in [(SAVE, "Save"), (PASTE, "Paste")] {
        for line in ["key", "address", "model"] {
            let want = (Some(label.to_owned()), Some(format!("{label} the {line}")));
            assert_eq!(named(line, kind), want, "a short label, a full name");
        }
    }
}

/// That key page with its address refused and line `line`'s Paste refused by `fault`.
fn pasted(line: CloudLine, fault: PasteFault) -> Value {
    let mut panel = typed_panel(0, CloudLine::Key, 0, &[]);
    if let VoiceSheet::Service(s) = &mut panel.sheet {
        (s.pasted, s.address_fault) = (Some((line, fault)), Some(AddressFault::Https));
    }
    let rig = Rig::new();
    let heard = heard(&rig);
    panel_json(&rig, &heard, panel)
}

#[test]
fn an_empty_or_busy_paste_says_so_under_the_line_pasted_in_place_of_its_refusal() {
    let empty = "The clipboard holds no text.";
    let busy = "The clipboard did not answer. Try Paste again.";
    let https = "The address must use HTTPS, such as https://example.com";
    for (at, line) in CloudLine::ALL.into_iter().enumerate() {
        for (fault, said) in [(PasteFault::Empty, empty), (PasteFault::Busy, busy)] {
            let p = pasted(line, fault);
            let got = ["key", "address", "model"].map(|id| p["sheet"][id]["fault"].clone());
            let mut want = [Value::Null, Value::from(https), Value::Null];
            want[at] = Value::from(said);
            assert_eq!(got, want, "{line:?} {fault:?}");
        }
    }
}
