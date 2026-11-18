//! Typing on the key page's lines (ADR-0049, edge 22): letters, Space, Backspace, Delete and the arrows work the line at
//! its caret, a click places it, Enter and Save set it through the page's own acts, an empty line saves nothing, and the
//! key typed is wiped on Save, Paste, Remove, a page turn, leaving the page and the panel closing.

use kx_module_api::Redacted;
use kx_module_api::ports::{
    AddressFault, CloudButton, CloudLine, HttpsUrl, PanelButton, PanelKey, PanelLine, ServiceSheet,
    TAG_LEN, VoiceClick, VoiceTab,
};

use super::super::super::rig::Rig;
use super::super::super::{Do, Page};
use super::lines::on_groq_lines;
use super::{act, apply, lines_sheet, on_cloud, put};
use crate::settings::SERVICES;

/// A key-like text, built as the test runs so no key sits in the source.
fn made_key() -> String {
    ["kx", "typed", &"5".repeat(12)].join("-")
}

/// A key from our keyboard, on the view the ui would have drawn now.
fn key(rig: &mut Rig, key: &PanelKey) -> Vec<Do> {
    rig.view();
    rig.with(|p, f| p.line_key(key, f))
}

/// Types `text` letter by letter.
fn type_in(rig: &mut Rig, text: &str) {
    for c in text.chars() {
        assert!(key(rig, &PanelKey::Text(Redacted::new(c.into()))).is_empty());
    }
}

/// A click on `line` with its caret `at`.
fn caret(rig: &mut Rig, line: CloudLine, at: u32) {
    let at = Redacted::new(at);
    rig.click(&VoiceClick::Caret { line, at });
}

/// The key typed, the line taking the letters and its caret, as the page shows them.
fn typing(sheet: &ServiceSheet) -> (u32, CloudLine, u32) {
    (*sheet.typed.expose(), sheet.typing, *sheet.caret.expose())
}

/// Custom on with `extra` in its entry, on its key page's second part: the address and model lines.
fn on_custom_model(extra: &str) -> Rig {
    let mut rig = on_cloud("");
    let entry = format!("custom = {{ on = 2026-11-01, terms = 1{extra} }}");
    put(&mut rig, "services", &entry);
    act(&mut rig, 1, CloudButton::Lines);
    rig.click(&VoiceClick::Page { back: false });
    rig
}

#[test]
fn letters_backspace_delete_and_the_arrows_work_the_key_line_shown_only_as_dots() {
    let mut rig = on_groq_lines();
    let made = made_key();
    type_in(&mut rig, &made);
    let n = u32::try_from(made.len()).unwrap();
    let sheet = lines_sheet(&mut rig);
    assert_eq!(typing(&sheet), (n, CloudLine::Key, n));
    assert!(!format!("{sheet:?}").contains(&made), "the key never shows");
    key(&mut rig, &PanelKey::Left);
    key(&mut rig, &PanelKey::Left);
    key(&mut rig, &PanelKey::Erase);
    assert_eq!(
        typing(&lines_sheet(&mut rig)),
        (n - 1, CloudLine::Key, n - 3)
    );
    key(&mut rig, &PanelKey::Delete);
    key(&mut rig, &PanelKey::Tick);
    key(&mut rig, &PanelKey::Right);
    assert_eq!(
        typing(&lines_sheet(&mut rig)),
        (n - 1, CloudLine::Key, n - 1)
    );
    for _ in 0..n {
        key(&mut rig, &PanelKey::Erase);
    }
    assert_eq!(typing(&lines_sheet(&mut rig)), (0, CloudLine::Key, 0));
}

