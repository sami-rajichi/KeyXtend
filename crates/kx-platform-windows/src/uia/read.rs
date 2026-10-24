//! Plain reads from UI Automation: an element's box, name, process and state, line boxes, the selection, and the
//! password flag with the Win32 edit style as its fallback.

use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::UI::Accessibility::{
    IUIAutomationElement, IUIAutomationScrollPattern, IUIAutomationSelectionItemPattern,
    IUIAutomationTextPattern, IUIAutomationTextRange, IUIAutomationValuePattern,
    UIA_SelectionItemPatternId, UIA_TextPatternId, UIA_ValuePatternId,
};
use windows::Win32::UI::WindowsAndMessaging::{GWL_STYLE, GetWindowLongPtrW, RealGetWindowClassW};
use windows::core::BOOL;

use super::rects::{rects_of, vector};
use super::{UiaError, call};
use crate::field;

/// Longest Win32 window class name, in UTF-16 units.
const CLASS_MAX: usize = 256;

/// The screen box of `el`.
///
/// # Errors
/// `UiaError::Call` when UI Automation cannot read it.
#[allow(unsafe_code, reason = "A plain UI Automation read.")]
pub fn rect(el: &IUIAutomationElement) -> Result<RECT, UiaError> {
    // SAFETY: a plain property read on a live element.
    unsafe { el.CurrentBoundingRectangle() }.map_err(call("box"))
}

/// The name of `el`, or empty.
#[must_use]
#[allow(unsafe_code, reason = "A plain UI Automation read.")]
pub fn name(el: &IUIAutomationElement) -> String {
    // SAFETY: a plain property read on a live element.
    unsafe { el.CurrentName() }
        .map(|b| b.to_string())
        .unwrap_or_default()
}

/// The id of the process that shows `el`, or 0.
#[must_use]
#[allow(unsafe_code, reason = "A plain UI Automation read.")]
pub fn pid(el: &IUIAutomationElement) -> u32 {
    // SAFETY: a plain property read on a live element.
    let id = unsafe { el.CurrentProcessId() }.unwrap_or_default();
    u32::try_from(id).unwrap_or_default()
}

/// The class and control type of `el`, for error messages; never its name or text.
#[must_use]
#[allow(unsafe_code, reason = "Plain UI Automation reads.")]
pub fn kind(el: &IUIAutomationElement) -> String {
    // SAFETY: plain property reads on a live element.
    let (class, ty) = unsafe {
        (
            el.CurrentClassName().map(|b| b.to_string()),
            el.CurrentControlType().map(|t| t.0),
        )
    };
    format!(
        "{} / type {}",
        class.unwrap_or_default(),
        ty.unwrap_or_default()
    )
}

/// True when `el` is selected.
///
/// # Errors
/// `UiaError::Call` when `el` cannot be selected or UI Automation cannot say.
#[allow(unsafe_code, reason = "Plain UI Automation pattern calls.")]
pub fn selected(el: &IUIAutomationElement) -> Result<bool, UiaError> {
    // SAFETY: plain pattern calls on a live element.
    unsafe {
        let sel: IUIAutomationSelectionItemPattern = el
            .GetCurrentPatternAs(UIA_SelectionItemPatternId)
            .map_err(call("selection"))?;
        sel.CurrentIsSelected()
            .map(BOOL::as_bool)
            .map_err(call("selection"))
    }
}

/// The sideways and downward scroll percents of `sp`; -1 when it cannot scroll that way.
#[must_use]
#[allow(unsafe_code, reason = "Plain UI Automation reads.")]
pub fn percent(sp: &IUIAutomationScrollPattern) -> Option<(f64, f64)> {
    // SAFETY: plain property reads on a live pattern.
    unsafe {
        Some((
            sp.CurrentHorizontalScrollPercent().ok()?,
            sp.CurrentVerticalScrollPercent().ok()?,
        ))
    }
}

/// The value of a text field; `None` for a password box, whose value is never read, and for one past `max` UTF-16
/// units.
#[must_use]
#[allow(unsafe_code, reason = "Plain UI Automation reads.")]
pub fn value(el: &IUIAutomationElement, max: usize) -> Option<String> {
    if is_password(el) {
        return None;
    }
    // SAFETY: plain pattern and property reads on a live element.
    unsafe {
        let v: IUIAutomationValuePattern = el.GetCurrentPatternAs(UIA_ValuePatternId).ok()?;
        let value = v.CurrentValue().ok()?;
        (value.len() <= max).then(|| value.to_string())
    }
}

