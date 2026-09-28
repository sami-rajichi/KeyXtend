//! The latest note from a worker thread, kept until the UI takes it on its next tick.

use std::sync::{Arc, Mutex};

/// The latest note, shared between a worker thread and the UI.
pub type Note = Arc<Mutex<Option<String>>>;

/// Replaces the note with `text`.
pub fn put(note: &Note, text: String) {
    if let Ok(mut n) = note.lock() {
        *n = Some(text);
    }
}

/// Takes the latest note, if there is one.
pub fn take(note: &Note) -> Option<String> {
    note.lock().ok().and_then(|mut n| n.take())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_latest_note_is_taken_once() {
        let note: Note = Arc::default();
        put(&note, "old".into());
        put(&note, "new".into());
        assert_eq!(take(&note).as_deref(), Some("new"));
        assert_eq!(take(&note), None);
    }
}