#[test]
fn enter_and_save_set_the_typed_key_which_is_wiped_and_an_empty_line_saves_nothing() {
    let mut rig = on_groq_lines();
    assert!(key(&mut rig, &PanelKey::Enter).is_empty(), "nothing typed");
    assert!(rig.click(&VoiceClick::Save(CloudLine::Key)).is_empty());
    assert!(!lines_sheet(&mut rig).saves.contains(&CloudLine::Key));
    let made = made_key();
    type_in(&mut rig, &format!(" {made}"));
    assert!(lines_sheet(&mut rig).saves.contains(&CloudLine::Key));
    let out = key(&mut rig, &PanelKey::Enter);
    let [Do::SetKey { service, key: set }] = &out[..] else {
        panic!("no key set: {out:?}");
    };
    assert_eq!(
        (service.as_str(), set.expose().as_str()),
        ("groq", made.as_str())
    );
    assert!(!format!("{out:?}").contains(&made), "the key never prints");
    assert_eq!(*lines_sheet(&mut rig).typed.expose(), 0, "wiped on Save");
    type_in(&mut rig, &made);
    let out = rig.click(&VoiceClick::Save(CloudLine::Key));
    assert!(matches!(&out[..], [Do::SetKey { .. }]), "{out:?}");
    assert!(
        key(&mut rig, &PanelKey::Enter).is_empty(),
        "nothing is left"
    );
}

#[test]
fn the_typed_key_is_wiped_on_paste_remove_leaving_the_page_and_closing() {
    let wipes: [fn(&mut Rig); 4] = [
        |rig| drop(rig.click(&VoiceClick::Service(CloudButton::Paste))),
        |rig| drop(rig.click(&VoiceClick::Service(CloudButton::Remove))),
        |rig| {
            rig.click(&VoiceClick::Button(PanelButton::Back));
            act(rig, 0, CloudButton::Lines);
        },
        |rig| {
            rig.click(&VoiceClick::Button(PanelButton::Close));
            rig.panel.open();
            rig.click(&VoiceClick::Open(VoiceTab::Cloud));
            act(rig, 0, CloudButton::Lines);
        },
    ];
    for (at, wipe) in wipes.into_iter().enumerate() {
        let mut rig = on_groq_lines();
        rig.keyed.insert("groq".into());
        type_in(&mut rig, "abc");
        wipe(&mut rig);
        assert_eq!(rig.panel.page, Some(Page::Lines), "{at}");
        assert_eq!(*lines_sheet(&mut rig).typed.expose(), 0, "{at}");
        assert!(key(&mut rig, &PanelKey::Enter).is_empty(), "{at}");
    }
}

#[test]
fn a_page_turn_starts_the_new_parts_lines_afresh() {
    let mut rig = on_custom_model("");
    rig.click(&VoiceClick::Page { back: true });
    type_in(&mut rig, "abc");
    assert_eq!(*lines_sheet(&mut rig).typed.expose(), 3);
    rig.click(&VoiceClick::Page { back: false });
    type_in(&mut rig, "x");
    rig.click(&VoiceClick::Page { back: true });
    assert_eq!(*lines_sheet(&mut rig).typed.expose(), 0, "the key went");
    rig.click(&VoiceClick::Page { back: false });
    let sheet = lines_sheet(&mut rig);
    assert_eq!(
        sheet.address.unwrap().expose().as_str(),
        "",
        "so did the address"
    );
}

