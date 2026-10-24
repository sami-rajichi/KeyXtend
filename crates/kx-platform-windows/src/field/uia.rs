//! What the field watch asks UI Automation: an element's facts and box id, and the text before its caret, which only a
//! text box gives and never past the read cap. Nothing here logs what it reads.

use std::hash::{DefaultHasher, Hash, Hasher};
use std::time::Instant;

use kx_module_api::ports::{FieldError, FieldKind, last_words};
use windows::Win32::Foundation::HWND;
use windows::Win32::System::Variant::VT_BOOL;
use windows::Win32::UI::Accessibility::{
    IUIAutomation, IUIAutomationElement, IUIAutomationTextPattern, IUIAutomationTextPattern2,
    IUIAutomationTextRange, IUIAutomationValuePattern, TextPatternRangeEndpoint_End,
    TextPatternRangeEndpoint_Start, TextUnit, TextUnit_Character, TextUnit_Word, UIA_E_TIMEOUT,
    UIA_IsReadOnlyAttributeId, UIA_TextPattern2Id, UIA_TextPatternId, UIA_ValuePatternId,
};
use windows::Win32::UI::WindowsAndMessaging::{GA_ROOT, GetAncestor};
use windows::core::BOOL;

use super::{Facts, READ_CAP, after_first, kind, table};
use crate::uia::{cap, class_of, style_of, vector};
use crate::window::foreground;

/// What UI Automation and Win32 tell about `el`; a part that cannot be read reads as unknown. Never its text.
#[allow(unsafe_code, reason = "Plain UI Automation reads.")]
pub(super) fn facts(el: &IUIAutomationElement) -> Facts {
    // SAFETY: plain property reads on a live element; a failure reads as unknown.
    let (control, password, hwnd) = unsafe {
        (
            el.CurrentControlType().ok().map(|t| t.0),
            el.CurrentIsPassword().ok().map(BOOL::as_bool),
            el.CurrentNativeWindowHandle().unwrap_or_default(),
        )
    };
    let (editable, read_only) = patterns(el);
    let top = if hwnd.is_invalid() {
        foreground()
    } else {
        root(hwnd)
    };
    Facts {
        control,
        password,
        class: class_of(hwnd),
        style: style_of(hwnd),
        top: class_of(top),
        editable,
        read_only,
    }
}

/// The kind of box `el` is; only a would-be text box gets its caret's range asked whether it is read-only.
pub(super) fn kind_of(el: &IUIAutomationElement) -> FieldKind {
    kind(
        &facts(el),
        table(),
        || matches!(caret_of(el), Ok(Some(range)) if read_only_range(&range)),
    )
}

/// The top-level window `hwnd` sits in.
#[allow(unsafe_code, reason = "A plain window query.")]
fn root(hwnd: HWND) -> HWND {
    // SAFETY: a plain query; a dead handle gives null.
    unsafe { GetAncestor(hwnd, GA_ROOT) }
}

/// Whether `el` is editable (a writable value pattern or a text pattern answers) and whether its value pattern says
/// it is read-only.
#[allow(unsafe_code, reason = "Plain UI Automation pattern reads.")]
fn patterns(el: &IUIAutomationElement) -> (bool, bool) {
    // SAFETY: plain pattern and property reads on a live element; a failure reads as no.
    let (fixed, text) = unsafe {
        let value = el.GetCurrentPatternAs::<IUIAutomationValuePattern>(UIA_ValuePatternId);
        let fixed = value.and_then(|v| v.CurrentIsReadOnly()).ok();
        let text = el.GetCurrentPatternAs::<IUIAutomationTextPattern>(UIA_TextPatternId);
        (fixed.map(BOOL::as_bool), text.is_ok())
    };
    (fixed == Some(false) || text, fixed == Some(true))
}

/// Whether the text of `range` says it is read-only; a mixed or missing answer says no.
#[allow(unsafe_code, reason = "A UI Automation text attribute read.")]
fn read_only_range(range: &IUIAutomationTextRange) -> bool {
    // SAFETY: a plain call on a live range; no text is read.
    let attr = unsafe { range.GetAttributeValue(UIA_IsReadOnlyAttributeId) };
    attr.is_ok_and(|v| v.vt() == VT_BOOL && bool::try_from(&v).unwrap_or(false))
}

