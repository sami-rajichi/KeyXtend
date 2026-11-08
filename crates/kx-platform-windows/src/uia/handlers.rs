//! Our watches' UI Automation handlers, added and removed only here, one at a time in the process under one lock, as
//! Microsoft asks of a client that uses several threads; `uia/tests/scan.rs` keeps every other file from doing it.

use std::sync::Mutex;

use windows::Win32::UI::Accessibility::{
    IUIAutomation, IUIAutomationElement, IUIAutomationEventHandler,
    IUIAutomationFocusChangedEventHandler, TreeScope, UIA_EVENT_ID,
};

/// Held around every handler added or removed here.
static HANDLERS: Mutex<()> = Mutex::new(());

/// Runs `f` while no other thread of ours adds or removes a handler.
fn alone<T>(f: impl FnOnce() -> T) -> T {
    let _one = crate::lock(&HANDLERS);
    f()
}

/// Adds focus handler `focus` on client `auto`; the thread that adds it removes it.
///
/// # Errors
/// UI Automation's own, when it refuses the handler.
#[allow(unsafe_code, reason = "Adds a UI Automation focus handler.")]
pub(crate) fn add_focus(
    auto: &IUIAutomation,
    focus: &IUIAutomationFocusChangedEventHandler,
) -> windows::core::Result<()> {
    // SAFETY: a plain call on a live client with a live handler.
    alone(|| unsafe { auto.AddFocusChangedEventHandler(None, focus) })
}

/// Removes focus handler `focus` from client `auto`, from the thread that added it.
///
/// # Errors
/// UI Automation's own, when it does not let the handler go.
#[allow(unsafe_code, reason = "Removes a UI Automation focus handler.")]
pub(crate) fn remove_focus(
    auto: &IUIAutomation,
    focus: &IUIAutomationFocusChangedEventHandler,
) -> windows::core::Result<()> {
    // SAFETY: a plain call on a live client with a live handler.
    alone(|| unsafe { auto.RemoveFocusChangedEventHandler(focus) })
}

/// Adds handler `hear` for `event` on element `el` within `scope`; the thread that adds it removes it.
///
/// # Errors
/// UI Automation's own, when the box refuses the handler.
#[allow(unsafe_code, reason = "Adds a UI Automation event handler.")]
pub(crate) fn add_event(
    auto: &IUIAutomation,
    event: UIA_EVENT_ID,
    el: &IUIAutomationElement,
    scope: TreeScope,
    hear: &IUIAutomationEventHandler,
) -> windows::core::Result<()> {
    // SAFETY: a plain call on a live client with a live element and handler.
    alone(|| unsafe { auto.AddAutomationEventHandler(event, el, scope, None, hear) })
}

/// Removes handler `hear` for `event` from element `el`, from the thread that added it.
///
/// # Errors
/// UI Automation's own, when it does not let the handler go.
#[allow(unsafe_code, reason = "Removes a UI Automation event handler.")]
pub(crate) fn remove_event(
    auto: &IUIAutomation,
    event: UIA_EVENT_ID,
    el: &IUIAutomationElement,
    hear: &IUIAutomationEventHandler,
) -> windows::core::Result<()> {
    // SAFETY: a plain call on a live client with a live element and handler.
    alone(|| unsafe { auto.RemoveAutomationEventHandler(event, el, hear) })
}
