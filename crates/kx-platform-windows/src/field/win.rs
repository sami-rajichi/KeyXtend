//! The Windows field watch: a thread with COM in the multithreaded apartment, a UI Automation client whose calls end at
//! `read_ms`, and a focus handler. The run and its rules are `kx_platform::field`'s; this side only reads elements.

use std::process;
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::time::Instant;

use kx_module_api::ports::{FieldChange, FieldConfig, FieldError, FieldRead, FieldRun, FieldWatch};
use kx_platform::field::{Job, Probe, Reader, Shared, Tell, serve, spawn};
use windows::Win32::UI::Accessibility::{IUIAutomation, IUIAutomationElement};

use super::focus::handler;
use super::uia::{before_cursor, box_id, ends_with, focused, kind_of};
use crate::com::Com;
use crate::uia::{client, pid};

/// The focused-box watch over UI Automation, for the app to register under `FieldWatchKey`.
#[derive(Clone, Copy, Debug, Default)]
pub struct WinFieldWatch;

impl FieldWatch for WinFieldWatch {
    fn watch(&self, cfg: FieldConfig, tell: Tell) -> Result<Box<dyn FieldRun>, FieldError> {
        spawn(cfg, tell, work)
    }
}

/// Tells whether an element shows in process `me`, and what box it is.
#[derive(Clone, Copy, Debug)]
pub(super) struct WinProbe {
    pub(super) me: u32,
}

impl WinProbe {
    /// A probe for our own process.
    pub(super) fn new() -> Self {
        Self { me: process::id() }
    }
}

impl Probe for WinProbe {
    type El = IUIAutomationElement;

    fn ours(&self, el: &IUIAutomationElement) -> bool {
        pid(el) == self.me
    }

    fn change(&self, el: &IUIAutomationElement) -> FieldChange {
        FieldChange {
            kind: kind_of(el),
            box_id: box_id(el),
        }
    }
}

/// Reads the focused box over UI Automation, never more than `chars` characters at once.
struct WinReader {
    auto: IUIAutomation,
    probe: WinProbe,
    chars: usize,
}

impl Probe for WinReader {
    type El = IUIAutomationElement;

    fn ours(&self, el: &IUIAutomationElement) -> bool {
        self.probe.ours(el)
    }

    fn change(&self, el: &IUIAutomationElement) -> FieldChange {
        self.probe.change(el)
    }
}

impl Reader for WinReader {
    fn focused(&self) -> Result<IUIAutomationElement, FieldError> {
        focused(&self.auto)
    }

    fn before_cursor(
        &self,
        el: &IUIAutomationElement,
        max_words: u8,
        by: Instant,
    ) -> Result<FieldRead, FieldError> {
        before_cursor(el, max_words, by, self.chars)
    }

    fn ends_with(
        &self,
        el: &IUIAutomationElement,
        text: &str,
        by: Instant,
    ) -> Result<Option<bool>, FieldError> {
        ends_with(el, text, by, self.chars)
    }
}

/// The watch thread's body: the client and the focus handler, then each read in turn until the run drops.
#[allow(
    unsafe_code,
    reason = "Adds and removes the UI Automation focus handler."
)]
fn work(inbox: &Receiver<Job>, shared: &Arc<Shared>, cfg: FieldConfig) {
    let Ok(_com) = Com::start() else {
        return shared.fail("COM did not start");
    };
    let Some(auto) = client(cfg.read_ms) else {
        return shared.fail("UI Automation did not start");
    };
    let focus = handler(Arc::clone(shared));
    // SAFETY: registers our live handler on our own client.
    if let Err(e) = unsafe { auto.AddFocusChangedEventHandler(None, &focus) } {
        tracing::warn!(code = %e.code(), "UI Automation refused the field watch's focus handler");
        return shared.fail("the focus handler was refused");
    }
    let chars = usize::from(cfg.read_chars);
    let reader = WinReader {
        auto,
        probe: WinProbe::new(),
        chars,
    };
    if let Ok(el) = reader.focused() {
        shared.first(&reader, &el);
    }
    for job in inbox {
        serve(&reader, job);
    }
    // SAFETY: removes the handler added above, from the thread that added it.
    if let Err(e) = unsafe { reader.auto.RemoveFocusChangedEventHandler(&focus) } {
        tracing::warn!(error = %e, "the field watch's focus handler was not removed");
    }
}
