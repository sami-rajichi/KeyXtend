//! Our shadow windows beside a capture (edge 16): they share one title, kept out with the windows they shadow in Bubble
//! and Hidden, one Qt shows later as it shows, and let back in at Back; Shown keeps none out, and a refused one hides ours.

use super::aside::{mode, settle, shielded};
use super::heard;
use super::ours::aside;
use crate::Rig;
use kx_module_api::ports::SnipAside;
use kx_platform::ui::UiError;
use kx_platform_fake::GuardCall;
use kx_ui::Wake;
use kx_ui::i18n::keys::WINDOW_SHADOW;
use serde_json::json;

/// Every shadow window's one title, in the keyboard's language.
fn shadow(rig: &Rig) -> String {
    rig.hub.i18n().text(WINDOW_SHADOW)
}

/// True once the guard was asked to keep `title` out of captures.
fn kept_out(rig: &Rig, title: &str) -> bool {
    let out = GuardCall::Shield(title.to_owned(), true);
    rig.guard.calls().contains(&out)
}

#[test]
fn every_shadow_is_kept_out_in_bubble_and_hidden_and_let_back_in_at_back() {
    for way in [SnipAside::Bubble, SnipAside::Hidden] {
        let rig = Rig::new();
        let heard = heard(&rig);
        let shadow = shadow(&rig);
        aside(&rig, &heard, way);
        assert!(shielded(&rig).contains(&shadow), "{way:?}");
        aside(&rig, &heard, SnipAside::Back);
        assert!(!shielded(&rig).contains(&shadow), "{way:?}: back in");
    }
}

#[test]
fn shown_keeps_no_shadow_out() {
    let rig = Rig::new();
    let _heard = heard(&rig);
    let shadow = shadow(&rig);
    settle(&rig);
    rig.hub.aside_window(&shadow);
    assert!(!kept_out(&rig, &shadow));
}

#[test]
fn a_shadow_not_made_yet_is_kept_out_as_qt_shows_it() {
    let rig = Rig::new();
    let heard = heard(&rig);
    let shadow = shadow(&rig);
    rig.guard.set_absent(&[shadow.as_str()]);
    aside(&rig, &heard, SnipAside::Bubble);
    assert_eq!(mode(&rig)["aside"], json!("bubble"), "none made yet");
    rig.guard.set_absent(&[]);
    rig.hub.aside_window(&shadow);
    assert!(shielded(&rig).contains(&shadow));
}

#[test]
fn a_refused_shadow_shield_at_the_aside_hides_ours() {
    let rig = Rig::new();
    let heard = heard(&rig);
    let shadow = shadow(&rig);
    rig.guard
        .set_refused(&[shadow.as_str()], UiError::Refused(5));
    aside(&rig, &heard, SnipAside::Bubble);
    assert_eq!(mode(&rig)["aside"], json!("hidden"));
}

#[test]
fn a_refused_shadow_shield_as_qt_shows_it_hides_ours() {
    let rig = Rig::new();
    let heard = heard(&rig);
    let shadow = shadow(&rig);
    aside(&rig, &heard, SnipAside::Bubble);
    rig.wakes();
    rig.guard
        .set_refused(&[shadow.as_str()], UiError::Refused(5));
    rig.hub.aside_window(&shadow);
    assert_eq!(rig.wakes(), [Wake::Aside], "as Qt shows it");
    assert_eq!(mode(&rig)["aside"], json!("hidden"));
}
