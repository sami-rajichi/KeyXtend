//! The focus handler, made by windows-rs's `implement` macro: UI Automation calls it on threads of its own, and each
//! call tells the watch's shared box.
#![allow(
    unsafe_code,
    clippy::inline_always,
    clippy::ref_as_ptr,
    reason = "The `implement` macro writes the COM glue as unsafe code in its own style."
)]

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

use kx_platform::field::Shared;
use windows::Win32::UI::Accessibility::{
    IUIAutomationElement, IUIAutomationFocusChangedEventHandler,
    IUIAutomationFocusChangedEventHandler_Impl,
};
use windows::core::{Ref, implement};

use super::win::WinProbe;

/// The handler's object: the shared box it tells, and the probe that says whose element took the focus.
#[implement(IUIAutomationFocusChangedEventHandler)]
struct Focus {
    shared: Arc<Shared>,
    probe: WinProbe,
}

/// A focus handler that tells `shared`.
pub(super) fn handler(shared: Arc<Shared>) -> IUIAutomationFocusChangedEventHandler {
    let probe = WinProbe::new();
    Focus { shared, probe }.into()
}

impl IUIAutomationFocusChangedEventHandler_Impl for Focus_Impl {
    /// Tells the shared box about the element that took the focus; a panic in the module's callback never crosses COM.
    fn HandleFocusChangedEvent(
        &self,
        sender: Ref<IUIAutomationElement>,
    ) -> windows::core::Result<()> {
        if let Some(el) = sender.as_ref() {
            let tell = || self.shared.moved(&self.probe, el);
            let told = catch_unwind(AssertUnwindSafe(tell));
            if told.is_err() {
                tracing::warn!("the field watch's callback panicked");
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
