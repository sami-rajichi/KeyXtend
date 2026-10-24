//! The field port: the watch's gate, errors that each say something else, and the words cut every watch shares.

use super::*;
use proptest::prelude::*;
use std::collections::HashSet;

#[test]
fn the_watch_is_named_and_gated_by_read_focused_text() {
    assert_eq!(FieldWatchKey::ID.as_str(), "field-watch");
    assert_eq!(FieldWatchKey::CAPABILITY, Some(Capability::ReadFocusedText));
}

#[test]
fn each_error_says_something_else() {
    let errors = [
        FieldError::Unavailable,
        FieldError::TimedOut,
        FieldError::Refused,
    ];
    let texts: HashSet<_> = errors.iter().map(ToString::to_string).collect();
    assert_eq!(texts.len(), errors.len());
}

#[test]
fn the_cut_keeps_the_last_whole_words_in_latin_and_arabic() {
    assert_eq!(last_words("I really like ap", 2), "like ap");
    assert_eq!(
        last_words("  I like ", 1),
        "like ",
        "the space after the word stays"
    );
    assert_eq!(last_words("أنا أحب القراءة", 2), "أحب القراءة");
    assert_eq!(last_words("one", 0), "");
    assert_eq!(last_words(" \t ", 3), "", "no words keeps nothing");
}

proptest! {
    #[test]
    fn the_cut_ends_the_text_and_keeps_at_most_n_whole_words(
        text in "[a-zé أ-ي\t]{0,40}",
        max in 0u8..8,
    ) {
        let cut = last_words(&text, max);
        let words = text.split_whitespace().count();
        prop_assert!(text.ends_with(cut));
        prop_assert_eq!(cut.split_whitespace().count(), words.min(usize::from(max)));
        prop_assert!(!cut.starts_with(char::is_whitespace), "it starts at a word");
        let head = &text[..text.len() - cut.len()];
        let whole = cut.is_empty() || head.is_empty() || head.ends_with(char::is_whitespace);
        prop_assert!(whole, "it never starts mid-word");
    }
}
