//! The fake clipboard, held to the port: notes reach the live watch at once, the watch and its config are recorded, a
//! refusal on cue, and files answer as scripted.

use super::*;
use kx_module_api::ports::{AppRef, ClipPart, ClipWait};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::Duration;

/// How long a test waits for another thread; it never sleeps.
const WITHIN: Duration = Duration::from_secs(10);

/// Test values; the real ones come from the clipboard module's settings.
const CFG: ClipConfig = ClipConfig {
    max_bytes: 1024,
    wait: ClipWait {
        open_ms: 100,
        poll_ms: 10,
    },
};

/// A copy's size over `CFG.max_bytes`.
const BIG: u64 = 2048;

/// A file's size on the fake drive.
const SIZE: u64 = 4096;

/// A copy of `text` from no known app.
fn copied(text: &str) -> ClipNote {
    let parts = vec![ClipPart::Text(Redacted::new(text.to_owned()))];
    ClipNote::Copied { parts, app: None }
}

/// A test app.
fn notepad() -> AppRef {
    AppRef {
        id: "notepad.exe".to_owned(),
        name: "Notepad".to_owned(),
    }
}

/// Starts a watch on `clip` whose notes land in the returned receiver.
fn watch(clip: &FakeClipboard) -> (Box<dyn ClipRun>, Receiver<ClipNote>) {
    let (tx, rx) = mpsc::channel();
    let tell = move |note| drop(tx.send(note));
    (clip.watch(CFG, Box::new(tell)).ok().unwrap(), rx)
}

#[test]
fn pushed_copies_skips_and_clears_reach_the_live_watch_at_once() {
    let clip = FakeClipboard::new();
    let (_run, rx) = watch(&clip);
    let notes = [
        copied("أرسلت الملف"),
        ClipNote::Marked {
            app: Some(notepad()),
        },
        ClipNote::TooBig {
            app: None,
            bytes: BIG,
        },
        ClipNote::Busy,
        ClipNote::Cleared {
            app: Some(notepad()),
        },
    ];
    for note in &notes {
        clip.push(note.clone());
    }
    assert_eq!(
        rx.try_iter().collect::<Vec<_>>(),
        notes,
        "in order, at once"
    );
}

#[test]
fn a_note_with_no_watch_is_lost_and_a_stopped_watch_hears_nothing() {
    let clip = FakeClipboard::new();
    clip.push(copied("before"));
    let (run, rx) = watch(&clip);
    assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));
    drop(run);
    clip.push(copied("after"));
    assert_eq!(
        rx.try_recv(),
        Err(TryRecvError::Disconnected),
        "tell dropped"
    );
}

#[test]
fn the_watch_and_its_config_are_recorded_until_dropped() {
    let clip = FakeClipboard::new();
    assert!(!clip.watching());
    assert_eq!((clip.watches(), clip.config()), (0, None));
    let (run, _rx) = watch(&clip);
    assert!(clip.watching());
    assert_eq!((clip.watches(), clip.config()), (1, Some(CFG)));
    drop(run);
    assert!(!clip.watching());
    assert_eq!(clip.config(), Some(CFG), "kept after the stop");
}

#[test]
fn a_second_watch_replaces_the_firsts_tell() {
    let clip = FakeClipboard::new();
    let (first, old) = watch(&clip);
    let (_second, new) = watch(&clip);
    assert_eq!(old.try_recv(), Err(TryRecvError::Disconnected));
    drop(first);
    assert!(clip.watching(), "the second watch still holds");
    clip.push(copied("x"));
    assert_eq!(new.try_recv(), Ok(copied("x")));
}

#[test]
fn dropping_the_newer_watch_first_ends_its_tell() {
    let clip = FakeClipboard::new();
    let (_first, _old) = watch(&clip);
    let (second, new) = watch(&clip);
    drop(second);
    assert!(clip.watching(), "the first watch still holds");
    clip.push(copied("x"));
    assert_eq!(new.try_recv(), Err(TryRecvError::Disconnected));
}

#[test]
fn a_scripted_refusal_comes_back_once_and_records_no_watch() {
    let clip = FakeClipboard::new();
    clip.fail_watch(ClipError::Refused);
    let refused = clip.watch(CFG, Box::new(|_: ClipNote| {}));
    assert_eq!(refused.err(), Some(ClipError::Refused));
    assert!(!clip.watching());
    assert_eq!((clip.watches(), clip.config()), (0, None));
    let (_run, _rx) = watch(&clip);
    assert!(clip.watching());
}

#[test]
fn files_answer_as_scripted_in_order_and_unknown_paths_are_gone() {
    let clip = FakeClipboard::new();
    clip.set_file("C:/docs/تقرير.pdf", FileFact::Size(SIZE));
    clip.set_file("C:/docs/old.txt", FileFact::Gone);
    let paths = ["C:/docs/old.txt", "C:/docs/تقرير.pdf", "C:/never.txt"];
    let paths = Redacted::new(paths.map(PathBuf::from).to_vec());
    let facts = [FileFact::Gone, FileFact::Size(SIZE), FileFact::Gone];
    assert_eq!(clip.files(&paths), facts);
    clip.set_file("C:/docs/تقرير.pdf", FileFact::Gone);
    assert_eq!(clip.files(&paths), [FileFact::Gone; 3], "deleted since");
    assert_eq!(clip.checks(), 2);
}

#[test]
fn a_watch_started_on_another_thread_wakes_a_waiting_test() {
    let clip = FakeClipboard::new();
    assert!(!clip.wait_watching(Duration::ZERO), "no watch yet");
    let port: Arc<dyn Clipboard> = Arc::new(clip.clone());
    let (tx, rx) = mpsc::channel();
    let module = thread::spawn(move || port.watch(CFG, Box::new(move |n| drop(tx.send(n)))));
    assert!(clip.wait_watching(WITHIN));
    clip.push(copied("x"));
    assert_eq!(rx.recv_timeout(WITHIN), Ok(copied("x")));
    drop(module.join().unwrap());
    assert!(!clip.watching());
}

#[test]
fn debug_never_prints_paths() {
    let clip = FakeClipboard::new();
    clip.set_file("C:/secret/plan.txt", FileFact::Size(SIZE));
    let text = format!("{clip:?}");
    assert!(text.contains("FakeClipboard"));
    assert!(!text.contains("plan"));
}

#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "never returns")]
fn a_watch_error_the_port_never_returns_is_a_test_mistake() {
    FakeClipboard::new().fail_watch(ClipError::Busy);
}
