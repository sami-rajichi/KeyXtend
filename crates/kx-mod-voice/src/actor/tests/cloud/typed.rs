//! The key page's lines through the actor (ADR-0049): the keyboard hears which line the panel shows after each view,
//! once per change, again when it asks and closed at the stop; a key typed on the key line is sealed and shown only
//! as dots; and another panel opening closes this one.

use kx_module_api::ports::{
    CloudButton, CloudLine, PanelKey, PanelLine, ServiceSheet, VoiceClick, VoiceSheet, VoiceTab,
};

use super::super::taps::Heard;
use super::*;
use crate::actor::Msg;
use crate::services::tests::shipped::{entry, first};

/// A key-like text, built as the test runs.
fn made_key() -> String {
    ["kx", "typed", &"3".repeat(20)].join("-")
}

/// Every line told, in order.
fn told(rig: &Rig) -> Vec<Option<PanelLine>> {
    let lines = rig.heard().into_iter().filter_map(|h| match h {
        Heard::Line(line) => Some(line),
        _ => None,
    });
    lines.collect()
}

/// Groq on with its terms confirmed and no key, with the panel on its key page.
fn on_lines() -> Rig {
    let mut rig = Rig::build(|_, host| {
        host.put("cloud", true.into());
        host.put("services", table(&entry(first(), "")));
    });
    rig.send(Msg::Open);
    rig.act(VoiceClick::Open(VoiceTab::Cloud));
    rig.act(VoiceClick::Row(0));
    rig.act(VoiceClick::Service(CloudButton::Lines));
    rig
}

/// The key page as last drawn.
fn lines(rig: &Rig) -> ServiceSheet {
    let panel = rig.views().last().and_then(|v| v.panel.clone());
    match panel.map(|p| p.sheet) {
        Some(VoiceSheet::Service(sheet)) => sheet,
        other => panic!("not a key page: {other:?}"),
    }
}

#[test]
fn the_keyboard_hears_the_panels_line_after_each_view_once_per_change() {
    let mut rig = on_lines();
    let open = Some(PanelLine::None);
    assert_eq!(told(&rig), [open.clone(), Some(PanelLine::Private)]);
    let heard = rig.heard();
    let last_view = heard.iter().rposition(|h| matches!(h, Heard::View(_)));
    let last_line = heard.iter().rposition(|h| matches!(h, Heard::Line(_)));
    assert!(last_view < last_line, "the line follows its view");
    rig.send(Msg::LineKey(PanelKey::Text(Redacted::new("a".into()))));
    assert_eq!(told(&rig).len(), 2, "a key changes no line");
    rig.send(Msg::Ask);
    assert_eq!(
        told(&rig).last(),
        Some(&Some(PanelLine::Private)),
        "told again when asked"
    );
    rig.act(VoiceClick::Button(kx_module_api::ports::PanelButton::Back));
    assert_eq!(told(&rig).last(), Some(&open));
    rig.send(Msg::Quit);
    assert_eq!(told(&rig).last(), Some(&None), "the stop closes it");
}

#[test]
fn a_key_typed_on_its_line_is_sealed_and_shown_only_as_dots() {
    let mut rig = on_lines();
    let key = made_key();
    for c in key.chars() {
        rig.send(Msg::LineKey(PanelKey::Text(Redacted::new(c.into()))));
    }
    let n = u32::try_from(key.len()).unwrap();
    assert_eq!(*lines(&rig).typed.expose(), n);
    rig.send(Msg::LineKey(PanelKey::Enter));
    let keys = Keys::new(Arc::new(rig.fake.shelf.clone()), false);
    let sealed = keys.get("groq").unwrap().map(|k| k.expose().to_string());
    assert_eq!(sealed.as_deref(), Some(key.as_str()));
    let sheet = lines(&rig);
    assert_eq!((*sheet.key.expose(), *sheet.typed.expose()), (n, 0));
    assert!(!sheet.saves.contains(&CloudLine::Key));
    let heard = format!("{:?}", rig.heard());
    assert!(!heard.contains(&key), "no event printed it");
}

#[test]
fn another_panel_opening_closes_this_one() {
    let mut rig = on_lines();
    rig.send(Msg::LineKey(PanelKey::Text(Redacted::new("a".into()))));
    rig.send(Msg::Close);
    assert!(rig.views().last().unwrap().panel.is_none());
    assert_eq!(told(&rig).last(), Some(&None));
    let before = rig.heard().len();
    rig.send(Msg::Close);
    assert_eq!(rig.heard().len(), before, "a late close changes nothing");
}
