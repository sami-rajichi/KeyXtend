//! The lane as `words_json` gives it for each kind: chips with their kind, Arabic script and the name readers hear, the
//! lock, off, empty and stopped lanes, the menu's switches and the email list's page.

use super::{chip, heard, lane, speaking, words};
use crate::Rig;
use kx_module_api::Redacted;
use kx_module_api::ports::{ChipKind, Lane, ListPage, Suggested};
use kx_ui::UiSettings;
use serde_json::{Value, json};

/// Every chip kind, an Arabic word among them, as the module may send them.
fn every_kind() -> Lane {
    Lane::Chips(vec![
        chip("hello", ChipKind::Word),
        chip("مرحبا", ChipKind::Word),
        chip("example.com", ChipKind::Domain),
        chip("sam@example.com", ChipKind::Email),
        chip("", ChipKind::More),
        chip("", ChipKind::Undo),
    ])
}

#[test]
fn chips_show_their_text_kind_and_script_with_the_name_readers_hear() {
    let rig = Rig::new();
    let (bus, _sent, _subs) = heard(&rig);
    bus.publish(&lane(6, every_kind())).unwrap();
    let shown = words(&rig);
    assert_eq!(
        (&shown["round"], &shown["kind"]),
        (&6.into(), &"chips".into())
    );
    let want = json!([
        { "text": "hello", "kind": "word", "arabic": false, "name": "Suggestion: hello" },
        { "text": "مرحبا", "kind": "word", "arabic": true, "name": "Suggestion: مرحبا" },
        { "text": "example.com", "kind": "domain", "arabic": false, "name": "Suggestion: example.com" },
        { "text": "sam@example.com", "kind": "email", "arabic": false, "name": "Suggestion: sam@example.com" },
        { "text": "", "kind": "more", "arabic": false, "name": "More emails" },
        { "text": "", "kind": "undo", "arabic": false, "name": "Undo" },
    ]);
    assert_eq!(shown["chips"], want);
}

#[test]
fn only_arabic_script_text_is_drawn_as_arabic_whatever_its_direction() {
    let rig = Rig::new();
    let (bus, _sent, _subs) = heard(&rig);
    let texts = [("שלום", false), ("ܫܠܡܐ", false), ("سلام", true)];
    let chips = texts.map(|(t, _)| chip(t, ChipKind::Word)).to_vec();
    let mut shown = lane(4, Lane::Chips(chips));
    shown.list = Some(ListPage {
        items: ["سارة@example.com", "sara@example.com"]
            .map(|e| Redacted::new(e.to_owned()))
            .to_vec(),
        page: 0,
        pages: 1,
    });
    bus.publish(&shown).unwrap();
    let v = words(&rig);
    for (i, (text, arabic)) in texts.iter().enumerate() {
        assert_eq!(v["chips"][i]["arabic"], *arabic, "{text}");
    }
    let rows = [
        &v["list"]["items"][0]["arabic"],
        &v["list"]["items"][1]["arabic"],
    ];
    assert_eq!(rows, [&json!(true), &json!(false)]);
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
fn in_forget_mode_each_chip_is_named_by_what_a_click_does() {
    let rows = UiSettings::from_defaults().unwrap().page_rows;
    let want = [
        ("en", "Forget: hello"),
        ("fr", "Oublier\u{a0}: hello"),
        ("ar", "نسيان: hello"),
    ];
    for (iso, name) in want {
        let (rig, _host) = speaking(iso, rows);
        let (bus, _sent, _subs) = heard(&rig);
        let words_only = Lane::Chips(vec![chip("hello", ChipKind::Word)]);
        let forgetting = Suggested {
            forgetting: true,
            ..lane(2, words_only)
        };
        bus.publish(&forgetting).unwrap();
        assert_eq!(words(&rig)["chips"][0]["name"], name, "{iso}");
    }
}

#[test]
fn the_lock_off_empty_and_stopped_lanes_show_no_chips() {
    let rig = Rig::new();
    let (bus, _sent, _subs) = heard(&rig);
    for (shown, kind) in [
        (Lane::Locked, "locked"),
        (Lane::Off, "off"),
        (Lane::Empty, "empty"),
        (Lane::Stopped, "stopped"),
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
    let items: Vec<Value> = emails
        .iter()
        .map(|e| json!({ "text": e, "arabic": false }))
        .collect();
    assert_eq!(v["list"], json!({ "items": items, "page": 1, "pages": 3 }));
}

#[test]
fn hidden_marks_never_reach_a_chip_or_a_row() {
    let rig = Rig::new();
    let (bus, _sent, _subs) = heard(&rig);
    let marked = "\u{202e}ab\u{200f}c\u{2066}";
    let mut shown = lane(3, Lane::Chips(vec![chip(marked, ChipKind::Word)]));
    shown.list = Some(ListPage {
        items: vec![Redacted::new(marked.to_owned())],
        page: 0,
        pages: 1,
    });
    bus.publish(&shown).unwrap();
    let v = words(&rig);
    assert_eq!(v["chips"][0]["text"], "abc");
    assert_eq!(v["chips"][0]["name"], "Suggestion: abc");
    assert_eq!(v["list"]["items"][0]["text"], "abc");
}
