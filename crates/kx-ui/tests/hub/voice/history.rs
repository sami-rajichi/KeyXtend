//! History: each text with its direction, its line naming when, the language and the model, the name readers hear,
//! Delete all in the header, Copy, Pin, Delete and Insert in the footer, Delete all's question, and an empty list.

use super::{acting, button, buttons, heard, panel_json};
use crate::Rig;
use kx_module_api::Redacted;
use kx_module_api::ports::{Date, HistoryRow, HistorySheet, PanelButton, VoiceSheet, When};
use kx_ui::i18n::keys::VOICE_EMPTY;
use serde_json::{Value, json};

/// Today at 09:05.
fn today() -> When {
    When {
        days_ago: 0,
        date: Date::new(2026, 11, 15).unwrap(),
        hour: 9,
        minute: 5,
    }
}

/// A pinned French text, first in the tick order.
fn bonjour() -> HistoryRow {
    HistoryRow {
        id: 7,
        text: Redacted::new("Bonjour".to_owned()),
        rtl: false,
        arabic: false,
        when: today(),
        lang: "fr".to_owned(),
        model: Redacted::new("Whisper small".to_owned()),
        pinned: true,
        tick: Some(1),
    }
}

/// History with `rows` of `total`, `unpinned` of them unpinned, Unpin and the question as given.
fn sheet(rows: Vec<HistoryRow>, total: u32, unpin: bool, confirm: bool) -> VoiceSheet {
    VoiceSheet::History(HistorySheet {
        rows,
        total,
        unpinned: 11,
        unpin,
        confirm,
    })
}

/// The panel QML reads for `sheet`, `ticked` rows ticked and every button acting.
fn page(sheet: VoiceSheet, ticked: u16) -> Value {
    let rig = Rig::new();
    let heard = heard(&rig);
    let mut shown = acting(sheet, &PanelButton::ALL);
    shown.ticked = ticked;
    panel_json(&rig, &heard, shown)
}

#[test]
fn each_text_shows_its_line_and_the_name_readers_hear() {
    let mut german = bonjour();
    (german.lang, german.pinned, german.tick) = ("de".to_owned(), false, None);
    let p = page(sheet(vec![bonjour(), german], 12, true, false), 1);
    assert_eq!(
        (&p["page"], &p["title"]),
        (&json!("history"), &json!("History"))
    );
    assert_eq!(p["sub"], "12 texts · newest first");
    let when = rig_when();
    let meta = format!("{when} · French · Whisper small");
    let name = format!("Bonjour, {meta}, Ticked 1, Pinned");
    let first = json!({"id": 7, "text": "Bonjour", "rtl": false, "arabic": false, "meta": meta, "pinned": true,
        "tick": 1, "name": name});
    assert_eq!(p["sheet"]["rows"][0], first);
    let other = &p["sheet"]["rows"][1];
    assert_eq!(
        other["meta"],
        format!("{when} · de · Whisper small"),
        "an unknown code shows itself"
    );
    assert_eq!(other["tick"], Value::Null);
    assert_eq!(p["sheet"]["empty"], Value::Null);
}

/// Today at 09:05, as the clipboard panel words it in English.
fn rig_when() -> String {
    Rig::new().hub.i18n().clip_when(&today())
}

#[test]
fn the_footer_copies_pins_deletes_and_inserts_in_tick_order() {
    let p = page(sheet(vec![bonjour()], 12, true, false), 2);
    let foot = [
        button("button", "copy", "Copy", true, "plain"),
        button("button", "pin", "Unpin", true, "plain"),
        button("button", "delete", "Delete", true, "danger"),
        button("button", "insert", "Insert (2)", true, "primary"),
    ];
    assert_eq!(buttons(&p["foot"]), foot);
    assert_eq!(p["extra"]["id"], "delete-all");
    assert_eq!(
        (&p["extra"]["label"], &p["extra"]["on"]),
        (&json!("Delete all"), &json!(true))
    );
    let one = page(sheet(vec![bonjour()], 12, false, false), 1);
    let labels: Vec<String> = buttons(&one["foot"])
        .into_iter()
        .map(|b| b[2].clone())
        .collect();
    assert_eq!(labels, ["Copy", "Pin", "Delete", "Insert"]);
}

#[test]
fn delete_all_asks_first_naming_the_unpinned_texts() {
    let p = page(sheet(vec![bonjour()], 12, false, true), 0);
    assert_eq!(p["line"], "Delete 11 texts? Pinned ones stay.");
    let yes = json!({"kind": "confirm", "id": "", "arg": 1, "label": "Yes", "on": true, "tone": "danger"});
    assert_eq!(p["foot"][0], yes);
    assert_eq!(
        (&p["foot"][1]["arg"], &p["foot"][1]["label"]),
        (&json!(0), &json!("No"))
    );
    assert_eq!(p["extra"]["on"], false, "dimmed while it asks");
}

#[test]
fn an_empty_history_says_so() {
    let p = page(sheet(Vec::new(), 0, false, false), 0);
    assert_eq!(p["sheet"]["empty"], super::english_text(VOICE_EMPTY));
    assert_eq!(p["sub"], "0 texts · newest first");
}