#[cfg(test)]
thread_local! {
    /// How many calls this thread made into a box's caret or words, for the tests that prove a closed box gets none.
    pub(super) static TEXT_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Counts one call into a box's caret or words, in tests.
fn text_call() {
    #[cfg(test)]
    TEXT_CALLS.with(|n| n.set(n.get() + 1));
}

/// A hash of `el`'s runtime id, which names it while it lives; 0 when UI Automation gives none.
#[allow(unsafe_code, reason = "Reads an element's runtime id array.")]
pub(super) fn box_id(el: &IUIAutomationElement) -> u64 {
    // SAFETY: a plain call on a live element.
    let Ok(array) = (unsafe { el.GetRuntimeId() }) else {
        return 0;
    };
    // SAFETY: the array UI Automation handed us is ours, a vector of 32-bit ids, which `vector` reads and frees.
    match unsafe { vector::<i32>(array) } {
        Ok(parts) if !parts.is_empty() => {
            let mut hash = DefaultHasher::new();
            parts.hash(&mut hash);
            hash.finish()
        }
        _ => 0,
    }
}

/// The element with the keyboard focus, in any app.
#[allow(unsafe_code, reason = "A UI Automation lookup.")]
pub(super) fn focused(auto: &IUIAutomation) -> Result<IUIAutomationElement, FieldError> {
    // SAFETY: a plain call on our live client.
    unsafe { auto.GetFocusedElement() }.map_err(failed)
}

/// At most `max_words` words before the caret of text box `el`; `None` when `el` is no text box or has no caret.
pub(super) fn before_cursor(
    el: &IUIAutomationElement,
    max_words: u8,
    by: Instant,
) -> Result<Option<String>, FieldError> {
    let Some(caret) = text_caret(el, by)? else {
        return Ok(None);
    };
    let words = i32::from(max_words) + 1;
    if let Some(text) = tail(&caret, TextUnit_Word, words, by)? {
        return Ok(Some(last_words(&text, max_words).to_owned()));
    }
    let text = tail(&caret, TextUnit_Character, cap(0), by)?;
    Ok(text.map(|t| last_words(after_first(&t), max_words).to_owned()))
}

/// Whether the text before the caret of text box `el` ends with `want`; `None` when it cannot tell.
pub(super) fn ends_with(
    el: &IUIAutomationElement,
    want: &str,
    by: Instant,
) -> Result<Option<bool>, FieldError> {
    let units = want.encode_utf16().count();
    let Some(back) = i32::try_from(units).ok().filter(|_| units <= READ_CAP) else {
        return Ok(None);
    };
    let Some(caret) = text_caret(el, by)? else {
        return Ok(None);
    };
    Ok(tail(&caret, TextUnit_Character, back, by)?.map(|t| t.ends_with(want)))
}

/// The caret of text box `el`; `None` unless `el` is a text box with one. The kind check fetches it once.
fn text_caret(
    el: &IUIAutomationElement,
    by: Instant,
) -> Result<Option<IUIAutomationTextRange>, FieldError> {
    let mut caret = None;
    let kind = kind(&facts(el), table(), || {
        let got = caret_of(el);
        let fixed = matches!(&got, Ok(Some(range)) if read_only_range(range));
        caret = Some(got);
        fixed
    });
    if kind != FieldKind::Text {
        return Ok(None);
    }
    late(by)?;
    caret.unwrap_or_else(|| caret_of(el))
}

/// The caret of `el`, from `TextPattern2` or else the first selection; `None` when it has none.
#[allow(unsafe_code, reason = "Plain UI Automation pattern calls.")]
fn caret_of(el: &IUIAutomationElement) -> Result<Option<IUIAutomationTextRange>, FieldError> {
    text_call();
    // SAFETY: plain pattern calls on a live element; `active` is a local that outlives the call.
    unsafe {
        if let Ok(text) = el.GetCurrentPatternAs::<IUIAutomationTextPattern2>(UIA_TextPattern2Id) {
            let mut active = BOOL::default();
            if let Ok(caret) = text.GetCaretRange(&raw mut active) {
                return Ok(Some(caret));
            }
        }
        let Ok(text) = el.GetCurrentPatternAs::<IUIAutomationTextPattern>(UIA_TextPatternId) else {
            return Ok(None);
        };
        let ranges = text.GetSelection().map_err(failed)?;
        if ranges.Length().map_err(failed)? == 0 {
            return Ok(None);
        }
        ranges.GetElement(0).map(Some).map_err(failed)
    }
}

/// The text from `back` units of `unit` before the caret up to it; `None` when it is longer than `READ_CAP`.
#[allow(unsafe_code, reason = "Plain UI Automation text range calls.")]
fn tail(
    caret: &IUIAutomationTextRange,
    unit: TextUnit,
    back: i32,
    by: Instant,
) -> Result<Option<String>, FieldError> {
    late(by)?;
    text_call();
    // SAFETY: plain calls on live ranges; only our own copy of the caret's range moves.
    let text = unsafe {
        let range = caret.Clone().map_err(failed)?;
        let (start, end) = (TextPatternRangeEndpoint_Start, TextPatternRangeEndpoint_End);
        range
            .MoveEndpointByRange(end, caret, start)
            .map_err(failed)?;
        range
            .MoveEndpointByUnit(start, unit, -back)
            .map_err(failed)?;
        late(by)?;
        range.GetText(cap(1)).map_err(failed)?
    };
    Ok((text.len() <= READ_CAP).then(|| text.to_string()))
}

/// `TimedOut` once `by` has passed.
pub(super) fn late(by: Instant) -> Result<(), FieldError> {
    if Instant::now() > by {
        Err(FieldError::TimedOut)
    } else {
        Ok(())
    }
}

/// A failed UI Automation call as the port's error: its own timeout is `TimedOut`, any other a refusal.
#[allow(
    clippy::needless_pass_by_value,
    reason = "It takes what `map_err` hands over."
)]
fn failed(e: windows::core::Error) -> FieldError {
    if e.code().0.cast_unsigned() == UIA_E_TIMEOUT {
        FieldError::TimedOut
    } else {
        FieldError::Refused
    }
}

#[cfg(test)]
mod tests;
