//! The one running copy on this PC: each test uses its own app name, so no test meets another or a real copy.

use super::*;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::Duration;
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowThreadProcessId, PostThreadMessageW, SendMessageW, WM_CLOSE, WM_QUIT,
};

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
fn a_listener_waiting_for_the_ui_thread_lets_that_thread_listen() {
    let running = app("handoff");
    let mut claim = running.claim().unwrap();
    let (started_tx, started) = mpsc::channel();
    let (go_tx, go) = mpsc::channel::<()>();
    let (done_tx, done) = mpsc::channel();
    let on: OnCommand = Box::new(move |_| {
        let _ = started_tx.send(());
        let _ = done_tx.send(go.recv_timeout(WAIT).is_ok());
    });
    claim.listen(on).unwrap();
    running.send(Command::Show).unwrap();
    started.recv_timeout(WAIT).unwrap();
    let (next, _rx) = heard_list();
    claim.listen(next).unwrap();
    let _ = go_tx.send(());
    assert_eq!(done.recv_timeout(WAIT), Ok(true), "the UI thread answered");
}

#[test]
fn a_listener_may_listen_again_from_its_own_thread() {
    type Shared = Arc<Mutex<Option<Box<dyn Claim>>>>;
    let running = app("relisten");
    let shared: Shared = Arc::new(Mutex::new(Some(running.claim().unwrap())));
    let (next, rx) = heard_list();
    let next = Mutex::new(Some(next));
    let keep = Arc::clone(&shared);
    let first: OnCommand = Box::new(move |_| {
        let replacement = next.lock().unwrap().take();
        if let (Some(on), Some(claim)) = (replacement, keep.lock().unwrap().as_mut()) {
            claim.listen(on).unwrap();
        }
    });
    shared
        .lock()
        .unwrap()
        .as_mut()
        .unwrap()
        .listen(first)
        .unwrap();
    running.send(Command::Show).unwrap();
    running.send(Command::Quit).unwrap();
    assert_eq!(rx.recv_timeout(WAIT), Ok(Command::Quit), "the new listener");
    shared.lock().unwrap().take();
}

#[test]
fn a_listener_that_panics_still_hears_the_next_command() {
    let running = app("panic");
    let mut claim = running.claim().unwrap();
    let (tx, rx) = mpsc::channel();
    let on: OnCommand = Box::new(move |c| {
        assert_ne!(c, Command::Show, "a listener bug");
        let _ = tx.send(c);
    });
    claim.listen(on).unwrap();
    running.send(Command::Show).unwrap();
    running.send(Command::Quit).unwrap();
    assert_eq!(rx.recv_timeout(WAIT), Ok(Command::Quit));
}

#[test]
fn another_program_cannot_close_the_command_window() {
    let running = app("close");
    let mut claim = running.claim().unwrap();
    let (on, rx) = heard_list();
    claim.listen(on).unwrap();
    close(&running);
    assert_eq!(
        running.send(Command::Show),
        Ok(()),
        "the window is still there"
    );
    assert_eq!(rx.recv_timeout(WAIT), Ok(Command::Show));
}

#[test]
fn a_quit_posted_to_the_command_window_does_not_end_it() {
    let running = app("quit");
    let mut claim = running.claim().unwrap();
    let (on, rx) = heard_list();
    claim.listen(on).unwrap();
    assert!(post(&running, WM_QUIT, 0));
    assert_eq!(
        running.send(Command::Show),
        Ok(()),
        "the window is still there"
    );
    assert_eq!(rx.recv_timeout(WAIT), Ok(Command::Show), "and still hears");
}

#[test]
fn a_thread_quit_from_another_program_does_not_end_the_command_window() {
    let running = app("threadquit");
    let mut claim = running.claim().unwrap();
    let (on, rx) = heard_list();
    claim.listen(on).unwrap();
    assert!(post_thread_quit(&running));
    assert_eq!(
        running.send(Command::Show),
        Ok(()),
        "the window is still there"
    );
    assert_eq!(rx.recv_timeout(WAIT), Ok(Command::Show), "and still hears");
}

/// Posts `WM_QUIT` with no window to the thread of `copy`'s command window, as another program could.
#[allow(
    unsafe_code,
    reason = "Posts a thread message to the test's own window thread."
)]
fn post_thread_quit(copy: &WinInstance) -> bool {
    let class = PCWSTR(copy.class.as_ptr());
    // SAFETY: the class name ends with a null; the post carries plain numbers.
    unsafe {
        FindWindowExW(Some(HWND_MESSAGE), None, class, PCWSTR::null()).is_ok_and(|w| {
            let thread = GetWindowThreadProcessId(w, None);
            PostThreadMessageW(thread, WM_QUIT, WPARAM(0), LPARAM(0)).is_ok()
        })
    }
}

/// Sends `WM_CLOSE` to the command window of `copy` and waits for its answer, as another program could.
#[allow(unsafe_code, reason = "Sends WM_CLOSE to the test's own window.")]
fn close(copy: &WinInstance) {
    let class = PCWSTR(copy.class.as_ptr());
    // SAFETY: the class name ends with a null; WM_CLOSE carries no pointers.
    unsafe {
        let window = FindWindowExW(Some(HWND_MESSAGE), None, class, PCWSTR::null()).unwrap();
        SendMessageW(window, WM_CLOSE, None, None);
    }
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
    assert!(post(&running, COMMAND_MSG, usize::MAX));
    running.send(Command::Quit).unwrap();
    assert_eq!(
        rx.recv_timeout(WAIT),
        Ok(Command::Quit),
        "only the known one"
    );
}

/// Posts `msg` with `code` to the command window of `copy`, as another program could.
#[allow(unsafe_code, reason = "Posts a message to the test's own window.")]
fn post(copy: &WinInstance, msg: u32, code: usize) -> bool {
    let class = PCWSTR(copy.class.as_ptr());
    // SAFETY: the class name ends with a null; the post carries plain numbers.
    unsafe {
        FindWindowExW(Some(HWND_MESSAGE), None, class, PCWSTR::null())
            .and_then(|w| PostMessageW(Some(w), msg, WPARAM(code), LPARAM(0)))
            .is_ok()
    }
}

#[test]
fn when_windows_cannot_check_each_copy_runs() {
    let bad = WinInstance::new(r"KeyXtendTest\no\such\path");
    let first = bad.claim();
    assert!(first.is_ok() && bad.claim().is_ok());
}
