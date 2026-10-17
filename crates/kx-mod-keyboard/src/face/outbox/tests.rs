use super::*;
use std::sync::atomic::AtomicUsize;
use std::sync::mpsc;
use std::sync::{Mutex, PoisonError};
use std::thread;
use std::time::Duration;

/// How long a thread waits for the other side before the test fails.
const WAIT: Duration = Duration::from_secs(10);

#[test]
fn a_post_sends_once() {
    let (o, sent) = (Outbox::default(), AtomicUsize::new(0));
    o.post(|| {
        sent.fetch_add(1, SeqCst);
    });
    assert_eq!(sent.load(SeqCst), 1);
}

#[test]
fn a_post_from_inside_a_send_sends_again_after_it_returns() {
    let o = Outbox::default();
    let log = Mutex::new(Vec::new());
    let send = || {
        let n = {
            let mut log = log.lock().unwrap_or_else(PoisonError::into_inner);
            log.push("send");
            log.len()
        };
        if n == 1 {
            o.post(|| unreachable!("the outer sender sends"));
            log.lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push("asked");
        }
    };
    o.post(send);
    assert_eq!(*log.lock().unwrap(), ["send", "asked", "send"]);
}

#[test]
fn a_post_while_another_thread_sends_returns_at_once_and_that_thread_sends_again() {
    let o = Outbox::default();
    let (inside, entered) = mpsc::channel();
    let (go, gate) = mpsc::channel::<()>();
    let sent = AtomicUsize::new(0);
    thread::scope(|s| {
        let (o, sent) = (&o, &sent);
        let sender = s.spawn(move || {
            o.post(|| {
                if sent.fetch_add(1, SeqCst) == 0 {
                    inside.send(()).unwrap();
                    gate.recv_timeout(WAIT).unwrap();
                }
            });
        });
        entered.recv_timeout(WAIT).unwrap();
        o.post(|| unreachable!("the other thread sends"));
        go.send(()).unwrap();
        sender.join().unwrap();
    });
    assert_eq!(sent.load(SeqCst), 2, "the busy sender took the request");
}
