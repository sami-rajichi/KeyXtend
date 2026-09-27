//! UI Automation for the probes: items by name, text line boxes, selection, and ScrollPattern.

use std::ffi::c_void;

use spike_core::hold::Pt;
use windows::Win32::Foundation::{HWND, POINT, RECT, RPC_E_CHANGED_MODE};
use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, SAFEARRAY,
};
use windows::Win32::System::Ole::{
    SafeArrayAccessData, SafeArrayDestroy, SafeArrayGetDim, SafeArrayGetElemsize,
    SafeArrayGetLBound, SafeArrayGetUBound, SafeArrayUnaccessData,
};
use windows::Win32::System::Variant::VARIANT;
use windows::Win32::UI::Accessibility::{
    CUIAutomation, IUIAutomation, IUIAutomationCondition, IUIAutomationElement,
    IUIAutomationScrollPattern, IUIAutomationSelectionItemPattern, IUIAutomationTextPattern,
    ScrollAmount, TreeScope_Descendants, UIA_ControlTypePropertyId, UIA_DocumentControlTypeId,
    UIA_IsTextPatternAvailablePropertyId, UIA_ListItemControlTypeId, UIA_NamePropertyId,
    UIA_PROPERTY_ID, UIA_ScrollPatternId, UIA_SelectionItemPatternId, UIA_TextPatternId,
};

/// Numbers per box from `GetBoundingRectangles`: left, top, width, height.
const BOX: usize = 4;

/// A UI Automation client.
pub struct Uia(IUIAutomation);

impl Uia {
    /// Starts COM on this thread and creates the client.
    pub fn new() -> Result<Self, String> {
        // SAFETY: COM start-up for this thread; a thread already in another mode still works for UIA.
        let hr = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        if hr.is_err() && hr != RPC_E_CHANGED_MODE {
            return Err(format!("CoInitializeEx: {hr:?}"));
        }
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

    /// The ScrollPattern of the element at `p`, or of its nearest ancestor that has one.
    pub fn scroller(&self, p: Pt) -> Option<IUIAutomationScrollPattern> {
        // SAFETY: plain calls; each failure ends the walk.
        unsafe {
            let walker = self.0.ControlViewWalker().ok()?;
            let mut el = self.0.ElementFromPoint(POINT { x: p.x, y: p.y }).ok()?;
            loop {
                if let Ok(sp) = el.GetCurrentPatternAs(UIA_ScrollPatternId) {
                    return Some(sp);
                }
                el = walker.GetParentElement(&el).ok()?;
            }
        }
    }
}

/// The screen box of `el`.
pub fn rect(el: &IUIAutomationElement) -> Result<RECT, String> {
    // SAFETY: a plain property read.
    unsafe { el.CurrentBoundingRectangle() }.map_err(|e| format!("UIA box: {e}"))
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

/// The boxes of the visible text lines of `el`, top first.
pub fn lines(el: &IUIAutomationElement) -> Result<Vec<RECT>, String> {
    // SAFETY: plain pattern calls; the array we get is ours to read and free.
    unsafe {
        let text: IUIAutomationTextPattern = el
            .GetCurrentPatternAs(UIA_TextPatternId)
            .map_err(|e| format!("UIA text: {e}"))?;
        let range = text
            .DocumentRange()
            .map_err(|e| format!("UIA range: {e}"))?;
        let sa = range
            .GetBoundingRectangles()
            .map_err(|e| format!("UIA line boxes: {e}"))?;
        Ok(rects_of(&doubles(sa)?))
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

/// Reads and frees a one-dimensional SAFEARRAY of doubles.
///
/// # Safety
/// `sa` is null or a live array of VT_R8 that we own.
unsafe fn doubles(sa: *mut SAFEARRAY) -> Result<Vec<f64>, String> {
    if sa.is_null() {
        return Ok(Vec::new());
    }
    // SAFETY: the caller hands us a live array; it is unlocked and freed on every path.
    unsafe {
        let read = || -> windows::core::Result<Vec<f64>> {
            let shaped =
                SafeArrayGetDim(sa) == 1 && SafeArrayGetElemsize(sa) as usize == size_of::<f64>();
            if !shaped {
                return Ok(Vec::new());
            }
            let n = usize::try_from(SafeArrayGetUBound(sa, 1)? - SafeArrayGetLBound(sa, 1)? + 1)
                .unwrap_or(0);
            if n == 0 {
                return Ok(Vec::new());
            }
            let mut data: *mut c_void = std::ptr::null_mut();
            SafeArrayAccessData(sa, &mut data)?;
            let v = std::slice::from_raw_parts(data.cast::<f64>(), n).to_vec();
            SafeArrayUnaccessData(sa)?;
            Ok(v)
        };
        let got = read();
        let _ = SafeArrayDestroy(sa);
        got.map_err(|e| format!("UIA array: {e}"))
    }
}

/// Boxes from `left, top, width, height` groups; empty boxes and a partial group are dropped.
pub fn rects_of(v: &[f64]) -> Vec<RECT> {
    v.as_chunks::<BOX>()
        .0
        .iter()
        .filter(|b| b[2] > 0.0 && b[3] > 0.0)
        .map(|b| RECT {
            left: b[0].round() as i32,
            top: b[1].round() as i32,
            right: (b[0] + b[2]).round() as i32,
            bottom: (b[1] + b[3]).round() as i32,
        })
        .collect()
}

/// The centre of `r`.
pub fn centre(r: &RECT) -> Pt {
    Pt {
        x: (r.left + r.right) / 2,
        y: (r.top + r.bottom) / 2,
    }
}

/// Two points on text line `line`: `inset` px in from its start (never past its middle), and its middle.
pub fn line_points(line: &RECT, inset: i32) -> (Pt, Pt) {
    let mid = centre(line);
    let start = Pt {
        x: (line.left + inset).min(mid.x),
        y: mid.y,
    };
    (start, mid)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(left: i32, top: i32, right: i32, bottom: i32) -> RECT {
        RECT {
            left,
            top,
            right,
            bottom,
        }
    }

    #[test]
    fn line_boxes_come_from_groups_of_four_and_skip_empty_ones() {
        let v = [10.0, 20.0, 100.4, 18.6, 5.0, 5.0, 0.0, 10.0, 1.0, 2.0];
        assert_eq!(rects_of(&v), [r(10, 20, 110, 39)]);
        assert!(rects_of(&[]).is_empty());
    }

    #[test]
    fn line_points_start_inside_the_line_and_never_pass_its_middle() {
        let (start, mid) = line_points(&r(100, 50, 300, 70), 3);
        assert_eq!((start, mid), (Pt { x: 103, y: 60 }, Pt { x: 200, y: 60 }));
        let (start, mid) = line_points(&r(100, 50, 104, 70), 3);
        assert_eq!(start, mid);
    }
}
