//! Acting on elements: press a button, give a field the focus, scroll, read where a caret is.

use windows::Win32::UI::Accessibility::{
    IUIAutomationElement, IUIAutomationInvokePattern, IUIAutomationScrollPattern,
    IUIAutomationTextPattern, ScrollAmount, TextPatternRangeEndpoint_End,
    TextPatternRangeEndpoint_Start, UIA_InvokePatternId,
};

use super::{UiaError, call};

/// Presses button `el`, as a screen reader would.
///
/// # Errors
/// `UiaError::Call` when `el` cannot be pressed.
#[allow(unsafe_code, reason = "Plain UI Automation pattern calls.")]
pub fn invoke(el: &IUIAutomationElement) -> Result<(), UiaError> {
    // SAFETY: plain pattern calls on a live element.
    unsafe {
        let p: IUIAutomationInvokePattern = el
            .GetCurrentPatternAs(UIA_InvokePatternId)
            .map_err(call("invoke"))?;
        p.Invoke().map_err(call("invoke"))
    }
}

/// Gives `el` the keyboard focus.
///
/// # Errors
/// `UiaError::Call` when `el` refuses it.
#[allow(unsafe_code, reason = "A plain UI Automation call.")]
pub fn focus(el: &IUIAutomationElement) -> Result<(), UiaError> {
    // SAFETY: a plain call on a live element.
    unsafe { el.SetFocus() }.map_err(call("focus"))
}

/// Scrolls `sp` one small step per `steps`, sideways by `h` and down by `v`.
///
/// # Errors
/// `UiaError::Call` at the first step that fails.
#[allow(unsafe_code, reason = "A plain UI Automation call.")]
pub fn scroll(
    sp: &IUIAutomationScrollPattern,
    h: ScrollAmount,
    v: ScrollAmount,
    steps: u32,
) -> Result<(), UiaError> {
    for _ in 0..steps {
        // SAFETY: a plain pattern call on a live pattern.
        unsafe { sp.Scroll(h, v) }.map_err(call("scroll"))?;
    }
    Ok(())
}

/// Where the caret is in `tp`'s text, in UTF-16 units from the start.
///
/// # Errors
/// `UiaError::Call` when UI Automation cannot read the selection or the text.
#[allow(unsafe_code, reason = "Plain UI Automation text range calls.")]
pub fn caret(tp: &IUIAutomationTextPattern) -> Result<usize, UiaError> {
    let err = call("caret");
    // SAFETY: plain calls on live text ranges; the document range is our own copy.
    unsafe {
        let at = tp
            .GetSelection()
            .map_err(&err)?
            .GetElement(0)
            .map_err(&err)?;
        let before = tp.DocumentRange().map_err(&err)?;
        before
            .MoveEndpointByRange(
                TextPatternRangeEndpoint_End,
                &at,
                TextPatternRangeEndpoint_Start,
            )
            .map_err(&err)?;
        let text = before.GetText(-1).map_err(&err)?;
        Ok(text.to_string().encode_utf16().count())
    }
}
