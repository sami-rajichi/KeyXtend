//! The focus handler, a small COM object of our own: UI Automation calls it on threads of its own, and each call tells
//! the watch's shared box. It answers only `IUnknown` and its handler interface.
#![allow(
    unsafe_code,
    reason = "A COM object's methods are unsafe calls through its method table."
)]

use std::ffi::c_void;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::process;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering, fence};

use windows::Win32::Foundation::{E_NOINTERFACE, E_POINTER, S_OK};
use windows::Win32::UI::Accessibility::{
    IUIAutomationElement, IUIAutomationFocusChangedEventHandler,
    IUIAutomationFocusChangedEventHandler_Vtbl,
};
use windows::core::{GUID, HRESULT, IUnknown, IUnknown_Vtbl, Interface};

use super::run::Shared;
use super::win::seen;

/// The handler's methods, in the interface's order.
static METHODS: IUIAutomationFocusChangedEventHandler_Vtbl =
    IUIAutomationFocusChangedEventHandler_Vtbl {
        base__: IUnknown_Vtbl {
            QueryInterface: query,
            AddRef: add_ref,
            Release: release,
        },
        HandleFocusChangedEvent: handle,
    };

/// The object behind the interface: its method table first, as COM lays it out, then its count and the shared box.
#[repr(C)]
struct Focus {
    methods: &'static IUIAutomationFocusChangedEventHandler_Vtbl,
    refs: AtomicU32,
    shared: Arc<Shared>,
}

/// A focus handler that tells `shared`.
pub(super) fn handler(shared: Arc<Shared>) -> IUIAutomationFocusChangedEventHandler {
    let focus = Box::new(Focus {
        methods: &METHODS,
        refs: AtomicU32::new(1),
        shared,
    });
    // SAFETY: a live object laid out as COM wants, holding the one reference the interface takes over.
    unsafe { IUIAutomationFocusChangedEventHandler::from_raw(Box::into_raw(focus).cast()) }
}

/// `this` as our object.
///
/// # Safety
/// `this` is a live `Focus` that `handler` made.
unsafe fn object<'a>(this: *mut c_void) -> &'a Focus {
    // SAFETY: the caller's promise.
    unsafe { &*this.cast::<Focus>() }
}

/// Hands out this object for `IUnknown` and the handler interface, refusing any other.
unsafe extern "system" fn query(
    this: *mut c_void,
    iid: *const GUID,
    out: *mut *mut c_void,
) -> HRESULT {
    if out.is_null() {
        return E_POINTER;
    }
    // SAFETY: COM passes a live IID or null, and `out` is a live slot, checked above.
    unsafe {
        let ours = [IUnknown::IID, IUIAutomationFocusChangedEventHandler::IID];
        if iid.is_null() || !ours.contains(&*iid) {
            *out = std::ptr::null_mut();
            return E_NOINTERFACE;
        }
        add_ref(this);
        *out = this;
    }
    S_OK
}

/// Counts one more reference.
unsafe extern "system" fn add_ref(this: *mut c_void) -> u32 {
    // SAFETY: COM calls it on a live object.
    let before = unsafe { object(this) }.refs.fetch_add(1, Ordering::Relaxed);
    before.wrapping_add(1)
}

/// Counts one reference less, and frees the object with the last.
unsafe extern "system" fn release(this: *mut c_void) -> u32 {
    // SAFETY: COM calls it on a live object, once per reference it holds.
    let before = unsafe { object(this) }.refs.fetch_sub(1, Ordering::Release);
    let left = before.wrapping_sub(1);
    if left == 0 {
        fence(Ordering::Acquire);
        // SAFETY: the last reference is gone, so the box `handler` made is ours to free.
        drop(unsafe { Box::from_raw(this.cast::<Focus>()) });
    }
    left
}

/// Tells the shared box about the element that took the focus; a panic in the module's callback never crosses COM.
unsafe extern "system" fn handle(this: *mut c_void, sender: *mut c_void) -> HRESULT {
    // SAFETY: COM calls it on a live object, with an element it lends for the call, or null.
    let (focus, el) = unsafe {
        (
            object(this),
            IUIAutomationElement::from_raw_borrowed(&sender),
        )
    };
    if let Some(el) = el {
        let told = catch_unwind(AssertUnwindSafe(|| seen(&focus.shared, el, process::id())));
        if told.is_err() {
            tracing::warn!("the field watch's callback panicked");
        }
    }
    S_OK
}

#[cfg(test)]
mod tests;
