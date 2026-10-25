//! The lane as `words_json` gives it for each kind: chips with their kind, direction and the name readers hear, the
//! lock, off and empty lanes, the menu's switches and the email list's page.

use super::{chip, heard, lane, speaking, words};
use crate::Rig;
use kx_module_api::Redacted;
use kx_module_api::ports::{ChipKind, Lane, ListPage};
use kx_ui::UiSettings;
use serde_json::{Value, json};

/// Every chip kind, an Arabic word among them, as the module may send them.
fn every_kind() -> Lane {
    Lane::Chips(vec![
        chip("hello", ChipKind::Word, false),
        chip("مرحبا", ChipKind::Word, true),
        chip("example.com", ChipKind::Domain, false),
        chip("sam@example.com", ChipKind::Email, false),
        chip("", ChipKind::More, false),
        chip("", ChipKind::Undo, false),
    ])
}

#[test]
fn chips_show_their_text_kind_and_direction_with_the_name_readers_hear() {
    let rig = Rig::new();
    let (bus, _sent, _subs) = heard(&rig);
    bus.publish(&lane(6, every_kind())).unwrap();
    let shown = words(&rig);
    assert_eq!(
        (&shown["round"], &shown["kind"]),
        (&6.into(), &"chips".into())
    );
    let want = json!([
        { "text": "hello", "kind": "word", "rtl": false, "name": "Suggestion: hello" },
        { "text": "مرحبا", "kind": "word", "rtl": true, "name": "Suggestion: مرحبا" },
        { "text": "example.com", "kind": "domain", "rtl": false, "name": "Suggestion: example.com" },
        { "text": "sam@example.com", "kind": "email", "rtl": false, "name": "Suggestion: sam@example.com" },
        { "text": "", "kind": "more", "rtl": false, "name": "More emails" },
        { "text": "", "kind": "undo", "rtl": false, "name": "Undo" },
    ]);
    assert_eq!(shown["chips"], want);
}

#[test]
fn chip_names_speak_the_keyboards_language() {
    let rows = UiSettings::from_defaults().unwrap().page_rows;
    let (rig, _host) = speaking("ar", rows);
    let (bus, _sent, _subs) = heard(&rig);
    bus.publish(&lane(1, every_kind())).unwrap();
    let names: Vec<Value> = words(&rig)["chips"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].clone())
        .collect();
    let want = [
        "اقتراح: hello",
        "اقتراح: مرحبا",
        "اقتراح: example.com",
        "اقتراح: sam@example.com",
        "عناوين بريد أخرى",
        "تراجع",
    ];
    assert_eq!(names, want.map(Value::from));
}

#[test]
fn the_lock_off_and_empty_lanes_show_no_chips() {
    let rig = Rig::new();
    let (bus, _sent, _subs) = heard(&rig);
    for (shown, kind) in [
        (Lane::Locked, "locked"),
        (Lane::Off, "off"),
        (Lane::Empty, "empty"),
    ] {
        bus.publish(&lane(2, shown)).unwrap();
        let v = words(&rig);
        assert_eq!(v["kind"], kind);
        assert_eq!(v["chips"], json!([]), "{kind}");
    }
}

#[test]
fn the_menus_switches_and_the_email_list_pass_through() {
    let rig = Rig::new();
    let (bus, _sent, _subs) = heard(&rig);
    let mut shown = lane(8, Lane::Chips(Vec::new()));
    shown.forgetting = true;
    shown.learn = false;
    shown.on = false;
    let emails = ["a@example.com", "b@example.org"];
    shown.list = Some(ListPage {
        items: emails
            .iter()
            .map(|e| Redacted::new((*e).to_owned()))
            .collect(),
        page: 1,
        pages: 3,
    });
    bus.publish(&shown).unwrap();
    let v = words(&rig);
    let flags = [&v["forgetting"], &v["learn"], &v["on"]];
    assert_eq!(flags, [&json!(true), &json!(false), &json!(false)]);
    assert_eq!(v["list"], json!({ "items": emails, "page": 1, "pages": 3 }));
}

#[test]
fn hidden_marks_never_reach_a_chip_or_a_row() {
    let rig = Rig::new();
    let (bus, _sent, _subs) = heard(&rig);
    let marked = "\u{202e}ab\u{200f}c\u{2066}";
    let mut shown = lane(3, Lane::Chips(vec![chip(marked, ChipKind::Word, false)]));
    shown.list = Some(ListPage {
        items: vec![Redacted::new(marked.to_owned())],
        page: 0,
        pages: 1,
    });
    bus.publish(&shown).unwrap();
    let v = words(&rig);
    assert_eq!(v["chips"][0]["text"], "abc");
    assert_eq!(v["chips"][0]["name"], "Suggestion: abc");
    assert_eq!(v["list"]["items"], json!(["abc"]));
}
