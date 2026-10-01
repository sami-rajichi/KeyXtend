//! A second copy's commands reach the running copy through the relay: in order when they come before the hub exists,
//! and straight to the hub after. Its Quit ends the running app as the tray's Quit does.

use crate::rig::{Rig, watch};
use keyxtend::copy::{Reach, Start, start};
use keyxtend::drain;
use keyxtend::relay::Relay;
use kx_platform::ui::{Claim, Command};
use kx_platform_fake::FakeInstance;
use kx_ui::{Request, Wake};
use std::sync::Arc;
use std::time::Duration;

/// One try, so a copy that does not listen fails at once.
const ONCE: Reach = Reach {
    tries: 1,
    wait: Duration::ZERO,
};

/// The running copy's claim, listening through a new relay as `main` does right after it claims.
fn running(session: &FakeInstance) -> (Box<dyn Claim>, Arc<Relay>) {
    let Start::Run(mut claim) = start(session, Command::Show, ONCE, |_| {}) else {
        panic!("the first copy runs");
    };
    let relay = Arc::new(Relay::default());
    let heard = Arc::clone(&relay);
    claim.listen(Box::new(move |c| heard.hear(c))).unwrap();
    (claim, relay)
}

/// A later copy in `session` asking for `command`; true when the running copy took it.
fn later(session: &FakeInstance, command: Command) -> bool {
    matches!(start(&session.clone(), command, ONCE, |_| {}), Start::Done)
}

#[test]
fn commands_before_the_hub_wait_and_keep_their_order() {
    let session = FakeInstance::new();
    let (_claim, relay) = running(&session);
    assert!(later(&session, Command::Show) && later(&session, Command::Quit));
    let boot = Rig::new().load();
    let wakes = watch(&boot.hub);
    relay.attach(&boot.hub);
    assert_eq!(*wakes.lock().unwrap(), [Wake::Show], "Show went to the UI");
    assert_eq!(
        boot.requests.try_recv(),
        Ok(Request::Quit),
        "Quit to the app"
    );
}

#[test]
fn commands_after_the_hub_go_straight_to_it() {
    let session = FakeInstance::new();
    let (_claim, relay) = running(&session);
    let boot = Rig::new().load();
    let wakes = watch(&boot.hub);
    relay.attach(&boot.hub);
    assert!(later(&session, Command::Show));
    assert_eq!(*wakes.lock().unwrap(), [Wake::Show]);
    assert!(
        boot.requests.try_recv().is_err(),
        "nothing asked of the app"
    );
}

#[test]
fn a_second_copys_quit_stops_the_running_app() {
    let session = FakeInstance::new();
    let (_claim, relay) = running(&session);
    let mut boot = Rig::new().load();
    boot.start().unwrap();
    let wakes = watch(&boot.hub);
    relay.attach(&boot.hub);
    let hub = Arc::clone(&boot.hub);
    let thread = drain::spawn(boot.kernel, Arc::clone(&hub), boot.requests).unwrap();
    assert!(later(&session, Command::Quit));
    thread.join().unwrap();
    assert_eq!(wakes.lock().unwrap().last(), Some(&Wake::End));
    assert!(!hub.quit(), "the queue ended");
}
