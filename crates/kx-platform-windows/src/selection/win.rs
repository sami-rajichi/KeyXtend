//! The Windows selection watch over `kx_platform::selection`'s run: a thread with COM in the multithreaded apartment,
//! a UI Automation client whose calls end at `read_ms`, a focus handler, and a selection handler on the open box last
//! looked at. That handler moves on this thread once each look in time has answered, never in a callback and never off
//! a hung app's box, and only our own handlers are added and removed, through `uia`'s locked helpers.

use std::cell::RefCell;
use std::process;
use std::sync::Arc;
use std::time::{Duration, Instant};

use kx_module_api::Redacted;
use kx_module_api::ports::{SelConfig, SelError, SelRun, SelSpot, SelectionWatch};
use kx_platform::deadline;
use kx_platform::selection::{Inbox, Ours, SelProbe, Shared, Tell, late, serve, spawn};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Accessibility::{
    IUIAutomation, IUIAutomationElement, IUIAutomationEventHandler, TreeScope_Element,
    UIA_Text_TextSelectionChangedEventId,
};
use windows::Win32::UI::WindowsAndMessaging::IsHungAppWindow;

use super::follow::{Heard, Hook, Seen};
use super::hear::handlers;
use super::uia::{read, sel, spot, still, text_of};
use crate::clip::open::holds;
use crate::com::Com;
use crate::field::{box_id, focused, top};
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

    /// Ours: process `me`, which a test sets to another than this one.
    #[cfg(test)]
    pub(super) fn of(me: u32) -> Self {
        Self { me }
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
    heard: RefCell<Heard<Added>>,
}

/// The element a job looks at, and whether the walk up reached it from the focused one.
struct Focus {
    el: IUIAutomationElement,
    climbed: bool,
}

/// Where the selection handler goes: the element, and its top-level window, read before the add.
struct Added {
    el: IUIAutomationElement,
    top: HWND,
}

/// The selection handler added on one element alone, not on the tree below it, and removed from it; `Heard` sends
/// neither call to an app Windows reports hung, but for the stop's removal.
impl Hook for WinSel {
    type El = IUIAutomationElement;
    type On = Added;

    fn at(&self, el: &IUIAutomationElement) -> Added {
        Added {
            el: el.clone(),
            top: top(el),
        }
    }

    fn add(&self, on: &Added) -> bool {
        let event = UIA_Text_TextSelectionChangedEventId;
        let added = add_event(&self.auto, event, &on.el, TreeScope_Element, &self.hear);
        if let Err(e) = &added {
            tracing::debug!(code = %e.code(), "a box refused the selection handler");
        }
        added.is_ok()
    }

    fn remove(&self, on: &Added) {
        let event = UIA_Text_TextSelectionChangedEventId;
        if let Err(e) = remove_event(&self.auto, event, &on.el, &self.hear) {
            tracing::debug!(code = %e.code(), "the selection handler was not removed from a box");
        }
    }

    #[allow(unsafe_code, reason = "A plain window query.")]
    fn hung(&self, on: &Added) -> bool {
        // SAFETY: a plain query that reads Windows' own record, sending the window nothing.
        unsafe { IsHungAppWindow(on.top) }.as_bool()
    }
}

impl Ours for WinSel {
    type El = Focus;

    fn ours(&self, at: &Focus) -> bool {
        self.mine.ours(&at.el)
    }
}

/// Every job looks at the element that holds the focused box's text, so a spot, its box id, `still` and a read all
/// name that same element.
impl SelProbe for WinSel {
    fn focused(&self, by: Instant) -> Result<Focus, SelError> {
        let el = focused(&self.auto).map_err(sel)?;
        let (el, climbed) = text_of(el, &self.auto, self.mine, by);
        Ok(Focus { el, climbed })
    }

    fn spot(&self, at: &Focus, by: Instant) -> Result<SelSpot, SelError> {
        late(by)?;
        spot(&at.el, box_id(&at.el), self.mine, by, at.climbed)
    }

    fn still(&self, at: &Focus, box_id: u64, by: Instant) -> bool {
        still(&at.el, box_id, by)
    }

    fn read(&self, at: &Focus, max_chars: u16, by: Instant) -> Result<Redacted<String>, SelError> {
        read(&at.el, max_chars, by)
    }

    fn clip_holds(&self) -> bool {
        holds()
    }

    /// The selection handler moves only once the look answered, so its adds and removals never count against it.
    fn looked(&self, at: &Focus, spot: &SelSpot, late: bool) {
        let seen = Seen::of(&Ok(*spot), late);
        self.heard
            .borrow_mut()
            .look(self, &at.el, spot.box_id, seen);
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
