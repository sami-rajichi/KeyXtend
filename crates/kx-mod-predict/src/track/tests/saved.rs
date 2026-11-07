//! A token a saved pick typed into: it teaches nothing, whatever is glued before the pick, added after it or erased,
//! and the mark ends with the token.

use kx_module_api::Redacted;
use kx_module_api::ports::Piece;

use super::*;

/// The saved email a pick types.
const SAVED: &str = "lea@carnet.fr";

/// A tracker that followed `before`, then a pick's echo of the saved email, marked.
fn marked(before: &str) -> Tracker {
    let mut t = tracker(2);
    type_text(&mut t, before);
    t.typed(&Piece::Text(Redacted::new(SAVED.to_owned())), &knows);
    t.mark_saved();
    t
}

#[test]
fn a_marked_token_tells_nothing_whatever_is_glued_before_or_added_after() {
    for (before, after) in [("alex", " "), ("أليكس", " "), ("", ".tn ")] {
        let mut t = marked(before);
        assert_eq!(t.email(), None, "{before:?}: leaving tells nothing");
        let told = type_text(&mut t, after);
        assert!(told.is_empty(), "{before:?} {after:?}: {told:?}");
    }
}

#[test]
fn a_marked_token_teaches_no_word_once_its_at_sign_is_erased() {
    let mut t = marked("");
    for _ in "@carnet.fr".chars() {
        t.typed(&Piece::Erase, &knows);
    }
    type_text(&mut t, "s");
    assert!(
        type_text(&mut t, " ").is_empty(),
        "the word left of it teaches nothing"
    );
}

#[test]
fn the_mark_ends_with_its_token_a_reset_or_a_read() {
    let mut t = marked("");
    type_text(&mut t, " ");
    assert_eq!(type_text(&mut t, "nour@y.com "), [mail("nour@y.com")]);
    let mut t = marked("");
    t.reset();
    assert_eq!(type_text(&mut t, "nour@y.com "), [mail("nour@y.com")]);
    let mut t = marked("");
    t.read(Some("hi "), &knows);
    assert_eq!(type_text(&mut t, "nour@y.com "), [mail("nour@y.com")]);
}
