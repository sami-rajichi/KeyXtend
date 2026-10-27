//! A text box's caret over UI Automation: `TextPattern2`'s caret or the first selection, and whether a selection
//! holds text, which a read stops before and a check never confirms.

use kx_module_api::ports::FieldError;
use windows::Win32::UI::Accessibility::{
    IUIAutomationElement, IUIAutomationTextPattern, IUIAutomationTextPattern2,
    IUIAutomationTextRange, TextPatternRangeEndpoint_End, TextPatternRangeEndpoint_Start,
    UIA_TextPattern2Id, UIA_TextPatternId,
};
use windows::core::BOOL;

use super::{failed, text_call};

/// Whether a box's selection holds text, as `GetSelection` told.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum Held {
    /// Nothing is selected.
    No,
    /// The selection holds text, so no end before the caret can be confirmed.
    Yes,
    /// `GetSelection` failed or is missing, so no end can be confirmed either.
    Unknown,
}

/// A text box's caret: the range whose start a read stops at, and whether a selection holds text.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct Caret<R = IUIAutomationTextRange> {
    /// The selection when it holds text, else `TextPattern2`'s caret, else the empty selection.
    pub(super) range: R,
    /// Whether a selection holds text.
    pub(super) held: Held,
}

/// A caret lookup's answer: the caret, none, or why it failed.
pub(super) type Found<R = IUIAutomationTextRange> = Result<Option<Caret<R>>, FieldError>;

/// What `GetSelection` told of a box.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Sel<R> {
    /// The box has no text pattern.
    Missing,
    /// No range: nothing is selected.
    Empty,
    /// The first range, and whether the selection holds text.
    Range(R, bool),
}

/// The caret of `el`, from `TextPattern2`'s caret and its selection; `None` when it has neither.
pub(super) fn caret_of(el: &IUIAutomationElement) -> Found {
    text_call();
    let caret = active(el);
    chosen(caret, selection(el))
}

/// The caret from `TextPattern2`'s `caret` and what `GetSelection` told: a selection that holds text wins, and a
/// failed or missing one keeps that caret, unsure; with no caret the selection serves.
pub(super) fn chosen<R>(caret: Option<R>, sel: Result<Sel<R>, FieldError>) -> Found<R> {
    let found = |range, held| Ok(Some(Caret { range, held }));
    match (caret, sel) {
        (_, Ok(Sel::Range(range, true))) => found(range, Held::Yes),
        (Some(range), Ok(Sel::Range(..) | Sel::Empty)) | (None, Ok(Sel::Range(range, false))) => {
            found(range, Held::No)
        }
        (Some(range), Ok(Sel::Missing) | Err(_)) => found(range, Held::Unknown),
        (None, Ok(Sel::Missing | Sel::Empty)) => Ok(None),
        (None, Err(error)) => Err(error),
    }
}

/// What `GetSelection` tells of `el`: its first range, and whether the selection holds text or more ranges follow.
#[allow(unsafe_code, reason = "Plain UI Automation pattern and range calls.")]
fn selection(el: &IUIAutomationElement) -> Result<Sel<IUIAutomationTextRange>, FieldError> {
    // SAFETY: plain pattern and range calls on a live element; nothing moves.
    unsafe {
        let Ok(text) = el.GetCurrentPatternAs::<IUIAutomationTextPattern>(UIA_TextPatternId) else {
            return Ok(Sel::Missing);
        };
        let ranges = text.GetSelection().map_err(failed)?;
        let count = ranges.Length().map_err(failed)?;
        if count == 0 {
            return Ok(Sel::Empty);
        }
        let range = ranges.GetElement(0).map_err(failed)?;
        let (start, end) = (TextPatternRangeEndpoint_Start, TextPatternRangeEndpoint_End);
        let span = range.CompareEndpoints(start, &range, end).map_err(failed)?;
        Ok(Sel::Range(range, count > 1 || span != 0))
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
