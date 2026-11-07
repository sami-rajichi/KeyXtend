//! The change handlers with no UI Automation client: the interfaces they answer, their last references freeing them,
//! a change with no element being no error, and when the selection handler moves to another box.

use super::super::hear::handlers;
use super::super::win::same;
use kx_module_api::ports::{SelConfig, SelRun};
use kx_platform::selection::{Shared, spawn};
use std::sync::Arc;
use std::sync::mpsc;
use std::time::Duration;
use windows::Win32::Foundation::E_NOINTERFACE;
use windows::Win32::UI::Accessibility::{
    IUIAutomationElement, IUIAutomationEventHandler, IUIAutomationFocusChangedEventHandler,
    UIA_Text_TextSelectionChangedEventId,
};
use windows::core::{IUnknown, Interface};

/// A watch no test reaches the pace of.
const CONFIG: SelConfig = SelConfig {
    settle_ms: 5000,
    read_ms: 5000,
    looks_per_s: 1,
};
/// How long a test waits for the watch's thread; a bounded wait, never a sleep.
const WAIT: Duration = Duration::from_secs(5);

/// A run with an idle thread, and the desk it shares.
fn shared() -> (Box<dyn SelRun>, Arc<Shared>) {
    let (give, got) = mpsc::channel();
    let body = move |inbox, shared: &Arc<Shared>, _| {
        let _ = give.send(Arc::clone(shared));
        for _job in inbox {}
    };
    let run = spawn(CONFIG, Box::new(|| {}), body).unwrap();
    (run, got.recv_timeout(WAIT).unwrap())
}

#[test]
fn each_answers_iunknown_and_both_handler_interfaces_and_refuses_others() {
    let (_run, shared) = shared();
    let (focus, hear) = handlers(&shared);
    for base in [focus.cast::<IUnknown>(), hear.cast::<IUnknown>()] {
        let base = base.unwrap();
        assert!(base.cast::<IUIAutomationFocusChangedEventHandler>().is_ok());
        assert!(base.cast::<IUIAutomationEventHandler>().is_ok());
        let other = base.cast::<IUIAutomationElement>().err().map(|e| e.code());
        assert_eq!(other, Some(E_NOINTERFACE));
    }
}

#[test]
fn the_last_references_free_them_and_their_hold_on_the_desk() {
    let (_run, shared) = shared();
    let before = Arc::strong_count(&shared);
    let (focus, hear) = handlers(&shared);
    assert_eq!(
        Arc::strong_count(&shared),
        before + 2,
        "each holds the desk"
    );
    let kept = hear.cast::<IUnknown>().unwrap();
    drop((focus, hear));
    assert_eq!(Arc::strong_count(&shared), before + 1, "one is still held");
    drop(kept);
    assert_eq!(Arc::strong_count(&shared), before);
}

#[test]
#[allow(unsafe_code, reason = "Calls the handlers as UI Automation does.")]
fn a_change_with_no_element_is_no_error() {
    let (_run, shared) = shared();
    let (focus, hear) = handlers(&shared);
    // SAFETY: plain calls on our own live handlers, with no element.
    unsafe {
        assert!(focus.HandleFocusChangedEvent(None).is_ok());
        let id = UIA_Text_TextSelectionChangedEventId;
        assert!(hear.HandleAutomationEvent(None, id).is_ok());
    }
}

#[test]
fn the_selection_handler_stays_on_its_box_and_moves_for_any_other() {
    assert!(same(7, 7));
    assert!(!same(7, 8), "another box");
    assert!(!same(0, 0), "a box with no id is never the same");
    assert!(!same(7, 0) && !same(0, 7));
}
