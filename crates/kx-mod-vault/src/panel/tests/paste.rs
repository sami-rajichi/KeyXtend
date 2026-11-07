//! Paste into a New or Edit field: the text replaces the value of the field it was asked for under the field rules
//! (`field_chars`); each other outcome, a wait that ran out, or a form left meanwhile ends it.

use kx_module_api::ports::{ClipText, EntryKind, FieldButton, FillButton, FillNote, Typing};

use super::super::Do;
use super::super::rig::Rig;
use super::button;
use super::edit::{at, limited, new, sheet, values};
use crate::book::Limits;
use crate::fixture::{LIMITS, secret};

/// Paste's answer holding `text` as the clipboard held it.
fn text(text: &str) -> ClipText {
    ClipText::Text(secret(text))
}

/// A New login with Paste asked on field `i`.
fn asked(r: &mut Rig, i: u8) {
    new(r, EntryKind::Login);
    assert_eq!(r.click(at(i, FieldButton::Paste)), Do::Paste);
}

/// Hands Paste's answer in, then draws.
fn pasted(r: &mut Rig, found: &ClipText) -> Do {
    let done = r.panel.pasted(found);
    r.draw();
    done
}

#[test]
fn the_text_replaces_the_value_of_the_field_paste_was_asked_on() {
    let mut r = Rig::sample();
    asked(&mut r, 0);
    assert_eq!(sheet(&r).typing, Some(Typing::Value(0)));
    r.text("old");
    assert_eq!(pasted(&mut r, &text("sam\r\nmartin")), Do::Show);
    assert_eq!(values(&r)[0].as_deref(), Some("sam martin"));
    assert_eq!(r.page.note, None);
}

#[test]
fn no_text_shows_the_empty_note_and_changes_nothing() {
    let mut r = Rig::sample();
    asked(&mut r, 0);
    assert_eq!(pasted(&mut r, &ClipText::Empty), Do::Show);
    assert_eq!(r.page.note.map(|n| n.note), Some(FillNote::NoText));
    assert_eq!(values(&r)[0].as_deref(), Some(""));
}

#[test]
fn a_text_past_field_chars_is_refused_with_its_note() {
    let mut r = limited(Limits {
        field_chars: 8,
        ..LIMITS
    });
    asked(&mut r, 0);
    assert_eq!(pasted(&mut r, &text("123456789")), Do::Show);
    assert_eq!(r.page.note.map(|n| n.note), Some(FillNote::Refused));
    assert_eq!(values(&r)[0].as_deref(), Some(""));
    r.click(at(0, FieldButton::Paste));
    pasted(&mut r, &text("12345678"));
    assert_eq!(values(&r)[0].as_deref(), Some("12345678"));
}

#[test]
fn a_shown_value_loses_the_bidi_marks_and_a_hidden_one_refuses_them() {
    let mut r = Rig::sample();
    asked(&mut r, 0);
    pasted(&mut r, &text("06\u{200f} 12"));
    assert_eq!(values(&r)[0].as_deref(), Some("06 12"));
    r.click(at(1, FieldButton::Paste));
    assert_eq!(pasted(&mut r, &text("pa\u{200f}ss")), Do::Show);
    assert_eq!(r.page.note.map(|n| n.note), Some(FillNote::Refused));
    r.click(at(0, FieldButton::Paste));
    pasted(&mut r, &text("a\u{200b}b"));
    assert_eq!(
        r.page.note.map(|n| n.note),
        Some(FillNote::Refused),
        "other marks"
    );
    assert_eq!(values(&r)[0].as_deref(), Some("06 12"));
}

#[test]
fn a_hidden_field_takes_the_text_and_stays_masked() {
    let mut r = Rig::sample();
    asked(&mut r, 1);
    pasted(&mut r, &text("hunter2"));
    assert_eq!(values(&r)[1], None, "masked");
    r.click(at(1, FieldButton::Show));
    assert_eq!(values(&r)[1].as_deref(), Some("hunter2"));
}

#[test]
fn a_wait_that_ran_out_says_the_history_is_off_and_drops_the_late_answer() {
    let mut r = Rig::sample();
    asked(&mut r, 0);
    assert_eq!(r.panel.paste_off(), Do::Show);
    r.draw();
    assert_eq!(r.page.note.map(|n| n.note), Some(FillNote::ClipOff));
    assert_eq!(pasted(&mut r, &text("late")), Do::Stay);
    assert_eq!(values(&r)[0].as_deref(), Some(""));
    assert_eq!(r.panel.paste_off(), Do::Stay, "once");
}

#[test]
fn a_form_left_meanwhile_drops_the_answer() {
    let mut r = Rig::sample();
    asked(&mut r, 0);
    r.click(button(FillButton::Cancel));
    assert_eq!(pasted(&mut r, &text("late")), Do::Stay);
    assert_eq!(r.panel.paste_off(), Do::Stay);
    new(&mut r, EntryKind::Login);
    assert_eq!(
        pasted(&mut r, &text("late")),
        Do::Stay,
        "a new form asked nothing"
    );
    assert_eq!(values(&r)[0].as_deref(), Some(""));
}

#[test]
fn a_copied_password_loses_its_trailing_break_and_no_space_hides_in_it() {
    let mut r = Rig::sample();
    asked(&mut r, 1);
    assert_eq!(pasted(&mut r, &text("hunter2\r\n")), Do::Show);
    r.click(at(1, FieldButton::Show));
    assert_eq!(values(&r)[1].as_deref(), Some("hunter2"));
}

#[test]
fn a_break_inside_a_hidden_value_refuses_the_paste_and_keeps_the_value() {
    let mut r = Rig::sample();
    asked(&mut r, 1);
    r.text("old");
    assert_eq!(pasted(&mut r, &text("user\r\npassword")), Do::Show);
    assert_eq!(r.page.note.map(|n| n.note), Some(FillNote::Refused));
    r.click(at(1, FieldButton::Show));
    assert_eq!(values(&r)[1].as_deref(), Some("old"));
}

#[test]
fn each_outcome_that_is_no_text_shows_its_own_note() {
    let mut r = Rig::sample();
    for (found, note) in [
        (text("\r\n"), FillNote::NoText),
        (ClipText::Empty, FillNote::NoText),
        (ClipText::TooLong, FillNote::Refused),
        (ClipText::Busy, FillNote::ClipBusy),
    ] {
        asked(&mut r, 0);
        assert_eq!(pasted(&mut r, &found), Do::Show, "{found:?}");
        assert_eq!(r.page.note.map(|n| n.note), Some(note), "{found:?}");
        assert_eq!(values(&r)[0].as_deref(), Some(""), "{found:?}");
        r.click(button(FillButton::Cancel));
    }
}
