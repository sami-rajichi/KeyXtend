//! A text box's caret over UI Automation: its first selection, which a read stops before and a check never confirms
//! while it holds text, else `TextPattern2`'s caret.

use kx_module_api::ports::FieldError;
use windows::Win32::UI::Accessibility::{
    IUIAutomationElement, IUIAutomationTextPattern, IUIAutomationTextPattern2,
    IUIAutomationTextRange, TextPatternRangeEndpoint_End, TextPatternRangeEndpoint_Start,
    UIA_TextPattern2Id, UIA_TextPatternId,
};
use windows::core::BOOL;

use super::{failed, text_call};

/// A text box's caret: the range whose start a read stops at, and whether it holds a selection.
pub(super) struct Caret {
    /// The first selection, or `TextPattern2`'s caret when no selection holds text.
    pub(super) range: IUIAutomationTextRange,
    /// The selection holds text, so no end before the caret can be confirmed.
    pub(super) selected: bool,
}

/// A caret lookup's answer: the caret, none, or why it failed.
pub(super) type Found = Result<Option<Caret>, FieldError>;

/// The caret of `el`: a selection that holds text, else `TextPattern2`'s caret, else the empty selection; `None` when
/// it has none.
pub(super) fn caret_of(el: &IUIAutomationElement) -> Found {
    text_call();
    let selection = selection(el)?;
    if selection.as_ref().is_some_and(|s| s.selected) {
        return Ok(selection);
    }
    let range = active(el);
    Ok(range
        .map(|range| Caret {
            range,
            selected: false,
        })
        .or(selection))
}

/// The first selection of `el`, and whether it holds text or more ranges follow; `None` when it tells none.
#[allow(unsafe_code, reason = "Plain UI Automation pattern and range calls.")]
fn selection(el: &IUIAutomationElement) -> Found {
    // SAFETY: plain pattern and range calls on a live element; nothing moves.
    unsafe {
        let Ok(text) = el.GetCurrentPatternAs::<IUIAutomationTextPattern>(UIA_TextPatternId) else {
            return Ok(None);
        };
        let ranges = text.GetSelection().map_err(failed)?;
        let count = ranges.Length().map_err(failed)?;
        if count == 0 {
            return Ok(None);
        }
        let range = ranges.GetElement(0).map_err(failed)?;
        let (start, end) = (TextPatternRangeEndpoint_Start, TextPatternRangeEndpoint_End);
        let span = range.CompareEndpoints(start, &range, end).map_err(failed)?;
        let selected = count > 1 || span != 0;
        Ok(Some(Caret { range, selected }))
    }
}

/// The caret `TextPattern2` gives `el`; `None` when it gives none.
#[allow(unsafe_code, reason = "Plain UI Automation pattern calls.")]
fn active(el: &IUIAutomationElement) -> Option<IUIAutomationTextRange> {
    // SAFETY: plain pattern calls on a live element; `active` is a local that outlives the call.
    unsafe {
        let text = el.GetCurrentPatternAs::<IUIAutomationTextPattern2>(UIA_TextPattern2Id);
        let mut active = BOOL::default();
        text.ok()?.GetCaretRange(&raw mut active).ok()
    }
}