#[test]
fn an_address_is_edited_from_the_kept_one_and_saved_through_its_own_checks() {
    let mut rig = on_custom_model(r#", address = "https://a.example.com/v1""#);
    let sheet = lines_sheet(&mut rig);
    assert_eq!(
        typing(&sheet),
        (0, CloudLine::Address, 24),
        "the caret after the kept text"
    );
    assert_eq!(sheet.saves, [CloudLine::Address]);
    for _ in 0..3 {
        key(&mut rig, &PanelKey::Left);
    }
    type_in(&mut rig, "/x");
    let sheet = lines_sheet(&mut rig);
    let text = sheet.address.unwrap();
    assert_eq!(text.expose().as_str(), "https://a.example.com/x/v1");
    assert_eq!(*sheet.caret.expose(), 23);
    let out = key(&mut rig, &PanelKey::Enter);
    assert!(matches!(&out[..], [Do::Set(SERVICES, _)]), "{out:?}");
    apply(&mut rig, &out);
    assert_eq!(
        rig.settings.services["custom"].address,
        "https://a.example.com/x/v1"
    );
    key(&mut rig, &PanelKey::Erase);
    type_in(&mut rig, " ");
    let out = rig.click(&VoiceClick::Save(CloudLine::Address));
    assert!(out.is_empty(), "a space is no path");
    let sheet = lines_sheet(&mut rig);
    assert_eq!(sheet.address_fault, Some(AddressFault::Path));
    let kept = sheet.address.unwrap();
    assert_eq!(
        kept.expose().as_str(),
        "https://a.example.com/x/v ",
        "it stays to put right"
    );
}

#[test]
fn an_address_or_model_past_its_limit_is_refused_with_its_own_reason() {
    let mut rig = on_custom_model("");
    let long = format!("https://a.example.com/{}", "a".repeat(HttpsUrl::MAX_BYTES));
    key(&mut rig, &PanelKey::Text(Redacted::new(long)));
    let sheet = lines_sheet(&mut rig);
    let text = sheet.address.unwrap();
    assert_eq!(
        text.expose().len(),
        HttpsUrl::MAX_BYTES + 1,
        "one past, then no more"
    );
    assert!(rig.click(&VoiceClick::Save(CloudLine::Address)).is_empty());
    assert_eq!(
        lines_sheet(&mut rig).address_fault,
        Some(AddressFault::Long)
    );
    key(&mut rig, &PanelKey::Down);
    assert_eq!(lines_sheet(&mut rig).typing, CloudLine::Model);
    let id = "m".repeat(*TAG_LEN.end() * 2);
    key(&mut rig, &PanelKey::Text(Redacted::new(id)));
    assert!(key(&mut rig, &PanelKey::Enter).is_empty());
    let sheet = lines_sheet(&mut rig);
    assert!(sheet.model_refused);
    assert_eq!(sheet.model.expose().len(), *TAG_LEN.end() + 1, "it stays");
}

#[test]
fn up_and_down_and_a_click_pick_the_line_and_a_click_places_its_caret() {
    let mut rig = on_custom_model(r#", address = "https://a.example.com/v1", model = "whisper-1""#);
    assert_eq!(lines_sheet(&mut rig).typing, CloudLine::Address);
    key(&mut rig, &PanelKey::Up);
    assert_eq!(
        lines_sheet(&mut rig).typing,
        CloudLine::Address,
        "the first stays"
    );
    key(&mut rig, &PanelKey::Down);
    key(&mut rig, &PanelKey::Down);
    assert_eq!(
        lines_sheet(&mut rig).typing,
        CloudLine::Model,
        "the last stays"
    );
    caret(&mut rig, CloudLine::Address, 2);
    let sheet = lines_sheet(&mut rig);
    assert_eq!(typing(&sheet), (0, CloudLine::Address, 2));
    caret(&mut rig, CloudLine::Key, 0);
    assert_eq!(
        lines_sheet(&mut rig).typing,
        CloudLine::Address,
        "no key line on this part"
    );
    key(&mut rig, &PanelKey::Down);
    caret(&mut rig, CloudLine::Model, 1);
    type_in(&mut rig, "x");
    let sheet = lines_sheet(&mut rig);
    assert_eq!(sheet.model.expose().as_str(), "wxhisper-1");
    assert_eq!(sheet.saves, [CloudLine::Address, CloudLine::Model]);
}

#[test]
fn the_panel_tells_a_private_line_only_on_a_key_page() {
    let mut rig = on_groq_lines();
    let line = |rig: &mut Rig| rig.with(|p, f| p.line(f));
    assert_eq!(line(&mut rig), Some(PanelLine::Private));
    rig.click(&VoiceClick::Service(CloudButton::Facts));
    assert_eq!(line(&mut rig), Some(PanelLine::None), "a card");
    rig.click(&VoiceClick::Button(PanelButton::Back));
    rig.click(&VoiceClick::Button(PanelButton::Back));
    assert_eq!(line(&mut rig), Some(PanelLine::None), "the Cloud page");
    rig.click(&VoiceClick::Button(PanelButton::Close));
    assert_eq!(line(&mut rig), None, "closed");
}

#[test]
fn keys_off_a_key_page_change_nothing() {
    let mut rig = on_cloud("");
    let before = rig.view();
    for k in [
        PanelKey::Text(Redacted::new("a".into())),
        PanelKey::Enter,
        PanelKey::Down,
    ] {
        assert!(key(&mut rig, &k).is_empty());
    }
    assert_eq!(rig.view(), before);
}
