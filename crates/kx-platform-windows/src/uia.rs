//! A UI Automation client, moved from the P1 spike (ADR-0014): elements by name, text documents, the focused element and
//! patterns found up the tree. The gates and the field watch use it; it never logs an element's text.

mod act;
mod read;
mod rects;

use thiserror::Error;
use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance};
use windows::Win32::System::Variant::VARIANT;
use windows::Win32::UI::Accessibility::{
    CUIAutomation, CUIAutomation8, IUIAutomation, IUIAutomation2, IUIAutomationCondition,
    IUIAutomationElement, IUIAutomationScrollPattern, IUIAutomationTextPattern,
    TreeScope_Descendants, UIA_ControlTypePropertyId, UIA_DocumentControlTypeId,
    UIA_IsTextPatternAvailablePropertyId, UIA_ListItemControlTypeId, UIA_NamePropertyId,
    UIA_PATTERN_ID, UIA_PROPERTY_ID, UIA_ScrollPatternId, UIA_TextPatternId,
};
use windows::core::Interface;

pub use act::{caret, focus, invoke, scroll};
pub use read::{
    Sel, boxes, is_password, kind, lines, name, password_flag, percent, pid, rect, selected,
    selection, value,
};
pub use rects::rects_of;

pub(crate) use read::{class_of, style_of};
pub(crate) use rects::vector;

use crate::PlatformError;
use crate::com::Com;

/// Most ancestors walked when looking for a pattern above an element.
const MAX_DEPTH: usize = 32;

