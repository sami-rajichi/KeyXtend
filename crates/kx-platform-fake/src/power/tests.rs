//! The fake power dialog: opens counted, a set error refused until cleared, clones that share the count, and a waiter
//! woken by an open on another thread.

use super::*;
use std::sync::Arc;
use std::time::Duration;

/// The longest a test waits for the other thread; it never sleeps.
const WAIT: Duration = Duration::from_secs(10);

#[test]
fn each_open_is_counted() {
    let fake = FakePowerDialog::new();
    assert_eq!(fake.opened(), 0);
    assert_eq!(fake.open(), Ok(()));
    assert_eq!(fake.open(), Ok(()));
    assert_eq!(fake.opened(), 2);
}

#[test]
fn a_set_error_refuses_every_later_open_until_cleared() {
    let fake = FakePowerDialog::new();
    fake.set_error(Some(PowerError::Unavailable));
    assert_eq!(fake.open(), Err(PowerError::Unavailable));
    fake.set_error(Some(PowerError::Failed));
    assert_eq!(fake.open(), Err(PowerError::Failed));
    assert_eq!(fake.opened(), 0, "a refused open shows no dialog");
    fake.set_error(None);
    assert_eq!(fake.open(), Ok(()));
    assert_eq!(fake.opened(), 1);
}

#[test]
fn clones_share_the_count_and_serve_as_the_port_from_another_thread() {
    let fake = FakePowerDialog::new();
    let port: Arc<dyn PowerDialog> = Arc::new(fake.clone());
    std::thread::spawn(move || port.open())
        .join()
        .unwrap()
        .unwrap();
    assert_eq!(fake.opened(), 1);
}

#[test]
fn a_waiter_wakes_when_another_thread_opens_and_says_when_none_came() {
    let fake = FakePowerDialog::new();
    assert!(!fake.wait_opened(1, Duration::ZERO), "nothing opened yet");
    let port: Arc<dyn PowerDialog> = Arc::new(fake.clone());
    let opener = std::thread::spawn(move || port.open());
    assert!(fake.wait_opened(1, WAIT));
    opener.join().unwrap().unwrap();
    assert!(
        fake.wait_opened(1, Duration::ZERO),
        "an open already counted"
    );
    assert!(!fake.wait_opened(2, Duration::ZERO));
}
