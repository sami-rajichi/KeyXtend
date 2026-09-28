//! Acting on elements: press a button, give a field the focus, read where its caret is.

use windows::Win32::UI::Accessibility::{
    IUIAutomationElement, IUIAutomationInvokePattern, IUIAutomationTextPattern,
    TextPatternRangeEndpoint_End, TextPatternRangeEndpoint_Start, UIA_InvokePatternId,
};

/// Presses button `el`, as a screen reader would.
pub fn invoke(el: &IUIAutomationElement) -> Result<(), String> {
    // SAFETY: plain pattern calls on a live element.
    unsafe {
        let p: IUIAutomationInvokePattern = el
            .GetCurrentPatternAs(UIA_InvokePatternId)
            .map_err(|e| format!("UIA invoke: {e}"))?;
        p.Invoke().map_err(|e| format!("UIA invoke: {e}"))
    }
}

/// Gives `el` the keyboard focus.
pub fn focus(el: &IUIAutomationElement) -> Result<(), String> {
    // SAFETY: a plain call on a live element.
    unsafe { el.SetFocus() }.map_err(|e| format!("UIA focus: {e}"))
}

/// Where the caret is in `tp`'s text, in UTF-16 units from the start.
pub fn caret(tp: &IUIAutomationTextPattern) -> Result<usize, String> {
    let err = |e: windows::core::Error| format!("UIA caret: {e}");
    // SAFETY: plain calls on live text ranges; the document range is our own copy.
    unsafe {
        let at = tp.GetSelection().map_err(err)?.GetElement(0).map_err(err)?;
        let before = tp.DocumentRange().map_err(err)?;
        before
            .MoveEndpointByRange(
                TextPatternRangeEndpoint_End,
                &at,
                TextPatternRangeEndpoint_Start,
            )
            .map_err(err)?;
        let text = before.GetText(-1).map_err(err)?;
        Ok(text.to_string().encode_utf16().count())
    }
}