/// Why a UI Automation call failed; it names the call, never an element's text.
#[derive(Debug, Error)]
pub enum UiaError {
    /// COM did not start on this thread.
    #[error(transparent)]
    Com(#[from] PlatformError),
    /// A call failed: what it was, and Windows' error.
    #[error("UI Automation {what}: {source}")]
    Call {
        /// The call that failed.
        what: &'static str,
        /// Windows' error.
        source: windows::core::Error,
    },
    /// A call gave nothing to work with.
    #[error("UI Automation {0}: none")]
    Empty(&'static str),
    /// A text was longer than the read cap, so it was not read.
    #[error("UI Automation {0}: longer than the read cap")]
    TooLong(&'static str),
}

/// Names a failed call `what`, for `map_err`.
pub(crate) fn call(what: &'static str) -> impl Fn(windows::core::Error) -> UiaError {
    move |source| UiaError::Call { what, source }
}

/// `READ_CAP` plus `more`, as UI Automation takes a length.
pub(crate) fn cap(more: usize) -> i32 {
    i32::try_from(crate::field::READ_CAP + more).unwrap_or(i32::MAX)
}

/// The system's client, with its connection and call timeouts at `read_ms`; none when it does not start.
#[allow(unsafe_code, reason = "Makes and sets up the UI Automation client.")]
pub(crate) fn client(read_ms: u32) -> Option<IUIAutomation> {
    // SAFETY: makes the system's UI Automation client on this thread, where COM runs.
    let auto: IUIAutomation =
        unsafe { CoCreateInstance(&CUIAutomation8, None, CLSCTX_INPROC_SERVER) }.ok()?;
    if let Ok(timed) = auto.cast::<IUIAutomation2>() {
        // SAFETY: plain settings on our own client; a refusal keeps Windows' defaults.
        unsafe {
            let _ = timed.SetConnectionTimeout(read_ms);
            let _ = timed.SetTransactionTimeout(read_ms);
        }
    }
    Some(auto)
}

/// A UI Automation client.
pub struct Uia(IUIAutomation);

impl Uia {
    /// Starts COM on this thread for good, since elements may outlive the client, and creates the client.
    ///
    /// # Errors
    /// `UiaError` when COM or the client does not start.
    #[allow(unsafe_code, reason = "Makes the UI Automation client.")]
    pub fn new() -> Result<Self, UiaError> {
        Com::start()?.keep();
        // SAFETY: creates the system's UIA client object on this thread, where COM runs.
        let auto = unsafe { CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER) };
        Ok(Self(auto.map_err(call("client"))?))
    }

    /// A condition that holds when every property has its value; one property needs no AND.
    #[allow(unsafe_code, reason = "Builds UI Automation conditions.")]
    fn all(
        &self,
        props: Vec<(UIA_PROPERTY_ID, VARIANT)>,
    ) -> Result<IUIAutomationCondition, UiaError> {
        let mut conds = Vec::new();
        for (id, value) in props {
            // SAFETY: a plain call with a live VARIANT, which the condition copies.
            let c = unsafe { self.0.CreatePropertyCondition(id, &value) };
            conds.push(c.map_err(call("condition"))?);
        }
        if conds.len() == 1 {
            return conds.pop().ok_or(UiaError::Empty("condition"));
        }
        let conds: Vec<Option<IUIAutomationCondition>> = conds.into_iter().map(Some).collect();
        // SAFETY: a plain call over live conditions.
        unsafe { self.0.CreateAndConditionFromNativeArray(&conds) }.map_err(call("condition"))
    }

    /// The first element below `under` that fits every property.
    #[allow(unsafe_code, reason = "A UI Automation tree search.")]
    fn first(
        &self,
        under: &IUIAutomationElement,
        props: Vec<(UIA_PROPERTY_ID, VARIANT)>,
    ) -> Option<IUIAutomationElement> {
        let cond = self.all(props).ok()?;
        // SAFETY: a plain tree search on a live element.
        unsafe { under.FindFirst(TreeScope_Descendants, &cond) }.ok()
    }

    /// The element of window `hwnd`.
    ///
    /// # Errors
    /// `UiaError::Call` when the window is gone.
    #[allow(unsafe_code, reason = "A UI Automation lookup.")]
    pub fn window(&self, hwnd: HWND) -> Result<IUIAutomationElement, UiaError> {
        // SAFETY: a plain call; a dead handle gives an error.
        unsafe { self.0.ElementFromHandle(hwnd) }.map_err(call("window"))
    }

    /// The list item called `name` in window `hwnd`, such as a file in Explorer.
    #[must_use]
    pub fn item(&self, hwnd: HWND, name: &str) -> Option<IUIAutomationElement> {
        let kind = VARIANT::from(UIA_ListItemControlTypeId.0);
        let props = vec![
            (UIA_NamePropertyId, VARIANT::from(name)),
            (UIA_ControlTypePropertyId, kind),
        ];
        self.first(&self.window(hwnd).ok()?, props)
    }

    /// The first element called `name` in window `hwnd`, such as a web field by its label.
    #[must_use]
    pub fn named(&self, hwnd: HWND, name: &str) -> Option<IUIAutomationElement> {
        let props = vec![(UIA_NamePropertyId, VARIANT::from(name))];
        self.first(&self.window(hwnd).ok()?, props)
    }

    /// The box to press on `item`: its label called `name`, else the item's own.
    ///
    /// # Errors
    /// `UiaError::Call` when the box cannot be read.
    pub fn grip(&self, item: &IUIAutomationElement, name: &str) -> Result<RECT, UiaError> {
        let label = self.first(item, vec![(UIA_NamePropertyId, VARIANT::from(name))]);
        rect(label.as_ref().unwrap_or(item))
    }

    /// The text document in window `hwnd`: a Document with `TextPattern`, never an address bar.
    #[must_use]
    pub fn text(&self, hwnd: HWND) -> Option<IUIAutomationElement> {
        let has_text = (UIA_IsTextPatternAvailablePropertyId, VARIANT::from(true));
        let doc = (
            UIA_ControlTypePropertyId,
            VARIANT::from(UIA_DocumentControlTypeId.0),
        );
        self.first(&self.window(hwnd).ok()?, vec![has_text, doc])
    }

    /// The text of the document in window `hwnd`, at most `max` characters; `None` when it cannot be read.
    #[must_use]
    #[allow(unsafe_code, reason = "Reads a UI Automation text range.")]
    pub fn doc_text(&self, hwnd: HWND, max: i32) -> Option<String> {
        let tp = self.text_of(&self.text(hwnd)?)?;
        // SAFETY: plain COM calls on live UI Automation objects.
        unsafe { tp.DocumentRange().ok()?.GetText(max).ok() }.map(|b| b.to_string())
    }

    /// The element that has the keyboard focus, in any app.
    ///
    /// # Errors
    /// `UiaError::Call` when UI Automation cannot say.
    #[allow(unsafe_code, reason = "A UI Automation lookup.")]
    pub fn focused(&self) -> Result<IUIAutomationElement, UiaError> {
        // SAFETY: a plain call on our live client.
        unsafe { self.0.GetFocusedElement() }.map_err(call("focus"))
    }

    /// Pattern `id` of `el`, or of its nearest ancestor that has it, at most `MAX_DEPTH` up.
    #[allow(unsafe_code, reason = "Walks up the UI Automation tree.")]
    fn up<T: Interface>(&self, el: IUIAutomationElement, id: UIA_PATTERN_ID) -> Option<T> {
        // SAFETY: plain calls on live objects; each failure ends the walk.
        unsafe {
            let walker = self.0.ControlViewWalker().ok()?;
            let mut el = el;
            for _ in 0..MAX_DEPTH {
                if let Ok(p) = el.GetCurrentPatternAs(id) {
                    return Some(p);
                }
                el = walker.GetParentElement(&el).ok()?;
            }
            None
        }
    }

    /// The `TextPattern` of `el`, or of its nearest ancestor that has one.
    #[must_use]
    pub fn text_of(&self, el: &IUIAutomationElement) -> Option<IUIAutomationTextPattern> {
        self.up(el.clone(), UIA_TextPatternId)
    }

    /// The `ScrollPattern` of the element at screen point `p`, or of its nearest ancestor that has one.
    #[must_use]
    #[allow(unsafe_code, reason = "A UI Automation lookup.")]
    pub fn scroller(&self, p: POINT) -> Option<IUIAutomationScrollPattern> {
        // SAFETY: a plain call on our live client.
        let el = unsafe { self.0.ElementFromPoint(p) }.ok()?;
        self.up(el, UIA_ScrollPatternId)
    }
}

#[cfg(test)]
mod tests;
