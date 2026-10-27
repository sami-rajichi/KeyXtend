//! G6's own UI Automation reads on Explorer: its parts' boxes by control type, the types at a point and above it,
//! and the keyboard focus put on its list. None of them moves the pointer.
#![cfg(windows)]

use kx_platform_windows::uia;
use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::System::Variant::VARIANT;
use windows::Win32::UI::Accessibility::{
    IUIAutomation, IUIAutomationElement, IUIAutomationScrollPattern, TreeScope_Descendants,
    UIA_CONTROLTYPE_ID, UIA_ControlTypePropertyId, UIA_ScrollPatternId,
};
use windows::core::BOOL;

/// What UI Automation shows at a point: the control types from the element there up to the root, and its class.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct At {
    /// The element's control type first, then each parent's.
    pub chain: Vec<UIA_CONTROLTYPE_ID>,
    /// True when the walk read every type up to the root.
    pub whole: bool,
    /// The element's class name.
    pub class: String,
}

/// What UI Automation shows at screen point `p`; an empty chain when it finds nothing there.
#[allow(unsafe_code, reason = "UI Automation calls on a live client.")]
pub fn chain_at(auto: &IUIAutomation, p: POINT) -> At {
    let mut at = At::default();
    // SAFETY: plain calls and property reads; the walk ends where an element has no parent, and it moves no pointer.
    unsafe {
        let (Ok(walker), Ok(mut el)) = (auto.ControlViewWalker(), auto.ElementFromPoint(p)) else {
            return at;
        };
        at.class = el
            .CurrentClassName()
            .map(|b| b.to_string())
            .unwrap_or_default();
        loop {
            let Ok(kind) = el.CurrentControlType() else {
                return at;
            };
            at.chain.push(kind);
            el = match walker.GetParentElement(&el) {
                Ok(parent) => parent,
                // The root has no parent: the call succeeds and gives none.
                Err(e) => {
                    at.whole = e.code().is_ok();
                    return at;
                }
            };
        }
    }
}

/// The screen boxes of every element of control type `kind` inside window `hwnd`, in tree order; none on a failure.
#[allow(
    unsafe_code,
    reason = "UI Automation calls on objects this client made."
)]
pub fn boxes(auto: &IUIAutomation, hwnd: HWND, kind: UIA_CONTROLTYPE_ID) -> Vec<RECT> {
    let want = VARIANT::from(kind.0);
    let mut out = Vec::new();
    // SAFETY: plain calls on live UI Automation objects; a failure ends the read.
    unsafe {
        let Ok(cond) = auto.CreatePropertyCondition(UIA_ControlTypePropertyId, &want) else {
            return out;
        };
        let found = auto.ElementFromHandle(hwnd);
        let Ok(found) = found.and_then(|root| root.FindAll(TreeScope_Descendants, &cond)) else {
            return out;
        };
        for i in 0..found.Length().unwrap_or_default() {
            let el = found.GetElement(i);
            out.extend(el.ok().and_then(|el| uia::rect(&el).ok()));
        }
    }
    out
}

/// The element at `p`, or its nearest ancestor, that scrolls, such as Explorer's list; none when nothing there does.
#[allow(unsafe_code, reason = "UI Automation calls on a live client.")]
fn scroller_at(auto: &IUIAutomation, p: POINT) -> Option<IUIAutomationElement> {
    // SAFETY: plain calls; the walk ends where an element has no parent.
    unsafe {
        let walker = auto.ControlViewWalker().ok()?;
        let mut el = auto.ElementFromPoint(p).ok()?;
        loop {
            let scrolls = el.GetCurrentPatternAs::<IUIAutomationScrollPattern>(UIA_ScrollPatternId);
            if scrolls.is_ok() {
                return Some(el);
            }
            el = walker.GetParentElement(&el).ok()?;
        }
    }
}

/// Puts the keyboard focus on what scrolls at `p`, and true when the element that has the focus then lies in it.
#[allow(unsafe_code, reason = "UI Automation calls on live elements.")]
pub fn focus_at(auto: &IUIAutomation, p: POINT) -> bool {
    let Some(list) = scroller_at(auto, p) else {
        return false;
    };
    // SAFETY: plain calls on live elements; SetFocus moves the keyboard focus, never the pointer.
    unsafe {
        let Ok(walker) = auto.ControlViewWalker() else {
            return false;
        };
        if list.SetFocus().is_err() {
            return false;
        }
        let mut el = auto.GetFocusedElement().ok();
        while let Some(e) = el {
            if auto.CompareElements(&e, &list).is_ok_and(BOOL::as_bool) {
                return true;
            }
            el = walker.GetParentElement(&e).ok();
        }
    }
    false
}
