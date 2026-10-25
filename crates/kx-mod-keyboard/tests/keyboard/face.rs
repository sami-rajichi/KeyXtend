//! The face's clicks: typing, dimmed keys, failed sends, a stale table, and a settings change that keeps latches.

use crate::host::{GOLDEN, id};
use crate::rig::{Rig, STATE};
use kx_module_api::ports::{FaceAction, FaceError, FaceReply, InjectError, Scan, TextRoute};
use kx_module_api::{Redacted, keys};
use kx_platform_fake::Stroke;

const Q: &str = "c-0x0010";

fn text(t: &str) -> Stroke {
    Stroke::Text(Redacted::new(t.into()), TextRoute::Unicode)
}

#[test]
fn a_click_types_and_redraws_the_state() {
    let r = Rig::new(&GOLDEN, None);
    assert_eq!(r.tap(Q), Ok(STATE));
    assert_eq!(r.fakes.injector.strokes(), [text("q")]);
}

#[test]
fn a_key_from_a_later_version_does_nothing() {
    let r = Rig::new(&GOLDEN, None);
    assert_eq!(r.tap("mic"), Ok(FaceReply::default()));
    assert!(r.fakes.injector.strokes().is_empty());
    assert_eq!(r.state()["keys"]["mic"]["paint"]["fill"], "dim");
}

#[test]
fn an_empty_key_does_nothing_and_keeps_its_latch_lit() {
    let r = Rig::new(&["ar-tn"], None);
    r.tap("shift").unwrap();
    assert_eq!(r.state()["keys"]["c-0x001F"]["blank"], "key.empty", "س");
    assert_eq!(r.tap("c-0x001F"), Ok(FaceReply::default()));
    assert!(r.fakes.injector.strokes().is_empty());
    assert_eq!(r.state()["keys"]["shift"]["on"], true);
}

#[test]
fn with_ctrl_latched_a_key_that_types_nothing_shifted_shows_its_letter_and_sends_its_shortcut() {
    // Ctrl, then Shift, on Arabic (102) AZERTY: a click on س sends Ctrl+Shift+S, so it never reads as empty.
    let r = Rig::new(&["ar-tn"], None);
    r.tap("ctrl").unwrap();
    r.tap("shift").unwrap();
    let state = r.state();
    let seen = &state["keys"]["c-0x001F"];
    assert!(seen["blank"].is_null(), "not empty: {seen}");
    assert_eq!(
        seen["main"], "س",
        "its letter, as before an empty key had a name"
    );
    r.tap("c-0x001F").unwrap();
    let (shift, ctrl) = (Scan::new(0x2A), Scan::new(0x1D));
    let want = Stroke::Chord(vec![shift, ctrl], Scan::new(0x1F));
    assert_eq!(r.fakes.injector.strokes(), [want]);
}

#[test]
fn a_key_that_is_not_on_the_board_is_refused() {
    let r = Rig::new(&GOLDEN, None);
    assert_eq!(r.tap("nope"), Err(FaceError::UnknownKey));
    assert_eq!(r.panel("nope", 5), Err(FaceError::UnknownKey));
}

#[test]
fn a_send_that_fails_shows_a_notice_and_redraws_what_it_used() {
    let r = Rig::new(&GOLDEN, None);
    r.tap("shift").unwrap();
    let part = InjectError::Refused { sent: 1, total: 4 };
    r.fakes.injector.set_error(Some(part));
    let want = FaceReply {
        notice: Some(keys::KEYS_NOT_SENT),
        ..STATE
    };
    assert_eq!(r.tap(Q), Ok(want));
    assert_eq!(
        r.state()["keys"]["shift"]["on"],
        false,
        "the part that went used it"
    );
}

#[test]
fn a_stale_click_refreshes_once_redraws_and_is_never_replayed() {
    let r = Rig::new(&["fr-fr", "en-us"], None);
    r.fakes.source.set_foreground(Some(id("en-us")));
    r.fakes.layouts.switch(id("en-us"));
    assert_eq!(
        r.tap("c-0x001A"),
        Ok(STATE),
        "EN would type [ under the FR ^"
    );
    assert_eq!(r.fakes.layouts.refreshes(), 1);
    assert!(r.fakes.injector.strokes().is_empty(), "nothing was sent");
    assert_eq!(r.shown(), "EN", "the redraw shows the app's layout");
    assert_eq!(r.tap("c-0x001A"), Ok(STATE), "the user clicks again");
    assert_eq!(r.fakes.injector.strokes(), [text("[")]);
    assert_eq!(
        r.fakes.layouts.refreshes(),
        1,
        "no refresh on a fresh table"
    );
}

#[test]
fn relabel_redraws_only_when_the_layout_shown_changed() {
    let r = Rig::new(&GOLDEN, None);
    assert_eq!(r.act(FaceAction::Relabel), Ok(FaceReply::default()));
    assert_eq!(r.switch("fr-fr"), STATE);
    assert_eq!(r.shown(), "FR");
    assert_eq!(r.state()["keys"][Q]["main"], "a", "AZERTY");
}

#[test]
fn relabel_redraws_when_caps_lock_changed_outside() {
    let r = Rig::new(&GOLDEN, None);
    assert_eq!(r.state()["keys"]["caps"]["on"], false);
    r.fakes.source.set_caps(true);
    assert_eq!(
        r.act(FaceAction::Relabel),
        Ok(STATE),
        "a real Caps Lock press"
    );
    assert_eq!(r.state()["keys"]["caps"]["on"], true);
    assert_eq!(
        r.act(FaceAction::Relabel),
        Ok(FaceReply::default()),
        "already shown"
    );
}

#[test]
fn a_settings_change_keeps_the_latches_and_takes_the_new_rules() {
    let mut r = Rig::new(&GOLDEN, None);
    r.tap("shift").unwrap();
    r.tap("ctrl").unwrap();
    r.restart(Some(("lang_key", "cycle".into())));
    let state = r.state();
    assert_eq!(state["keys"]["shift"]["on"], true);
    assert_eq!(state["keys"]["ctrl"]["on"], true);
    assert_eq!(r.board()["lang_key"], "cycle");
    r.restart(Some(("modifiers", "stay".into())));
    r.tap(Q).unwrap();
    assert_eq!(
        r.state()["keys"]["shift"]["on"],
        true,
        "stay keeps them after a key"
    );
}
