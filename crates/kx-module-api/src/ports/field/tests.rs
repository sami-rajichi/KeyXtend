//! The field port: the watch's gate, and errors that each say something else.

use super::*;
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
