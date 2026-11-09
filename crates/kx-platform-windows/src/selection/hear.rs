//! The change handlers, made by windows-rs's `implement` macro: UI Automation calls them on threads of its own, for a
//! focus change anywhere and a selection change in the box the watch's thread aims them at; each call tells the desk.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

use kx_platform::selection::Shared;
use windows::Win32::UI::Accessibility::{
    IUIAutomationElement, IUIAutomationEventHandler, IUIAutomationEventHandler_Impl,
    IUIAutomationFocusChangedEventHandler, IUIAutomationFocusChangedEventHandler_Impl,
    UIA_EVENT_ID,
};
use windows::core::Ref;

use super::win::Mine;
use glue::{Hear, Hear_Impl};

/// The handlers' object alone, so the allow covers only the COM glue the macro writes for it.
mod glue {
    #![allow(
        unsafe_code,
        clippy::inline_always,
        clippy::ref_as_ptr,
        reason = "The `implement` macro writes the COM glue as unsafe code in its own style."
    )]

    use super::{
        Arc, IUIAutomationEventHandler, IUIAutomationFocusChangedEventHandler, Mine, Shared,
    };
    use windows::core::implement;

    /// The handlers' object: the desk it tells, and whose element changed.
    #[implement(IUIAutomationFocusChangedEventHandler, IUIAutomationEventHandler)]
    pub(super) struct Hear {
        pub(super) shared: Arc<Shared>,
        pub(super) mine: Mine,
    }
}

/// The focus handler and the selection handler, each telling `shared`.
pub(super) fn handlers(
    shared: &Arc<Shared>,
) -> (
    IUIAutomationFocusChangedEventHandler,
    IUIAutomationEventHandler,
) {
    let hear = || Hear {
        shared: Arc::clone(shared),
        mine: Mine::new(),
    };
    (hear().into(), hear().into())
}

impl Hear {
    /// Tells the desk about `sender`; a panic never crosses COM.
    fn tell(&self, sender: &Ref<IUIAutomationElement>) {
        if let Some(el) = sender.as_ref() {
            let told = catch_unwind(AssertUnwindSafe(|| self.shared.moved(&self.mine, el)));
            if told.is_err() {
                tracing::warn!("the selection watch's handler panicked");
            }
        }
    }
}

impl IUIAutomationFocusChangedEventHandler_Impl for Hear_Impl {
    /// Tells the desk about the element that took the focus.
    fn HandleFocusChangedEvent(
        &self,
        sender: Ref<IUIAutomationElement>,
    ) -> windows::core::Result<()> {
        self.tell(&sender);
        Ok(())
    }
}

impl IUIAutomationEventHandler_Impl for Hear_Impl {
    /// Tells the desk about the box whose selection changed, the one event this handler is added for.
    fn HandleAutomationEvent(
        &self,
        sender: Ref<IUIAutomationElement>,
        _: UIA_EVENT_ID,
    ) -> windows::core::Result<()> {
        self.tell(&sender);
        Ok(())
    }
}
