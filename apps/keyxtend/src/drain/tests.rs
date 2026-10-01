use super::*;
use kx_ui::Wake;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Mutex;

#[path = "../../tests/common/fakes.rs"]
mod fakes;

/// A hub over the shared fakes, with a waker that keeps every wake.
fn hub() -> (Arc<Hub>, Arc<Mutex<Vec<Wake>>>) {
    let (hub, _requests) = Hub::new(fakes::ports(fakes::tray())).unwrap();
    let wakes = Arc::new(Mutex::new(Vec::new()));
    let seen = Arc::clone(&wakes);
    hub.set_waker(Box::new(move |w| seen.lock().unwrap().push(w)));
    (hub, wakes)
}

#[test]
fn a_panic_while_stopping_still_ends_the_ui_once() {
    let (hub, wakes) = hub();
    let stopped = catch_unwind(AssertUnwindSafe(|| {
        let _end = EndUi(&hub);
        panic!("a module would not stop");
    }));
    assert!(stopped.is_err());
    assert_eq!(*wakes.lock().unwrap(), [Wake::End]);
}
