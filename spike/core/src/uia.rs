//! UI Automation client: items by name, text documents, selection state and ScrollPattern.

mod geom;
mod read;

use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance};
use windows::Win32::System::Variant::VARIANT;
use windows::Win32::UI::Accessibility::{
    CUIAutomation, IUIAutomation, IUIAutomationCondition, IUIAutomationElement,
    IUIAutomationScrollPattern, IUIAutomationSelectionItemPattern, IUIAutomationTextPattern,
    IUIAutomationValuePattern, ScrollAmount, TreeScope_Descendants, UIA_ControlTypePropertyId,
    UIA_DocumentControlTypeId, UIA_IsTextPatternAvailablePropertyId, UIA_ListItemControlTypeId,
    UIA_NamePropertyId, UIA_PATTERN_ID, UIA_PROPERTY_ID, UIA_ScrollPatternId,
    UIA_SelectionItemPatternId, UIA_TextPatternId, UIA_ValuePatternId,
};
use windows::core::Interface;

pub use geom::{Pill, anchor, centre, line_points, rects_of};
pub use read::{Sel, boxes, is_password, lines, password_flag, selection};

use crate::com::Com;
use crate::hold::Pt;

/// Most ancestors walked when looking for a pattern above an element.
const MAX_DEPTH: usize = 32;

/// A UI Automation client.
pub struct Uia(IUIAutomation);

impl Uia {
    /// Starts COM on this thread and creates the client.
    pub fn new() -> Result<Self, String> {
        // Elements may outlive this client on the thread, so COM stays on.
        Com::start()?.keep();
        // SAFETY: creates the system's UIA client object.
        let auto = unsafe { CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER) }
            .map_err(|e| format!("UI Automation: {e}"))?;
        Ok(Self(auto))
    }

    /// A condition that holds when every property has its value; one property needs no AND.
    fn all(
        &self,
        props: Vec<(UIA_PROPERTY_ID, VARIANT)>,
    ) -> Result<IUIAutomationCondition, String> {
        let mut conds = Vec::new();
        for (id, value) in props {
            // SAFETY: a plain call with a live VARIANT, which the condition copies.
            let c = unsafe { self.0.CreatePropertyCondition(id, &value) };
            conds.push(c.map_err(|e| format!("UIA condition: {e}"))?);
        }
        if conds.len() == 1 {
            return conds.pop().ok_or_else(|| "UIA condition: none".to_string());
        }
        let conds: Vec<Option<IUIAutomationCondition>> = conds.into_iter().map(Some).collect();
        // SAFETY: a plain call over live conditions.
        unsafe { self.0.CreateAndConditionFromNativeArray(&conds) }
            .map_err(|e| format!("UIA condition: {e}"))
    }

    /// The first element below `under` that fits every property.
    fn first(
        &self,
        under: &IUIAutomationElement,
        props: Vec<(UIA_PROPERTY_ID, VARIANT)>,
    ) -> Option<IUIAutomationElement> {
        let cond = self.all(props).ok()?;
        // SAFETY: a plain tree search.
        unsafe { under.FindFirst(TreeScope_Descendants, &cond) }.ok()
    }

    /// The element of window `hwnd`.
    pub fn window(&self, hwnd: HWND) -> Result<IUIAutomationElement, String> {
        // SAFETY: a plain call; a dead handle gives an error.
        unsafe { self.0.ElementFromHandle(hwnd) }.map_err(|e| format!("UIA window: {e}"))
    }

    /// The list item called `name` in window `hwnd`, such as a file in Explorer.
    pub fn item(&self, hwnd: HWND, name: &str) -> Option<IUIAutomationElement> {
        let kind = VARIANT::from(UIA_ListItemControlTypeId.0);
        let props = vec![
            (UIA_NamePropertyId, VARIANT::from(name)),
            (UIA_ControlTypePropertyId, kind),
        ];
        self.first(&self.window(hwnd).ok()?, props)
    }

    /// The first element called `name` in window `hwnd`, such as a web field by its label.
    pub fn named(&self, hwnd: HWND, name: &str) -> Option<IUIAutomationElement> {
        let props = vec![(UIA_NamePropertyId, VARIANT::from(name))];
        self.first(&self.window(hwnd).ok()?, props)
    }

    /// Where to press `item`: the centre of its label called `name`, else of the item.
    pub fn grip(&self, item: &IUIAutomationElement, name: &str) -> Result<Pt, String> {
        let label = self.first(item, vec![(UIA_NamePropertyId, VARIANT::from(name))]);
        Ok(centre(&rect(label.as_ref().unwrap_or(item))?))
    }

    /// The text document in window `hwnd`: a Document with TextPattern, never an address bar.
    pub fn text(&self, hwnd: HWND) -> Option<IUIAutomationElement> {
        let has_text = (UIA_IsTextPatternAvailablePropertyId, VARIANT::from(true));
        let doc = (
            UIA_ControlTypePropertyId,
            VARIANT::from(UIA_DocumentControlTypeId.0),
        );
        self.first(&self.window(hwnd).ok()?, vec![has_text, doc])
    }

    /// The element that has the keyboard focus, in any app.
    pub fn focused(&self) -> Result<IUIAutomationElement, String> {
        // SAFETY: a plain call.
        unsafe { self.0.GetFocusedElement() }.map_err(|e| format!("UIA focus: {e}"))
    }

    /// Pattern `id` of `el`, or of its nearest ancestor that has it, at most `MAX_DEPTH` up.
    fn up<T: Interface>(&self, el: IUIAutomationElement, id: UIA_PATTERN_ID) -> Option<T> {
        // SAFETY: plain calls; each failure ends the walk.
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

    /// The TextPattern of `el`, or of its nearest ancestor that has one.
    pub fn text_of(&self, el: &IUIAutomationElement) -> Option<IUIAutomationTextPattern> {
        self.up(el.clone(), UIA_TextPatternId)
    }

    /// The ScrollPattern of the element at `p`, or of its nearest ancestor that has one.
    pub fn scroller(&self, p: Pt) -> Option<IUIAutomationScrollPattern> {
        // SAFETY: a plain call.
        let el = unsafe { self.0.ElementFromPoint(POINT { x: p.x, y: p.y }) }.ok()?;
        self.up(el, UIA_ScrollPatternId)
    }
}

