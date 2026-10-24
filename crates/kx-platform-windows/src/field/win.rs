//! The Windows field watch: a thread with COM in the multithreaded apartment, a UI Automation client whose calls end at
//! `read_ms`, and a focus handler that tells each box's kind. Focus in our own process keeps the last box.

use std::process;
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::time::Instant;

use kx_module_api::ports::{FieldChange, FieldConfig, FieldError, FieldRun, FieldWatch};
use windows::Win32::UI::Accessibility::{IUIAutomation, IUIAutomationElement};

use super::focus::handler;
use super::run::{Job, Shared, Tell, spawn};
use super::uia::{before_cursor, box_id, ends_with, focused, kind_of, late};
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

/// The watch thread's body: the client and the focus handler, then each read in turn until the run drops.
#[allow(
    unsafe_code,
    reason = "Adds and removes the UI Automation focus handler."
)]
fn work(inbox: &Receiver<Job>, shared: &Arc<Shared>, read_ms: u32) {
    let Ok(_com) = Com::start() else {
        return shared.fail("COM did not start");
    };
    let Some(auto) = client(read_ms) else {
        return shared.fail("UI Automation did not start");
    };
    let focus = handler(Arc::clone(shared));
    // SAFETY: registers our live handler on our own client.
    if let Err(e) = unsafe { auto.AddFocusChangedEventHandler(None, &focus) } {
        tracing::warn!(code = %e.code(), "UI Automation refused the field watch's focus handler");
        return shared.fail("the focus handler was refused");
    }
    let start = focused(&auto).ok();
    if let Some(change) = start.and_then(|el| change_of(&el, process::id())) {
        shared.first(change);
    }
    for job in inbox {
        serve(&auto, job);
    }
    // SAFETY: removes the handler added above, from the thread that added it.
    if let Err(e) = unsafe { auto.RemoveFocusChangedEventHandler(&focus) } {
        tracing::warn!(error = %e, "the field watch's focus handler was not removed");
    }
}

/// Answers `job` from the element with focus; one past its time is answered `TimedOut` unstarted.
fn serve(auto: &IUIAutomation, job: Job) {
    match job {
        Job::Before {
            max_words,
            by,
            reply,
        } => {
            let read = on_focus(auto, by, |el| before_cursor(el, max_words, by));
            let _ = reply.send(read);
        }
        Job::Ends { text, by, reply } => {
            let read = on_focus(auto, by, |el| ends_with(el, text.expose(), by));
            let _ = reply.send(read);
        }
    }
}

/// `read` on the element with focus; `None` when focus is in our own process.
fn on_focus<T>(
    auto: &IUIAutomation,
    by: Instant,
    read: impl FnOnce(&IUIAutomationElement) -> Result<Option<T>, FieldError>,
) -> Result<Option<T>, FieldError> {
    late(by)?;
    let el = focused(auto)?;
    if pid(&el) == process::id() {
        return Ok(None);
    }
    read(&el)
}

/// The kind and id of box `el`; `None` when it shows in process `me`, whose focus keeps the last box.
pub(super) fn change_of(el: &IUIAutomationElement, me: u32) -> Option<FieldChange> {
    (pid(el) != me).then(|| FieldChange {
        kind: kind_of(el),
        box_id: box_id(el),
    })
}
