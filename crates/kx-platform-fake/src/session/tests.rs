//! The fake session watch, held to the port: lock, unlock, sleep and wake told on cue in order, a refused watch, a
//! dropped watch that hears no more, and a waiter woken by a watch on another thread.

use super::*;
use std::thread::{self, ThreadId};
use std::time::Duration;

/// How long a test waits for another thread; it never sleeps.
const WITHIN: Duration = Duration::from_secs(10);

/// What each test watch heard, with the thread it heard it on.
type Heard = Arc<Mutex<Vec<(SessionNote, ThreadId)>>>;

/// A started watch whose `tell` records each note.
fn watched(fake: &FakeSessionWatch) -> (Box<dyn SessionRun>, Heard) {
    let heard = Heard::default();
    let log = Arc::clone(&heard);
    let tell = move |note| log.lock().unwrap().push((note, thread::current().id()));
    (fake.watch(Box::new(tell)).ok().unwrap(), heard)
}

/// The notes `heard` holds, in order.
fn notes(heard: &Heard) -> Vec<SessionNote> {
    heard.lock().unwrap().iter().map(|(n, _)| *n).collect()
}

#[test]
fn lock_and_sleep_are_told_on_cue_in_order_on_the_tests_thread() {
    let fake = FakeSessionWatch::new();
    let (_run, heard) = watched(&fake);
    assert!(fake.watching());
    let told = [SessionNote::Locked, SessionNote::Sleeping];
    let wake = [SessionNote::Woke, SessionNote::Unlocked];
    for note in told.into_iter().chain(wake) {
        assert!(fake.tell(note), "{note:?}");
    }
    let me = thread::current().id();
    assert!(heard.lock().unwrap().iter().all(|(_, t)| *t == me));
    assert_eq!(notes(&heard), [told, wake].concat());
}

#[test]
fn a_note_with_no_watch_is_heard_by_none() {
    let fake = FakeSessionWatch::new();
    assert!(!fake.tell(SessionNote::Locked));
    let (run, heard) = watched(&fake);
    drop(run);
    assert!(!fake.watching());
    assert!(
        !fake.tell(SessionNote::Sleeping),
        "a dropped watch hears no more"
    );
    assert!(notes(&heard).is_empty());
}

#[test]
fn a_failed_watch_fails_once_then_watches() {
    let fake = FakeSessionWatch::new();
    for error in [SessionError::Unavailable, SessionError::Refused] {
        fake.fail_watch(error);
        assert_eq!(fake.watch(Box::new(|_| {})).err(), Some(error));
        assert_eq!(fake.watches(), 0, "a refused watch is not counted");
    }
    let (_run, _heard) = watched(&fake);
    assert_eq!(fake.watches(), 1);
}

#[test]
fn a_second_watch_takes_the_notes_and_the_first_drop_keeps_it() {
    let fake = FakeSessionWatch::new();
    let (first, old) = watched(&fake);
    let (_second, new) = watched(&fake);
    drop(first);
    assert!(fake.watching(), "the second watch still runs");
    assert!(fake.tell(SessionNote::Locked));
    assert!(notes(&old).is_empty());
    assert_eq!(notes(&new), [SessionNote::Locked]);
}

#[test]
fn clones_share_the_watch_and_a_waiter_wakes_for_another_thread() {
    let fake = FakeSessionWatch::new();
    let port: Arc<dyn SessionWatch> = Arc::new(fake.clone());
    let heard = Heard::default();
    let log = Arc::clone(&heard);
    let tell = move |note| log.lock().unwrap().push((note, thread::current().id()));
    let other = thread::spawn(move || port.watch(Box::new(tell)).ok());
    assert!(fake.wait_watching(WITHIN));
    let run = other.join().unwrap();
    assert!(run.is_some());
    assert!(fake.tell(SessionNote::Sleeping));
    assert_eq!(notes(&heard), [SessionNote::Sleeping]);
}

#[test]
fn debug_shows_the_counts() {
    let fake = FakeSessionWatch::new();
    let (_run, _heard) = watched(&fake);
    let shown = format!("{fake:?}");
    assert!(shown.contains("watches: 1"), "{shown}");
}
