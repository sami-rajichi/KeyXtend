//! The fake running copy, held to the port: one claim, listeners outside the lock, drops that wait.

use super::*;

use std::sync::mpsc::{self, TryRecvError};

/// How long a test waits for a send on another thread; a wait, never a sleep.
const WAIT: std::time::Duration = std::time::Duration::from_secs(5);

#[test]
fn only_one_copy_claims_until_it_lets_go() {
    let first = FakeInstance::new();
    let second = first.clone();
    let claim = first.claim().unwrap();
    assert!(matches!(second.claim(), Err(Running)));
    drop(claim);
    assert!(second.claim().is_ok());
}

#[test]
fn a_later_copy_reaches_the_listener() {
    let running = FakeInstance::new();
    let later = running.clone();
    let heard = Arc::new(Mutex::new(Vec::new()));
    let keep = Arc::clone(&heard);
    let mut claim = running.claim().unwrap();
    claim
        .listen(Box::new(move |c| keep.lock().unwrap().push(c)))
        .unwrap();
    assert_eq!(later.send(Command::Show), Ok(()));
    assert_eq!(later.send(Command::Quit), Ok(()));
    assert_eq!(*heard.lock().unwrap(), [Command::Show, Command::Quit]);
    assert_eq!(running.sent(), [Command::Show, Command::Quit]);
}

#[test]
fn a_running_copy_that_does_not_listen_yet_has_no_window() {
    let running = FakeInstance::new();
    let later = running.clone();
    let _claim = running.claim().unwrap();
    assert_eq!(later.send(Command::Show), Err(UiError::NoWindow));
    assert!(running.sent().is_empty());
}

#[test]
fn a_later_listen_replaces_the_first_and_a_refused_one_keeps_it() {
    let running = FakeInstance::new();
    let heard = Arc::new(Mutex::new(Vec::new()));
    let hear = |tag: &'static str| {
        let keep = Arc::clone(&heard);
        Box::new(move |c: Command| keep.lock().unwrap().push((tag, c)))
    };
    let mut claim = running.claim().unwrap();
    claim.listen(hear("first")).unwrap();
    claim.listen(hear("second")).unwrap();
    running.set_listen_error(Some(UiError::Refused(9)));
    assert_eq!(claim.listen(hear("third")), Err(UiError::Refused(9)));
    running.send(Command::Show).unwrap();
    assert_eq!(*heard.lock().unwrap(), [("second", Command::Show)]);
}

#[test]
fn a_listener_may_listen_again() {
    type Shared = Arc<Mutex<Option<Box<dyn Claim>>>>;
    let running = FakeInstance::new();
    let shared: Shared = Arc::new(Mutex::new(Some(running.claim().unwrap())));
    let heard = Arc::new(Mutex::new(Vec::new()));
    let (keep, into) = (Arc::clone(&shared), Arc::clone(&heard));
    let first = Box::new(move |_| {
        let into = Arc::clone(&into);
        let next = Box::new(move |c| into.lock().unwrap().push(c));
        keep.lock().unwrap().as_mut().unwrap().listen(next).unwrap();
    });
    shared
        .lock()
        .unwrap()
        .as_mut()
        .unwrap()
        .listen(first)
        .unwrap();
    let later = running.clone();
    let (tx, rx) = std::sync::mpsc::channel();
    // On its own thread, so a listener that locks the session fails the test instead of hanging it.
    std::thread::spawn(move || tx.send([later.send(Command::Show), later.send(Command::Quit)]));
    assert_eq!(rx.recv_timeout(WAIT), Ok([Ok(()), Ok(())]));
    assert_eq!(*heard.lock().unwrap(), [Command::Quit]);
    shared.lock().unwrap().take();
}

#[test]
fn a_refused_listen_leaves_no_window() {
    let running = FakeInstance::new();
    running.set_listen_error(Some(UiError::Refused(8)));
    let mut claim = running.claim().unwrap();
    assert_eq!(claim.listen(Box::new(|_| {})), Err(UiError::Refused(8)));
    assert_eq!(running.send(Command::Quit), Err(UiError::NoWindow));
}

#[test]
fn with_no_running_copy_a_send_finds_no_window() {
    let fake = FakeInstance::new();
    assert_eq!(fake.send(Command::Show), Err(UiError::NoWindow));
    assert!(fake.sent().is_empty());
}

/// The commands a test listener heard, oldest first.
type Heard = Arc<Mutex<Vec<Command>>>;

/// Each command a listener hears, and the listener that records them.
fn heard_list() -> (Heard, OnCommand) {
    let heard = Arc::new(Mutex::new(Vec::new()));
    let into = Arc::clone(&heard);
    (heard, Box::new(move |c| into.lock().unwrap().push(c)))
}

