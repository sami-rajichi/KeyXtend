//! The session watch: its name and open gate, the notes in order, which ones mean the user is away, and the errors.

use super::*;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

/// A watch that tells each note once at start, as a platform would at a lock, an unlock, a sleep and a wake.
struct Told;

/// The run of `Told`.
struct Run;

impl SessionRun for Run {}

impl SessionWatch for Told {
    fn watch(
        &self,
        tell: Box<dyn Fn(SessionNote) + Send>,
    ) -> Result<Box<dyn SessionRun>, SessionError> {
        SessionNote::ALL.into_iter().for_each(tell);
        Ok(Box::new(Run))
    }
}

#[test]
fn the_watch_is_named_and_needs_no_capability() {
    assert_eq!(SessionWatchKey::ID.as_str(), "session-watch");
    assert_eq!(SessionWatchKey::CAPABILITY, None, "ADR-0042 decision 9");
}

#[test]
fn the_notes_are_lock_unlock_sleep_and_wake() {
    let want = [
        SessionNote::Locked,
        SessionNote::Unlocked,
        SessionNote::Sleeping,
        SessionNote::Woke,
    ];
    assert_eq!(SessionNote::ALL, want);
}

#[test]
fn only_a_lock_or_a_sleep_means_the_user_is_away() {
    let away = SessionNote::ALL.map(SessionNote::away);
    assert_eq!(
        away,
        [true, false, true, false],
        "a dictation stops and wipes"
    );
}

#[test]
fn a_watch_tells_each_note_through_its_port() {
    let heard = Arc::new(Mutex::new(Vec::new()));
    let keep = Arc::clone(&heard);
    let watch: &dyn SessionWatch = &Told;
    let run = watch.watch(Box::new(move |n| keep.lock().unwrap().push(n)));
    assert!(run.is_ok());
    assert_eq!(*heard.lock().unwrap(), SessionNote::ALL);
}

#[test]
fn each_error_says_something_else() {
    let errors = [SessionError::Unavailable, SessionError::Refused];
    let texts: HashSet<_> = errors.iter().map(ToString::to_string).collect();
    assert_eq!(texts.len(), errors.len());
}