/// The boxes of the visible text lines of `el`, top first.
///
/// # Errors
/// `UiaError::Call` when `el` has no text or UI Automation cannot read it.
#[allow(unsafe_code, reason = "Plain UI Automation pattern calls.")]
pub fn lines(el: &IUIAutomationElement) -> Result<Vec<RECT>, UiaError> {
    // SAFETY: plain pattern calls on a live element.
    let range = unsafe {
        let text: IUIAutomationTextPattern = el
            .GetCurrentPatternAs(UIA_TextPatternId)
            .map_err(call("text"))?;
        text.DocumentRange().map_err(call("range"))?
    };
    boxes(&range)
}

/// The screen boxes of `range`, one per line part.
///
/// # Errors
/// `UiaError::Call` when UI Automation cannot read them.
#[allow(unsafe_code, reason = "A UI Automation call that hands back an array.")]
pub fn boxes(range: &IUIAutomationTextRange) -> Result<Vec<RECT>, UiaError> {
    // SAFETY: a plain call on a live range; the array we get is ours, and `vector` reads and frees it.
    unsafe {
        let sa = range.GetBoundingRectangles().map_err(call("boxes"))?;
        Ok(rects_of(&vector::<f64>(sa).map_err(call("array"))?))
    }
}

/// The selected text of a text control and its screen boxes. Sensitive: never log the text.
pub struct Sel {
    /// The selected text, capped by the caller.
    pub text: String,
    /// One box per selected line part; empty when the selection is off screen.
    pub boxes: Vec<RECT>,
}

/// Adds at most `left` characters of `part` to `out`, and takes them off `left`.
pub(super) fn spend(out: &mut String, part: &str, left: &mut usize) {
    let taken: String = part.chars().take(*left).collect();
    *left -= taken.chars().count();
    out.push_str(&taken);
}

/// The selection of `tp`, all ranges together capped at `max_chars`; `None` for a bare caret.
///
/// # Errors
/// `UiaError::Call` when UI Automation cannot read the selection.
#[allow(unsafe_code, reason = "Plain UI Automation pattern calls.")]
pub fn selection(tp: &IUIAutomationTextPattern, max_chars: usize) -> Result<Option<Sel>, UiaError> {
    let mut sel = Sel {
        text: String::new(),
        boxes: Vec::new(),
    };
    let mut left = max_chars;
    let err = call("selection");
    // SAFETY: plain pattern calls on live objects; indexes stay below the array length.
    unsafe {
        let ranges = tp.GetSelection().map_err(&err)?;
        for i in 0..ranges.Length().map_err(&err)? {
            if left == 0 {
                break;
            }
            let range = ranges.GetElement(i).map_err(&err)?;
            // A negative length means "all" to UIA, so the budget never goes below 1 here.
            let ask = i32::try_from(left).unwrap_or(i32::MAX);
            let part = range.GetText(ask).map_err(&err)?.to_string();
            spend(&mut sel.text, &part, &mut left);
            // A range with no box still counts; the pill just has less to go on.
            sel.boxes.extend(boxes(&range).unwrap_or_default());
        }
    }
    Ok((!sel.text.is_empty()).then_some(sel))
}

/// True when `el` is a password box, from UIA or its edit window's style; an unreadable UIA answer counts as yes.
#[must_use]
#[allow(unsafe_code, reason = "Plain UI Automation reads.")]
pub fn is_password(el: &IUIAutomationElement) -> bool {
    // SAFETY: plain property reads on a live element; a failure reads as unknown.
    let (uia, hwnd) = unsafe {
        let uia = el.CurrentIsPassword().ok().map(BOOL::as_bool);
        (uia, el.CurrentNativeWindowHandle().unwrap_or_default())
    };
    password_flag(uia, &class_of(hwnd), style_of(hwnd))
}

/// UIA's password answer (`None` when it could not answer, which fails safe), else the password bit of an edit window.
#[must_use]
pub fn password_flag(uia: Option<bool>, class: &str, style: u32) -> bool {
    uia != Some(false) || field::table().hides(class, style)
}

/// The base class of window `hwnd`, so an Edit subclassed under another name still counts; empty for none.
#[allow(unsafe_code, reason = "Reads a window's class name.")]
pub(crate) fn class_of(hwnd: HWND) -> String {
    if hwnd.is_invalid() {
        return String::new();
    }
    let mut buf = [0u16; CLASS_MAX];
    // SAFETY: fills a local buffer of its given length; a dead handle gives 0.
    let len = unsafe { RealGetWindowClassW(hwnd, &mut buf) } as usize;
    String::from_utf16_lossy(&buf[..len.min(CLASS_MAX)])
}

/// The style bits of window `hwnd`; 0 for none.
#[allow(unsafe_code, reason = "Reads a window's style.")]
pub(crate) fn style_of(hwnd: HWND) -> u32 {
    if hwnd.is_invalid() {
        return 0;
    }
    // SAFETY: a plain read; a dead handle gives 0. The style is the low 32 bits of the long, so the cut is intended.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let style = unsafe { GetWindowLongPtrW(hwnd, GWL_STYLE) } as u32;
    style
}