/// The screen box of `el`.
pub fn rect(el: &IUIAutomationElement) -> Result<RECT, String> {
    // SAFETY: a plain property read.
    unsafe { el.CurrentBoundingRectangle() }.map_err(|e| format!("UIA box: {e}"))
}

/// The name of `el`, or empty.
pub fn name(el: &IUIAutomationElement) -> String {
    // SAFETY: a plain property read.
    unsafe { el.CurrentName() }
        .map(|b| b.to_string())
        .unwrap_or_default()
}

/// The id of the process that shows `el`, or 0.
pub fn pid(el: &IUIAutomationElement) -> u32 {
    // SAFETY: a plain property read.
    let id = unsafe { el.CurrentProcessId() }.unwrap_or_default();
    u32::try_from(id).unwrap_or_default()
}

/// The class and control type of `el`, for error messages; never its name or text.
pub fn kind(el: &IUIAutomationElement) -> String {
    // SAFETY: plain property reads.
    unsafe {
        let class = el.CurrentClassName().map(|b| b.to_string());
        let ty = el.CurrentControlType().map(|t| t.0);
        format!(
            "{} / type {}",
            class.unwrap_or_default(),
            ty.unwrap_or_default()
        )
    }
}

/// True when `el` is selected.
pub fn selected(el: &IUIAutomationElement) -> Result<bool, String> {
    // SAFETY: plain pattern calls.
    unsafe {
        let sel: IUIAutomationSelectionItemPattern = el
            .GetCurrentPatternAs(UIA_SelectionItemPatternId)
            .map_err(|e| format!("UIA selection: {e}"))?;
        sel.CurrentIsSelected()
            .map(|b| b.as_bool())
            .map_err(|e| format!("UIA selection: {e}"))
    }
}

/// Scrolls `sp` one small step per `steps`, sideways by `h` and down by `v`.
pub fn scroll(
    sp: &IUIAutomationScrollPattern,
    h: ScrollAmount,
    v: ScrollAmount,
    steps: u32,
) -> Result<(), String> {
    for _ in 0..steps {
        // SAFETY: a plain pattern call.
        unsafe { sp.Scroll(h, v) }.map_err(|e| format!("UIA scroll: {e}"))?;
    }
    Ok(())
}

/// The sideways and downward scroll percents of `sp`; -1 when it cannot scroll that way.
pub fn percent(sp: &IUIAutomationScrollPattern) -> Option<(f64, f64)> {
    // SAFETY: plain property reads.
    unsafe {
        Some((
            sp.CurrentHorizontalScrollPercent().ok()?,
            sp.CurrentVerticalScrollPercent().ok()?,
        ))
    }
}

/// The value of a text field; `None` for a password box, whose value is never read.
pub fn value(el: &IUIAutomationElement) -> Option<String> {
    if is_password(el) {
        return None;
    }
    // SAFETY: plain pattern and property reads.
    unsafe {
        let v: IUIAutomationValuePattern = el.GetCurrentPatternAs(UIA_ValuePatternId).ok()?;
        v.CurrentValue().ok().map(|b| b.to_string())
    }
}
