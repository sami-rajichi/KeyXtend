//! The Windows selection watch over `kx_platform::selection`'s run: a thread with COM in the multithreaded apartment,
//! a UI Automation client whose calls end at `read_ms`, a focus handler, and a selection handler on the open box last
//! looked at. That handler moves on this thread after each look in time, never in a callback, and only our own
//! handlers are added and removed, through `uia`'s locked helpers.

use std::cell::RefCell;
use std::process;
use std::sync::Arc;
use std::time::{Duration, Instant};

use kx_module_api::Redacted;
use kx_module_api::ports::{SelConfig, SelError, SelRun, SelSpot, SelectionWatch};
use kx_platform::deadline;
use kx_platform::selection::{Inbox, Ours, SelProbe, Shared, Tell, late, serve, spawn};
use windows::Win32::UI::Accessibility::{
    IUIAutomation, IUIAutomationElement, IUIAutomationEventHandler, TreeScope_Element,
    UIA_Text_TextSelectionChangedEventId,
};

use super::follow::{Heard, Hook, Seen};
use super::hear::handlers;
use super::uia::{read, sel, spot, still};
use crate::clip::open::holds;
use crate::com::Com;
use crate::field::{box_id, focused};
use crate::uia::{add_event, add_focus, client, pid, remove_event, remove_focus};

/// The focused box's selection over UI Automation, for the app to register under `SelectionWatchKey`.
#[derive(Clone, Copy, Debug, Default)]
pub struct WinSelectionWatch;

impl SelectionWatch for WinSelectionWatch {
    fn watch(&self, cfg: SelConfig, tell: Tell) -> Result<Box<dyn SelRun>, SelError> {
        spawn(cfg, tell, work)
    }
}

/// Whose an element is: ours when it shows in process `me`; the handlers ask it on any thread.
#[derive(Clone, Copy, Debug)]
pub(super) struct Mine {
    me: u32,
}

impl Mine {
    /// Ours: this process.
    pub(super) fn new() -> Self {
        Self { me: process::id() }
    }
}

impl Ours for Mine {
    type El = IUIAutomationElement;

    fn ours(&self, el: &IUIAutomationElement) -> bool {
        pid(el) == self.me
    }
}

/// The watch thread's probe: the client, and the selection handler with where it sits.
struct WinSel {
    auto: IUIAutomation,
    mine: Mine,
    hear: IUIAutomationEventHandler,
    heard: RefCell<Heard<IUIAutomationElement>>,
}

/// The selection handler added on one element alone, not on the tree below it, and removed from it.
impl Hook for WinSel {
    type El = IUIAutomationElement;

    fn add(&self, el: &IUIAutomationElement) -> bool {
        let event = UIA_Text_TextSelectionChangedEventId;
        let added = add_event(&self.auto, event, el, TreeScope_Element, &self.hear);
        if let Err(e) = &added {
            tracing::debug!(code = %e.code(), "a box refused the selection handler");
        }
        added.is_ok()
    }

    fn remove(&self, el: &IUIAutomationElement) {
        let event = UIA_Text_TextSelectionChangedEventId;
        if let Err(e) = remove_event(&self.auto, event, el, &self.hear) {
            tracing::debug!(code = %e.code(), "the selection handler was not removed from a box");
        }
    }
}

impl Ours for WinSel {
    type El = IUIAutomationElement;

    fn ours(&self, el: &IUIAutomationElement) -> bool {
        self.mine.ours(el)
    }
}

impl SelProbe for WinSel {
    fn focused(&self, _: Instant) -> Result<IUIAutomationElement, SelError> {
        focused(&self.auto).map_err(sel)
    }

    fn spot(&self, el: &IUIAutomationElement, by: Instant) -> Result<SelSpot, SelError> {
        late(by)?;
        let id = box_id(el);
        let spot = spot(el, id, self.mine, by);
        let seen = Seen::of(&spot, late(by).is_err());
        self.heard.borrow_mut().look(self, el, id, seen);
        spot
    }

    fn still(&self, el: &IUIAutomationElement, box_id: u64, by: Instant) -> bool {
        still(el, box_id, by)
    }

    fn read(
        &self,
        el: &IUIAutomationElement,
        max_chars: u16,
        by: Instant,
    ) -> Result<Redacted<String>, SelError> {
        read(el, max_chars, by)
    }

    fn clip_holds(&self) -> bool {
        holds()
    }
}

/// The watch thread's body: the client and the focus handler, a first change for the box in front, then each job in
/// turn until the run drops; our own two handlers go at the end, from this thread, and no other.
fn work(inbox: Inbox, shared: &Arc<Shared>, cfg: SelConfig) {
    let Ok(_com) = Com::start() else {
        return shared.fail("COM did not start");
    };
    let Some(auto) = client(cfg.read_ms) else {
        return shared.fail("UI Automation did not start");
    };
    let (focus, hear) = handlers(shared);
    if let Err(e) = add_focus(&auto, &focus) {
        tracing::warn!(code = %e.code(), "UI Automation refused the selection watch's focus handler");
        return shared.fail("the focus handler was refused");
    }
    let probe = WinSel {
        auto,
        mine: Mine::new(),
        hear,
        heard: RefCell::default(),
    };
    let first = probe.focused(deadline(Duration::from_millis(u64::from(cfg.read_ms))));
    if let Ok(el) = first {
        shared.moved(&probe, &el);
    }
    for job in inbox {
        serve(&probe, job);
    }
    probe.heard.borrow_mut().leave(&probe);
    if let Err(e) = remove_focus(&probe.auto, &focus) {
        tracing::warn!(code = %e.code(), "the selection watch's focus handler was not removed");
    }
}
