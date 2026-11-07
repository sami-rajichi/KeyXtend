use super::*;
use crate::fakes;
use crate::hidden::fixture::{self, Bed, Held};
use kx_platform::ui::{Command, HiddenCopy, UiError};
use kx_ui::Wake;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::mpsc::Sender;

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

/// A session claim that signals as it drops, so a held delete goes on only once the session is free.
struct Noted(Sender<()>);

impl Claim for Noted {
    fn listen(&mut self, _on: Box<dyn Fn(Command) + Send>) -> Result<(), UiError> {
        Ok(())
    }
}

impl Drop for Noted {
    fn drop(&mut self) {
        let _ = self.0.send(());
    }
}

#[test]
fn quit_frees_the_session_before_the_app_waits_for_a_delete() {
    let bed = Bed::new();
    let mut kernel = bed.kernel();
    kernel.boot().unwrap();
    let (hub, requests) = fixture::hub();
    let copy = HiddenCopy::new(
        PathBuf::from("copy"),
        "Claude_x".into(),
        fixture::APP.into(),
    );
    let (port, freed) = Held::new(vec![copy.clone()]);
    let asked = Asked::ask(port.clone(), &kernel, &hub);
    let deletes = asked.deletes();
    let id = hub.take_notices()[0].id;
    assert!(hub.answer(id, true) && hub.quit());
    drain(kernel, &hub, &requests, Some(asked));
    assert!(
        port.fake.removed().is_empty(),
        "the kernel's thread did not wait"
    );
    release(Box::new(Noted(freed)), &deletes);
    assert_eq!(
        port.fake.removed(),
        [copy],
        "the delete went on once the session was free"
    );
}
