//! The keyboard's texts follow the language of the layout in front, or a set `[ui] language`; the tray's follow the
//! system (ADR-0020).

use crate::{FakeFace, Rig, failed};
use kx_module_api::ports::{FaceAction, FaceReply, KeyboardFace};
use kx_platform::ui::TrayImage;
use kx_test_support::host::FakeHost;
use kx_ui::i18n::keys::{SIZE, SIZE_NAME, SIZES, WINDOW_KEYBOARD};
use kx_ui::{AUTO, Hub, I18n, MANIFEST, UiSettings, Wake, find};
use serde_json::{Value, json};
use std::sync::Arc;

/// The Esc key's word, which French prints as Échap.
const ESC: &str = "label.esc";
/// A dimmed key's tip.
const TIP: &str = "tip.later";

/// A redraw of the state, as a relabel that saw the app in front switch answers.
const STATE: FaceReply = FaceReply {
    board: false,
    state: true,
    notice: None,
    repeated: false,
};

/// The text of `key` in shipped language `tag`.
fn word(tag: &str, key: &str) -> String {
    I18n::new(find(tag).expect("a shipped language")).text(key)
}

/// A rig on a system in `os`, whose face shows a layout in `layout`, attached with `[ui] language` `setting`.
fn rig(os: &str, layout: &str, setting: &str) -> (Rig, Arc<FakeFace>, FakeHost) {
    let rig = Rig::new();
    rig.look.set_language(os);
    let face = Arc::new(FakeFace::default());
    face.set_lang(layout);
    face.answer(Ok(STATE));
    let mut host = FakeHost::new(&MANIFEST);
    let mut settings = UiSettings::from_defaults().unwrap();
    setting.clone_into(&mut settings.language);
    let shown: Arc<dyn KeyboardFace> = face.clone();
    rig.hub.attach(settings, shown, host.handle());
    rig.wakes();
    (rig, face, host)
}

/// The app in front switched to a layout in `iso`, and the relabel timer ran.
fn switch(rig: &Rig, face: &FakeFace, iso: &str) {
    face.set_lang(iso);
    let _ = rig.hub.act(FaceAction::Relabel);
}

/// The keyboard's language as QML reads it.
fn lang(hub: &Hub) -> Value {
    serde_json::from_str(&hub.lang_json()).unwrap()
}

#[test]
fn the_keyboard_speaks_the_language_of_the_layout_in_front() {
    assert_ne!(
        word("fr", ESC),
        word("en", ESC),
        "the case this test is for"
    );
    let (rig, face, _host) = rig("fr-FR", "en", AUTO);
    assert_eq!(
        rig.hub.i18n().text(ESC),
        word("en", ESC),
        "not the system's"
    );
    for iso in ["fr", "en", "fr"] {
        switch(&rig, &face, iso);
        assert_eq!(rig.wakes(), [Wake::Texts], "{iso}");
        assert_eq!(rig.hub.i18n().text(ESC), word(iso, ESC), "{iso}");
        assert_eq!(rig.hub.i18n().text(TIP), word(iso, TIP), "{iso}");
    }
}

#[test]
fn an_arabic_layout_takes_the_english_key_words_and_arabic_tips_right_to_left() {
    let (rig, face, _host) = rig("fr-FR", "en", AUTO);
    switch(&rig, &face, "ar");
    let t = rig.hub.i18n();
    assert_eq!(t.text(ESC), word("en", ESC), "as Arabic keyboards print");
    assert_eq!(t.text(TIP), word("ar", TIP));
    assert_ne!(t.text(TIP), word("en", TIP));
    let lang = lang(&rig.hub);
    assert_eq!((&lang["tag"], &lang["rtl"]), (&json!("ar"), &json!(true)));
}

#[test]
fn a_layout_language_without_texts_gives_english() {
    let (rig, face, _host) = rig("fr-FR", "fr", AUTO);
    switch(&rig, &face, "de");
    assert_eq!(rig.hub.i18n().lang().tag, "en");
    assert_eq!(rig.wakes(), [Wake::Texts]);
}

