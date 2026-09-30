//! The one running copy on this PC: each test uses its own app name, so no test meets another or a real copy.

use super::*;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::Duration;

/// How long a test waits for a posted command; a wait, never a sleep.
const WAIT: Duration = Duration::from_secs(5);

/// An app name no other test and no real copy uses.
fn app(test: &str) -> WinInstance {
    WinInstance::new(&format!("KeyXtendTest.{}.{test}", std::process::id()))
}

/// A listener that forwards each command, and where they arrive.
fn heard_list() -> (OnCommand, Receiver<Command>) {
    let (tx, rx) = mpsc::channel();
    (
        Box::new(move |c| {
            let _ = tx.send(c);
        }),
        rx,
    )
}

#[test]
fn only_one_copy_claims_until_it_lets_go() {
    let copy = app("claim");
    let claim = copy.claim().unwrap();
    assert!(matches!(copy.clone().claim(), Err(Running)));
    drop(claim);
    assert!(copy.claim().is_ok());
}

#[test]
fn a_later_copy_reaches_the_listener_on_its_own_thread() {
    let running = app("reach");
    let mut claim = running.claim().unwrap();
    let (tx, rx) = mpsc::channel();
    let on: OnCommand = Box::new(move |c| {
        let _ = tx.send((c, std::thread::current().id()));
    });
    claim.listen(on).unwrap();
    let later = running.clone();
    assert_eq!(later.send(Command::Show), Ok(()));
    assert_eq!(later.send(Command::Quit), Ok(()));
    let (first, thread) = rx.recv_timeout(WAIT).unwrap();
    assert_eq!(first, Command::Show);
    assert_ne!(
        thread,
        std::thread::current().id(),
        "the listener's own thread"
    );
    assert_eq!(rx.recv_timeout(WAIT).unwrap().0, Command::Quit);
}

#[test]
fn a_second_listen_replaces_the_first() {
    let running = app("replace");
    let mut claim = running.claim().unwrap();
    let (first, old) = heard_list();
    let (second, new) = heard_list();
    claim.listen(first).unwrap();
    claim.listen(second).unwrap();
    running.send(Command::Show).unwrap();
    assert_eq!(new.recv_timeout(WAIT), Ok(Command::Show));
    let gone = Err(TryRecvError::Disconnected);
    assert_eq!(
        old.try_recv(),
        gone,
        "the first listener was dropped, unheard"
    );
}

#[test]
fn a_running_copy_that_does_not_listen_yet_has_no_window() {
    let running = app("quiet");
    let _claim = running.claim().unwrap();
    assert_eq!(running.send(Command::Show), Err(UiError::NoWindow));
}

#[test]
fn with_no_running_copy_a_send_finds_no_window() {
    assert_eq!(app("none").send(Command::Quit), Err(UiError::NoWindow));
}

#[test]
fn a_dropped_claim_closes_its_window() {
    let running = app("closed");
    let mut claim = running.claim().unwrap();
    let (on, _rx) = heard_list();
    claim.listen(on).unwrap();
    drop(claim);
    assert_eq!(running.send(Command::Show), Err(UiError::NoWindow));
}

#[test]
fn an_unknown_code_reaches_no_listener() {
    let running = app("unknown");
    let mut claim = running.claim().unwrap();
    let (on, rx) = heard_list();
    claim.listen(on).unwrap();
    assert!(post_code(&running, usize::MAX));
    running.send(Command::Quit).unwrap();
    assert_eq!(
        rx.recv_timeout(WAIT),
        Ok(Command::Quit),
        "only the known one"
    );
}

/// Posts `code` to the command window of `copy`, as another program could.
#[allow(unsafe_code, reason = "Posts a bad code to the test's own window.")]
fn post_code(copy: &WinInstance, code: usize) -> bool {
    let class = PCWSTR(copy.class.as_ptr());
    // SAFETY: the class name ends with a null; the post carries plain numbers.
    unsafe {
        FindWindowExW(Some(HWND_MESSAGE), None, class, PCWSTR::null())
            .and_then(|w| PostMessageW(Some(w), COMMAND_MSG, WPARAM(code), LPARAM(0)))
            .is_ok()
    }
}

#[test]
fn when_windows_cannot_check_each_copy_runs() {
    let bad = WinInstance::new(r"KeyXtendTest\no\such\path");
    let first = bad.claim();
    assert!(first.is_ok() && bad.claim().is_ok());
}
