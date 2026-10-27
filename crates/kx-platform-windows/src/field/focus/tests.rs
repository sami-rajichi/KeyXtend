//! The focus handler: the interfaces it answers, its last reference freeing it, and calls that tell nothing for no
//! element or for our own.

use super::*;
use crate::field::testedit::{CONFIG, TestEdit, started};
use crate::lock;
use crate::scroll::testwin::{WAIT, one_client};
use kx_module_api::ports::{FieldChange, FieldKind, FieldRun};
use kx_platform::field::spawn;
use std::sync::Mutex;
use std::sync::mpsc::{self, Receiver};
use windows::Win32::Foundation::E_NOINTERFACE;
use windows::core::{IUnknown, Interface};

/// What a test's run heard.
type Heard = Arc<Mutex<Vec<FieldChange>>>;

/// A run with an idle thread, the shared box it got, and what its `tell` heard.
fn shared() -> (Box<dyn FieldRun>, Arc<Shared>, Heard) {
    let heard = Heard::default();
    let log = Arc::clone(&heard);
    let (give, got) = mpsc::channel();
    let body = move |inbox: &Receiver<_>, shared: &Arc<Shared>, _| {
        let _ = give.send(Arc::clone(shared));
        for _job in inbox {}
    };
    let tell = Box::new(move |change| lock(&log).push(change));
    let run = spawn(CONFIG, tell, body).unwrap();
    (run, got.recv_timeout(WAIT).unwrap(), heard)
}

#[test]
fn it_answers_iunknown_and_its_own_interface_and_refuses_others() {
    let (_run, shared, _) = shared();
    let focus = handler(shared);
    assert!(focus.cast::<IUnknown>().is_ok());
    assert!(
        focus
            .cast::<IUIAutomationFocusChangedEventHandler>()
            .is_ok()
    );
    let other = focus.cast::<IUIAutomationElement>().err().map(|e| e.code());
    assert_eq!(other, Some(E_NOINTERFACE));
}

#[test]
fn the_last_reference_frees_it_and_its_hold_on_the_box() {
    let (_run, shared, _) = shared();
    let before = Arc::strong_count(&shared);
    let focus = handler(Arc::clone(&shared));
    let more = focus.clone();
    let base = focus.cast::<IUnknown>().unwrap();
    assert_eq!(
        Arc::strong_count(&shared),
        before + 1,
        "the handler holds the box"
    );
    drop((focus, more));
    assert_eq!(
        Arc::strong_count(&shared),
        before + 1,
        "one reference is left"
    );
    drop(base);
    assert_eq!(Arc::strong_count(&shared), before);
}

#[test]
#[allow(unsafe_code, reason = "Calls the handler as UI Automation does.")]
fn no_element_or_one_of_our_own_tells_nothing() {
    let _one = one_client();
    let (_com, auto) = started();
    let (run, shared, heard) = shared();
    let focus = handler(shared);
    let own = TestEdit::start("ours", 0);
    // SAFETY: plain calls on our live handler, with no element and then a live one.
    unsafe {
        assert!(focus.HandleFocusChangedEvent(None).is_ok());
        assert!(focus.HandleFocusChangedEvent(&own.element(&auto)).is_ok());
    }
    assert!(lock(&heard).is_empty());
    assert_eq!(run.kind(), FieldKind::None);
}
