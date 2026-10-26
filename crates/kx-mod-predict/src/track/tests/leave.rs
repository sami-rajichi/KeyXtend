//! An email the user leaves: the open token's full address of typed characters is told once, and its end then tells
//! it no more; focus on the same box is no leave.

use kx_module_api::Redacted;

use super::*;

/// `text` as an email the tracker tells.
fn email(text: &str) -> Redacted<String> {
    Redacted::new(text.to_owned())
}

#[test]
fn an_open_typed_full_address_is_told_once() {
    let mut t = tracker(2);
    type_text(&mut t, "mail sami@x.org");
    assert_eq!(t.email(), Some(email("sami@x.org")));
    assert_eq!(t.email(), None, "told once");
    assert!(
        type_text(&mut t, " ").is_empty(),
        "its end tells it no more"
    );
}

#[test]
fn an_address_changed_after_it_was_told_is_told_at_its_end() {
    let mut t = tracker(2);
    type_text(&mut t, "sami@x.org");
    assert_eq!(t.email(), Some(email("sami@x.org")));
    let told = type_text(&mut t, ".tn ");
    assert_eq!(told, [Told::Email(email("sami@x.org.tn"))]);
}

#[test]
fn a_partial_or_read_address_or_a_word_is_never_told() {
    let mut t = tracker(2);
    type_text(&mut t, "sami@gm");
    assert_eq!(t.email(), None, "no domain yet");
    let mut t = tracker(2);
    t.read(Some("sami@x.org"), &knows);
    assert_eq!(t.email(), None, "a read token");
    let mut t = tracker(2);
    type_text(&mut t, "hello");
    assert_eq!(t.email(), None);
}

#[test]
fn only_focus_on_another_box_leaves_it() {
    let mut t = tracker(2);
    let first = FieldChange {
        kind: FieldKind::Text,
        box_id: 1,
    };
    assert!(t.leaves(first));
    t.focus(first);
    assert!(!t.leaves(first), "the same box");
    assert!(t.leaves(FieldChange { box_id: 2, ..first }));
}
