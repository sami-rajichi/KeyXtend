//! Typing on the key page's lines through the hub (ADR-0049): the key line draws the typed key's dots and caret, never
//! its text; only the line taking the letters has a caret; each line's Save dims while it is empty; and a click on a
//! line or its Save sends that line, a line's click silently.

use super::super::{button, buttons, heard, listening_bar, page, panel, panel_json};
use super::lines::groq_lines;
use crate::Rig;
use crate::sound::{assert_plays, rig_with, takes};
use kx_module_api::Redacted;
use kx_module_api::ports::{CloudLine, VoiceAct, VoiceClick, VoiceSheet};
use kx_theme::sounds::Event;
use kx_ui::UiSettings;
use kx_ui::voice_acts::{CARET, SAVE, SERVICE};
use serde_json::Value;

/// The round of the view the tests click on.
const ROUND: u32 = 9;

/// The panel QML reads for Groq's key page with `typed` dots typed, the caret `at` on line `typing`, and `saves`.
fn typed(typed: u32, typing: CloudLine, at: u32, saves: &[CloudLine]) -> Value {
    let mut lines = groq_lines();
    lines.address = Some(Redacted::new("https://a.example.com/v1".to_owned()));
    (lines.typed, lines.typing, lines.caret) = (Redacted::new(typed), typing, Redacted::new(at));
    lines.saves = saves.to_vec();
    let rig = Rig::new();
    let heard = heard(&rig);
    panel_json(&rig, &heard, panel(VoiceSheet::Service(lines)))
}

/// Line `line`'s Save, acting or dimmed.
fn save(line: &str, on: bool) -> [String; 5] {
    button(SAVE, line, "Save", on, "plain")
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
    let paste = button(SERVICE, "paste", "Paste", true, "plain");
    let test = button(SERVICE, "test", "Test", true, "plain");
    assert_eq!(buttons(&key["buttons"]), [paste, save("key", true), test]);
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
fn each_lines_save_dims_while_it_is_empty() {
    let p = typed(0, CloudLine::Key, 0, &[CloudLine::Address]);
    let sheet = &p["sheet"];
    assert!(buttons(&sheet["key"]["buttons"]).contains(&save("key", false)));
    assert_eq!(
        buttons(&sheet["address"]["buttons"]),
        [save("address", true)]
    );
    assert_eq!(buttons(&sheet["model"]["buttons"]), [save("model", false)]);
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
        (SAVE, "key", 0, VoiceClick::Save(CloudLine::Key)),
        (SAVE, "model", 0, VoiceClick::Save(CloudLine::Model)),
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
        (SAVE, "keep", 0),
    ] {
        rig.hub.voice_act(kind, id, arg, i64::from(ROUND));
        assert!(heard.sent.acts().is_empty(), "{kind} {id} {arg}");
    }
}

#[test]
fn each_lines_save_has_its_own_name_for_readers() {
    let p = typed(0, CloudLine::Key, 0, &[]);
    let sheet = &p["sheet"];
    let save = |line: &str| {
        let all = sheet[line]["buttons"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        let it = all.into_iter().find(|b| b["kind"] == SAVE).unwrap();
        (
            it["label"].as_str().map(str::to_owned),
            it["name"].as_str().map(str::to_owned),
        )
    };
    let named = |name: &str| (Some("Save".to_owned()), Some(name.to_owned()));
    assert_eq!(
        save("key"),
        named("Save the key"),
        "a short label, a full name"
    );
    assert_eq!(save("address"), named("Save the address"));
    assert_eq!(save("model"), named("Save the model"));
}

/// A click on a line sends two carets, its pick and its place, so both are silent; its Save sounds as a key does.
#[test]
fn a_lines_carets_are_silent_and_its_save_plays_the_key_sound_once() {
    let (rig, _face, _host) = rig_with(|_| {});
    let heard = heard(&rig);
    heard.shown(&rig, page(ROUND, listening_bar(), None));
    rig.speaker.take_plays();
    for at in [24, 4] {
        rig.hub.voice_act(CARET, "address", at, i64::from(ROUND));
    }
    assert!(
        rig.speaker.take_plays().is_empty(),
        "a line's click is silent"
    );
    rig.hub.voice_act(SAVE, "address", 0, i64::from(ROUND));
    let volume = UiSettings::from_defaults().unwrap().sound_volume;
    assert_plays(&rig.speaker.take_plays(), &takes(Event::Key, 1, volume));
}
