//! An email the user leaves: the open token's full address whose at sign was typed is told once, and its end tells
//! only a changed address, over the one told; focus on the same box is no leave.

use super::*;

/// `text` as an email the tracker tells first.
fn email(text: &str) -> Address {
    Address::of(text, None)
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
fn an_address_changed_after_it_was_told_is_told_at_its_end_over_the_one_told() {
    let mut t = tracker(2);
    type_text(&mut t, "sami@x.org");
    assert_eq!(t.email(), Some(email("sami@x.org")));
    let told = type_text(&mut t, ".tn ");
    let longer = Address::of("sami@x.org.tn", Some("sami@x.org"));
    assert_eq!(told, [Told::Email(longer)]);
    let mut t = tracker(2);
    type_text(&mut t, "sami@x.org");
    t.email();
    type_text(&mut t, ".tn");
    let left = Address::of("sami@x.org.tn", Some("sami@x.org"));
    assert_eq!(t.email(), Some(left), "left, not ended");
}

#[test]
fn a_partial_or_read_address_or_a_word_is_never_told() {
    let mut t = tracker(2);
    type_text(&mut t, "sami@gm");
    assert_eq!(t.email(), None, "no domain yet");
    let mut t = tracker(2);
    t.read(Some("sami@x.org"), &knows);
    assert_eq!(t.email(), None, "a read token");
    for typed in [",", "m"] {
        let mut t = tracker(2);
        t.read(Some("bob@x.com"), &knows);
        type_text(&mut t, typed);
        assert_eq!(t.email(), None, "a read address, then {typed:?}");
    }
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