#[test]
fn a_set_language_pins_the_keyboards_texts() {
    let (rig, face, _host) = rig("ar-TN", "en", "fr");
    assert_eq!(rig.hub.i18n().lang().tag, "fr");
    switch(&rig, &face, "ar");
    assert_eq!(rig.hub.i18n().lang().tag, "fr");
    assert!(rig.wakes().is_empty(), "nothing to read again");
}

#[test]
fn only_a_new_language_in_a_new_state_wakes_the_ui() {
    let (rig, face, _host) = rig("fr-FR", "en", AUTO);
    switch(&rig, &face, "en");
    assert!(rig.wakes().is_empty(), "the same language");
    face.answer(Ok(FaceReply::default()));
    switch(&rig, &face, "fr");
    assert!(rig.wakes().is_empty(), "no new state, so no new layout");
    assert_eq!(rig.hub.i18n().lang().tag, "en");
}

#[test]
fn the_tray_speaks_the_systems_language_whatever_the_keyboard_speaks() {
    let (rig, face, _host) = rig("ar-TN", "en", "fr");
    switch(&rig, &face, "fr");
    assert_eq!(rig.hub.tray_i18n().lang().tag, "ar");
    assert!(
        rig.hub
            .tray_show(&TrayImage::new(1, 1, vec![0; 4]).unwrap())
    );
    assert_eq!(rig.tray.shown()[0].1, word("ar", WINDOW_KEYBOARD));
    let tray = &lang(&rig.hub)["tray"];
    assert_eq!((&tray["tag"], &tray["rtl"]), (&json!("ar"), &json!(true)));
}

#[test]
fn until_a_layout_names_its_language_the_keyboard_speaks_english() {
    let rig = Rig::new();
    rig.look.set_language("ar-TN");
    let (hub, _requests) = Hub::new(rig.ports()).unwrap();
    assert_eq!(hub.i18n().lang().tag, "en", "no keyboard yet");
    let mut host = FakeHost::new(&MANIFEST);
    hub.attach(
        UiSettings::from_defaults().unwrap(),
        Arc::new(FakeFace::default()),
        host.handle(),
    );
    assert_eq!(hub.i18n().lang().tag, "en", "a layout with no language");
    assert_eq!(
        hub.tray_i18n().lang().tag,
        "ar",
        "the tray keeps the system's"
    );
}

#[test]
fn the_banner_tells_its_notices_again_in_the_new_language() {
    let (rig, face, _host) = rig("en-US", "en", AUTO);
    rig.hub.notice(failed());
    let first = rig.hub.take_notices();
    switch(&rig, &face, "fr");
    let again = rig.hub.shown_notices();
    assert_eq!(again.len(), 1);
    assert_eq!(again[0].id, first[0].id);
    assert_eq!(
        again[0].text,
        I18n::new(find("fr").unwrap()).notice(&failed())
    );
    assert_ne!(again[0].text, first[0].text);
}

#[test]
fn qml_reads_the_size_letters_in_the_keyboards_language() {
    let (rig, _face, _host) = rig("en-US", "ar", AUTO);
    let lang = lang(&rig.hub);
    let ar = rig.hub.i18n();
    let sizes = lang["sizes"].as_array().expect("a list of sizes");
    assert_eq!(sizes.len(), SIZES.len());
    for (s, id) in sizes.iter().zip(SIZES) {
        assert_eq!(s["label"].as_str(), Some(ar.name_of(SIZE, id).as_str()));
        assert_eq!(s["name"].as_str(), Some(ar.name_of(SIZE_NAME, id).as_str()));
        assert_ne!(s["label"].as_str(), Some(id), "{id} has an Arabic letter");
        assert_eq!(
            s["id"].as_str(),
            Some(id),
            "UI Automation's id, in every language"
        );
    }
}
