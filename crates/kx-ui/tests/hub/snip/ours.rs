//! Our other windows beside a capture (edge 16): while the keyboard is aside as its bubble or hidden, every window of ours
//! but Snip's is kept out of every capture, one Qt shows later as it shows, and Back lets them all in again; Shown
//! shields none, a refused shield hides ours, and the tray's Snip row shows no keyboard that a capture hides.

use super::aside::{mode, ours, settle, shielded};
use super::{Heard, heard};
use crate::Rig;
use kx_module_api::ports::{SnipAside, SnipClick};
use kx_platform::ui::UiError;
use kx_platform_fake::GuardCall;
use kx_ui::i18n::keys::{WINDOW_CLIP, WINDOW_KEYBOARD};
use kx_ui::{Mode, Wake};
use serde_json::json;

/// The clipboard panel's title, a window of ours that shows on demand.
fn clip(rig: &Rig) -> String {
    rig.hub.i18n().text(WINDOW_CLIP)
}

/// The guard's calls since `from`.
fn since(rig: &Rig, from: usize) -> Vec<GuardCall> {
    rig.guard.calls().split_off(from)
}

/// The keyboard steps aside as `aside`, which the Qt thread settles.
pub fn aside(rig: &Rig, heard: &Heard, aside: SnipAside) {
    heard.aside(aside);
    settle(rig);
}

#[test]
fn every_window_of_ours_is_kept_out_in_bubble_and_hidden_and_let_back_in_at_back() {
    for way in [SnipAside::Bubble, SnipAside::Hidden] {
        let rig = Rig::new();
        let heard = heard(&rig);
        aside(&rig, &heard, way);
        assert_eq!(shielded(&rig), ours(&rig), "{way:?}");
        assert!(ours(&rig).contains(&clip(&rig)), "the panels too");
        aside(&rig, &heard, SnipAside::Back);
        assert!(shielded(&rig).is_empty(), "{way:?}: in captures again");
    }
}

#[test]
fn shown_shields_none_of_ours() {
    let rig = Rig::new();
    let _heard = heard(&rig);
    settle(&rig);
    rig.hub.aside_window(&clip(&rig));
    assert!(shielded(&rig).is_empty());
}

#[test]
fn a_window_not_made_yet_waits_for_its_show_but_the_keyboard_must_be_there() {
    let rig = Rig::new();
    let heard = heard(&rig);
    let (clip, keyboard) = (clip(&rig), rig.hub.i18n().text(WINDOW_KEYBOARD));
    rig.guard.set_absent(&[clip.as_str()]);
    aside(&rig, &heard, SnipAside::Bubble);
    assert_eq!(mode(&rig)["aside"], json!("bubble"), "no panel open yet");
    rig.guard.set_absent(&[keyboard.as_str()]);
    aside(&rig, &heard, SnipAside::Bubble);
    assert_eq!(mode(&rig)["aside"], json!("hidden"), "a keyboard not found");
}

#[test]
fn a_window_qt_shows_during_the_aside_is_kept_out_as_it_shows() {
    let rig = Rig::new();
    let heard = heard(&rig);
    let clip = clip(&rig);
    aside(&rig, &heard, SnipAside::Bubble);
    let from = rig.guard.calls().len();
    rig.hub.aside_window(&clip);
    assert_eq!(since(&rig, from), [GuardCall::Shield(clip.clone(), true)]);
    for other in ["Snip strip", "Notepad", ""] {
        rig.hub.aside_window(other);
    }
    assert_eq!(
        since(&rig, from).len(),
        1,
        "Snip's and others' are left alone"
    );
    aside(&rig, &heard, SnipAside::Back);
    let from = rig.guard.calls().len();
    rig.hub.aside_window(&clip);
    assert_eq!(since(&rig, from), [], "the keyboard is back");
}

#[test]
fn a_refused_shield_on_a_window_shown_later_hides_ours() {
    let rig = Rig::new();
    let heard = heard(&rig);
    aside(&rig, &heard, SnipAside::Bubble);
    rig.wakes();
    rig.guard.set_shield_error(Some(UiError::Refused(5)));
    rig.hub.aside_window(&clip(&rig));
    assert_eq!(rig.wakes(), [Wake::Aside], "QML hides ours");
    assert_eq!(mode(&rig)["aside"], json!("hidden"), "never in a video");
    settle(&rig);
    assert_eq!(rig.wakes(), [], "its own wake settles it once");
}

#[test]
fn the_tray_shows_no_keyboard_that_a_capture_hides() {
    for (way, wakes) in [
        (SnipAside::Hidden, vec![]),
        (SnipAside::Bubble, vec![Wake::Show]),
    ] {
        let rig = Rig::new();
        let heard = heard(&rig);
        rig.hub.keep_mode(Mode::Away);
        aside(&rig, &heard, way);
        rig.wakes();
        rig.hub.snip_open();
        assert_eq!(rig.wakes(), wakes, "{way:?}");
        assert_eq!(heard.sent.clicks(), [SnipClick::Open], "{way:?}");
    }
}
