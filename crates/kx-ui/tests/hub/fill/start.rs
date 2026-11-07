//! What the vault hears from the ui as it starts: the rows a page holds, at start, as the panel opens and as the
//! settings change, and the label names in the keyboard's language, which a new language sends again.

use super::{fill, heard, listed};
use crate::{FakeFace, Rig};
use kx_module_api::ports::{FaceAction, FaceReply, FillAct, FillClick};
use kx_test_support::host::FakeHost;
use kx_ui::{MANIFEST, UiSettings, Wake};
use std::sync::Arc;

/// The `[ui]` defaults with `rows` rows a page.
fn rows(rows: usize) -> UiSettings {
    let mut settings = UiSettings::from_defaults().unwrap();
    settings.page_rows = rows;
    settings
}

/// Attaches a face showing a layout in `iso` with `settings`, whose relabels say the state changed.
fn attach(rig: &Rig, iso: &str, settings: UiSettings) -> (Arc<FakeFace>, FakeHost) {
    let face = Arc::new(FakeFace::default());
    face.set_lang(iso);
    face.answer(Ok(FaceReply {
        state: true,
        ..FaceReply::default()
    }));
    let mut host = FakeHost::new(&MANIFEST);
    rig.hub.attach(settings, face.clone(), host.handle());
    rig.wakes();
    (face, host)
}

/// The app in front switched to a layout in `iso`, and the relabel timer ran.
fn switch(rig: &Rig, face: &FakeFace, iso: &str) {
    face.set_lang(iso);
    let _ = rig.hub.act(FaceAction::Relabel);
}

/// The Rows click each act names.
fn rows_of(acts: &[FillAct]) -> Vec<(u32, u8)> {
    let rows = |a: &FillAct| match a.click {
        FillClick::Rows(n) => Some((a.round, n)),
        _ => None,
    };
    acts.iter().filter_map(rows).collect()
}
#[test]
fn the_rows_go_at_start_as_the_panel_opens_and_as_the_settings_change() {
    let rig = Rig::new();
    let heard = heard(&rig);
    let _kept = attach(&rig, "en", rows(4));
    assert_eq!(rows_of(&heard.sent.acts()), [(0, 4)], "at start");
    heard.panel(true);
    assert_eq!(rows_of(&heard.sent.acts()), [(0, 4)], "as it opens");
    heard.view(listed(7));
    let _kept = attach(&rig, "en", rows(6));
    assert_eq!(rows_of(&heard.sent.acts()), [(7, 6)], "the shown round");
    heard.panel(false);
    let _kept = attach(&rig, "en", rows(3));
    assert_eq!(
        rows_of(&heard.sent.acts()),
        [(0, 3)],
        "the vault keeps them"
    );
}

#[test]
fn rows_past_what_a_byte_holds_are_clamped() {
    let rig = Rig::new();
    let heard = heard(&rig);
    let _kept = attach(&rig, "en", rows(300));
    assert_eq!(rows_of(&heard.sent.acts()), [(0, u8::MAX)]);
}

#[test]
fn the_label_names_go_at_start_as_the_panel_opens_and_on_a_language_change() {
    let rig = Rig::new();
    let heard = heard(&rig);
    let (face, _host) = attach(&rig, "fr", rows(5));
    let names = heard.sent.names();
    assert_eq!(names.len(), 1, "at start");
    assert_eq!(names[0].0["email"], "E-mail");
    assert_eq!(names[0].0["pin"], "Code PIN");
    heard.panel(true);
    assert_eq!(heard.sent.names(), names, "as it opens");
    switch(&rig, &face, "ar");
    let arabic = heard.sent.names();
    assert_eq!(arabic.len(), 1, "on a language change");
    assert_eq!(arabic[0].0["email"], "البريد الإلكتروني");
    assert_eq!(
        arabic[0].0.keys().collect::<Vec<_>>(),
        names[0].0.keys().collect::<Vec<_>>()
    );
    switch(&rig, &face, "ar");
    assert!(
        heard.sent.names().is_empty(),
        "the same language sends none"
    );
}

#[test]
fn a_language_change_wakes_a_kept_view_to_be_worded_again() {
    let rig = Rig::new();
    let heard = heard(&rig);
    let (face, _host) = attach(&rig, "en", rows(5));
    switch(&rig, &face, "fr");
    assert_eq!(heard.sent.woken(), 0, "no view to word");
    switch(&rig, &face, "en");
    heard.shown(&rig, listed(2));
    assert_eq!(fill(&rig)["title"], "Quick-fill");
    switch(&rig, &face, "fr");
    assert_eq!(heard.sent.woken(), 1);
    assert_eq!(fill(&rig)["title"], "Remplissage rapide");
    assert!(
        rig.wakes().contains(&Wake::Texts),
        "the Ui object rewords too"
    );
}
