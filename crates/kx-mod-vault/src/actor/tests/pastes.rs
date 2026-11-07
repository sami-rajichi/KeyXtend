//! Paste through the actor: a field's Paste sends `ClipTake` with twice the field's characters, what it found fills
//! the field under the field rules or shows its note, and no answer within `paste_wait_ms` says the history is not on
//! (a fake clock, no sleep).

use std::time::Duration;

use kx_module_api::ports::{
    ClipTake, ClipTaken, ClipText, EntryKind, FieldButton, FillButton, FillClick, FillNote, Sheet,
};

use super::super::Msg;
use super::rig::Rig;
use crate::fixture::secret;
use crate::settings::VaultSettings;

/// The defaults.
fn defaults() -> VaultSettings {
    VaultSettings::from_defaults().unwrap()
}

/// A New login with Paste clicked on its user field.
fn asked(rig: &mut Rig) {
    rig.open();
    rig.click(FillClick::Button(FillButton::New));
    rig.click(FillClick::Kind(EntryKind::Login));
    rig.click(FillClick::Field {
        field: 0,
        button: FieldButton::Paste,
    });
}

/// The clipboard history answers the last take with `text`.
fn answer(rig: &mut Rig, text: ClipText) {
    let round = rig.takes().last().unwrap().round;
    rig.send(Msg::Taken(ClipTaken { round, text }));
}

/// The clipboard history answers the last take with the copied `text`.
fn copied(rig: &mut Rig, text: &str) {
    answer(rig, ClipText::Text(secret(text)));
}

/// The user field's value on the last page.
fn user(rig: &Rig) -> String {
    let Sheet::Edit(e) = rig.page().sheet else {
        panic!("not New or Edit");
    };
    let value = e.fields[0].value.text().map(|t| t.expose().to_string());
    value.unwrap_or_default()
}

#[test]
fn paste_asks_once_for_twice_the_fields_characters_and_its_text_fills_the_field() {
    let mut rig = Rig::new();
    asked(&mut rig);
    let takes = rig.takes();
    let field_chars = defaults().field_chars;
    assert_eq!(takes, [ClipTake::new(takes[0].round, field_chars)]);
    copied(&mut rig, "\r\nsam\r\nmartin\r\n");
    assert_eq!(user(&rig), "sam martin");
    assert_eq!(rig.note(), None);
}

#[test]
fn each_outcome_that_is_no_text_shows_its_note() {
    for (found, note) in [
        (ClipText::Empty, FillNote::NoText),
        (ClipText::TooLong, FillNote::Refused),
        (ClipText::Busy, FillNote::ClipBusy),
    ] {
        let mut rig = Rig::new();
        asked(&mut rig);
        answer(&mut rig, found);
        assert_eq!((rig.note(), user(&rig)), (Some(note), String::new()));
    }
}

#[test]
fn a_text_past_field_chars_is_refused_once_its_breaks_are_flat() {
    let mut rig = Rig::new();
    asked(&mut rig);
    let over = "x".repeat(defaults().field_chars + 1);
    copied(&mut rig, &over);
    assert_eq!(
        (rig.note(), user(&rig)),
        (Some(FillNote::Refused), String::new())
    );
}

#[test]
fn no_answer_within_the_wait_says_the_history_is_off_and_a_late_one_is_dropped() {
    let mut rig = Rig::new();
    let wait = defaults().paste_wait();
    asked(&mut rig);
    assert_eq!(rig.actor.wait(), Some(wait));
    rig.advance(wait.saturating_sub(Duration::from_millis(1)));
    assert_eq!(rig.note(), None);
    rig.advance(Duration::from_millis(1));
    assert_eq!(rig.note(), Some(FillNote::ClipOff));
    assert_eq!(rig.actor.wait(), None, "nothing waits");
    copied(&mut rig, "late");
    assert_eq!(user(&rig), "");
}

#[test]
fn closing_the_panel_ends_the_wait() {
    let mut rig = Rig::new();
    asked(&mut rig);
    rig.send(Msg::Close);
    assert_eq!(rig.actor.wait(), None);
}