#[test]
fn a_send_from_inside_the_listener_is_heard_next() {
    let running = FakeInstance::new();
    let (heard, record) = heard_list();
    let (again, answers) = (running.clone(), Arc::new(Mutex::new(Vec::new())));
    let keep = Arc::clone(&answers);
    let mut claim = running.claim().unwrap();
    claim
        .listen(Box::new(move |c| {
            record(c);
            if c == Command::Show {
                keep.lock().unwrap().push(again.send(Command::Quit));
            }
        }))
        .unwrap();
    assert_eq!(running.send(Command::Show), Ok(()));
    assert_eq!(*answers.lock().unwrap(), [Ok(())], "the window is there");
    assert_eq!(*heard.lock().unwrap(), [Command::Show, Command::Quit]);
    assert_eq!(running.sent(), [Command::Show, Command::Quit]);
}

#[test]
fn a_send_while_the_listener_runs_on_another_thread_is_heard_next() {
    let running = FakeInstance::new();
    let (heard, record) = heard_list();
    let (started_tx, started) = mpsc::channel();
    let (go_tx, go) = mpsc::channel::<()>();
    let mut claim = running.claim().unwrap();
    claim
        .listen(Box::new(move |c| {
            if c == Command::Show {
                let _ = started_tx.send(());
                let _ = go.recv_timeout(WAIT);
            }
            record(c);
        }))
        .unwrap();
    let first = running.clone();
    let busy = std::thread::spawn(move || first.send(Command::Show));
    started.recv_timeout(WAIT).unwrap();
    assert_eq!(running.send(Command::Quit), Ok(()), "the window is there");
    let _ = go_tx.send(());
    assert_eq!(busy.join().unwrap(), Ok(()));
    assert_eq!(*heard.lock().unwrap(), [Command::Show, Command::Quit]);
}

/// True once a claim drop in `running` waits for its listener; false if none does within `WAIT`.
fn a_drop_waits(running: &FakeInstance) -> bool {
    let shared = &running.0;
    let session = lock(&shared.session);
    let waiting = shared
        .idle
        .wait_timeout_while(session, WAIT, |s| s.waiting == 0);
    !waiting.unwrap().1.timed_out()
}

#[test]
fn dropping_the_claim_waits_for_a_running_listener() {
    let running = FakeInstance::new();
    let log = Arc::new(Mutex::new(Vec::new()));
    let (started_tx, started) = mpsc::channel();
    let (go_tx, go) = mpsc::channel::<()>();
    let into = Arc::clone(&log);
    let mut claim = running.claim().unwrap();
    claim
        .listen(Box::new(move |_| {
            let _ = started_tx.send(());
            let _ = go.recv_timeout(WAIT);
            into.lock().unwrap().push("listener done");
        }))
        .unwrap();
    let first = running.clone();
    let busy = std::thread::spawn(move || first.send(Command::Show));
    started.recv_timeout(WAIT).unwrap();
    let (dropped_tx, dropped) = mpsc::channel();
    let into = Arc::clone(&log);
    std::thread::spawn(move || {
        drop(claim);
        into.lock().unwrap().push("dropped");
        let _ = dropped_tx.send(());
    });
    assert!(a_drop_waits(&running), "the drop waits for the listener");
    assert_eq!(
        dropped.try_recv(),
        Err(TryRecvError::Empty),
        "and has not returned"
    );
    let _ = go_tx.send(());
    assert_eq!(dropped.recv_timeout(WAIT), Ok(()));
    assert_eq!(busy.join().unwrap(), Ok(()));
    assert_eq!(*log.lock().unwrap(), ["listener done", "dropped"]);
}

#[test]
fn a_claim_made_again_during_the_call_does_not_get_the_old_listener() {
    type Shared = Arc<Mutex<Option<Box<dyn Claim>>>>;
    let running = FakeInstance::new();
    let shared: Shared = Arc::new(Mutex::new(Some(running.claim().unwrap())));
    let (keep, again) = (Arc::clone(&shared), running.clone());
    let reclaim = Box::new(move |_| {
        drop(keep.lock().unwrap().take());
        *keep.lock().unwrap() = again.claim().ok();
    });
    shared
        .lock()
        .unwrap()
        .as_mut()
        .unwrap()
        .listen(reclaim)
        .unwrap();
    assert_eq!(running.send(Command::Show), Ok(()));
    assert!(shared.lock().unwrap().is_some(), "a new copy claimed");
    let quiet = running.send(Command::Quit);
    assert_eq!(
        quiet,
        Err(UiError::NoWindow),
        "the new copy does not listen yet"
    );
    shared.lock().unwrap().take();
}
