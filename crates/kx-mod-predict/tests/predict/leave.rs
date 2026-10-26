//! Emails learned when the user leaves them (owner, 2026-10-26): a domain chip that completes the address once its
//! text reaches the app, a click outside our windows or focus on another box; never a partial or read address, never
//! in a box that is no text box, and never while learning is paused or prediction is off.

use crate::rig::Rig;
use kx_mod_predict::Msg;
use kx_mod_predict::email::Emails;
use kx_module_api::ports::{FieldKind, Piece, WordsAsk};

/// A rig in English with focus on a new box of `kind`.
fn in_box(kind: FieldKind) -> Rig {
    let mut rig = Rig::new();
    rig.lang("en");
    rig.focus(kind);
    rig
}

/// Stops `rig` and gives the emails it saved, none when it saved none.
fn kept(rig: &mut Rig) -> Vec<String> {
    rig.stop();
    let name = kx_mod_predict::emails_blob().unwrap();
    let bytes = rig.fakes.store.blob(&name);
    bytes.map_or_else(Vec::new, |b| {
        Emails::from_bytes(&b, usize::MAX).unwrap().list().to_vec()
    })
}

/// Typed starts and the rest the first domain chip types after each: after the `@` alone, which it replaces none of, or
/// after the domain's first letter.
const STARTS: [(&str, &str); 2] = [("sami@", "gmail.com"), ("sami@g", "mail.com")];

/// Types `start`, picks the first domain chip, which types `rest`, then the keyboard tells that text when `told`.
fn domain_picked(rig: &mut Rig, (start, rest): (&str, &str), told: bool) {
    rig.typed(start);
    rig.pick(0);
    assert_eq!(rig.typed_words(), [(0, rest.to_owned())], "{start}");
    if told {
        rig.typed(rest);
    }
}

#[test]
fn a_domain_chip_that_completes_the_address_learns_it_once_its_text_reaches_the_app() {
    for start in STARTS {
        let mut rig = in_box(FieldKind::Text);
        domain_picked(&mut rig, start, true);
        assert_eq!(kept(&mut rig), ["sami@gmail.com"], "{}", start.0);
        let mut rig = in_box(FieldKind::Text);
        domain_picked(&mut rig, start, false);
        assert!(kept(&mut rig).is_empty(), "the keyboard never told it");
    }
}

#[test]
fn another_text_after_a_domain_chip_learns_nothing() {
    let mut rig = in_box(FieldKind::Text);
    domain_picked(&mut rig, STARTS[1], false);
    rig.send(Msg::Typed(Piece::Break));
    rig.typed("sami@gmail.com");
    assert!(kept(&mut rig).is_empty(), "the break ended the wait");
}

#[test]
fn a_click_away_after_a_typed_full_address_learns_it() {
    let mut rig = in_box(FieldKind::Text);
    rig.typed("sami@x.org");
    rig.send(Msg::Pressed(false));
    assert_eq!(kept(&mut rig), ["sami@x.org"]);
}

#[test]
fn a_click_away_after_a_partial_address_or_on_our_window_learns_nothing() {
    let mut rig = in_box(FieldKind::Text);
    rig.typed("sami@gm");
    rig.send(Msg::Pressed(false));
    assert!(kept(&mut rig).is_empty());
    let mut rig = in_box(FieldKind::Text);
    rig.typed("sami@x.org");
    rig.send(Msg::Pressed(true));
    assert!(kept(&mut rig).is_empty(), "the user is still on it");
}

#[test]
fn focus_on_another_text_box_learns_the_typed_address() {
    let mut rig = in_box(FieldKind::Text);
    rig.typed("sami@x.org");
    rig.focus(FieldKind::Text);
    assert_eq!(kept(&mut rig), ["sami@x.org"]);
}

#[test]
fn a_leave_the_watch_already_sees_as_a_password_box_or_another_kind_learns_nothing() {
    for kind in [FieldKind::Password, FieldKind::Unknown, FieldKind::None] {
        let mut rig = in_box(FieldKind::Text);
        rig.typed("sami@x.org");
        rig.focus(kind);
        assert!(kept(&mut rig).is_empty(), "focus on {kind:?}");
        let mut rig = in_box(FieldKind::Text);
        rig.typed("sami@x.org");
        rig.fakes.field.focus(kind);
        rig.late(Msg::Pressed(false));
        assert!(kept(&mut rig).is_empty(), "a click with {kind:?} seen");
    }
}

#[test]
fn a_box_that_is_no_text_box_learns_nothing_on_leave() {
    for kind in [FieldKind::Unknown, FieldKind::None] {
        let mut rig = in_box(kind);
        rig.typed("sami@x.org");
        rig.send(Msg::Pressed(false));
        assert!(kept(&mut rig).is_empty(), "{kind:?}");
    }
}

#[test]
fn paused_learning_or_prediction_off_learns_nothing_on_leave() {
    for ask in [WordsAsk::Pause(true), WordsAsk::Power(false)] {
        let mut rig = in_box(FieldKind::Text);
        rig.ask(ask);
        rig.typed("sami@x.org");
        rig.send(Msg::Pressed(false));
        assert!(kept(&mut rig).is_empty(), "{ask:?}");
    }
    let mut rig = in_box(FieldKind::Text);
    rig.ask(WordsAsk::Pause(true));
    domain_picked(&mut rig, STARTS[0], true);
    rig.typed(" ");
    assert!(kept(&mut rig).is_empty(), "a domain chip while paused");
}

#[test]
fn a_read_address_learns_nothing_on_leave() {
    let mut rig = Rig::build(|f, _| f.field.set_before(Some("to sami@x.org")));
    rig.lang("en");
    rig.focus(FieldKind::Text);
    rig.send(Msg::Pressed(false));
    assert!(kept(&mut rig).is_empty());
}

#[test]
fn a_break_still_ends_a_typed_address_unlearned() {
    let mut rig = in_box(FieldKind::Text);
    rig.typed("sami@x.org");
    rig.send(Msg::Typed(Piece::Break));
    rig.send(Msg::Pressed(false));
    assert!(kept(&mut rig).is_empty());
}
