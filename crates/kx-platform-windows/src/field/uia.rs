//! What the field watch asks UI Automation: an element's facts and box id, and the text before its caret, which only a
//! text box gives and never past the read cap in characters. Nothing here logs what it reads.

use std::hash::{DefaultHasher, Hash, Hasher};
use std::time::Instant;

use kx_module_api::Redacted;
use kx_module_api::ports::{FieldEnds, FieldError, FieldKind, FieldRead, last_words};
use kx_platform::field::{after_first, late};
use windows::Win32::Foundation::HWND;
use windows::Win32::System::Variant::VT_BOOL;
use windows::Win32::UI::Accessibility::{
    IUIAutomation, IUIAutomationElement, IUIAutomationTextPattern, IUIAutomationTextRange,
    IUIAutomationValuePattern, TextPatternRangeEndpoint_End, TextPatternRangeEndpoint_Start,
    TextUnit, TextUnit_Character, TextUnit_Word, UIA_E_TIMEOUT, UIA_IsReadOnlyAttributeId,
    UIA_TextPatternId, UIA_ValuePatternId,
};
use windows::Win32::UI::WindowsAndMessaging::{GA_ROOT, GetAncestor};
use windows::core::BOOL;

use super::{Facts, kind, table};
use crate::uia::{class_of, span, style_of, vector};
use crate::window::foreground;
use caret::{Caret, Found, Held, caret_of};

mod caret;

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
    probed(el).0
}

/// The kind of `el`, and the caret its read-only probe fetched; only a would-be text box is probed, so a text box
/// always comes with its caret's answer.
fn probed(el: &IUIAutomationElement) -> (FieldKind, Option<Found>) {
    let mut caret = None;
    let kind = kind(&facts(el), table(), || {
        let got = caret_of(el);
        let fixed = matches!(&got, Ok(Some(caret)) if read_only_range(&caret.range));
        caret = Some(got);
        fixed
    });
    (kind, caret)
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

/// At most `max_words` words before the caret of text box `el`, or before its selection's start, from at most `chars`
/// characters, with the kind the read saw; no text when `el` is no text box or has no caret.
pub(super) fn before_cursor(
    el: &IUIAutomationElement,
    max_words: u8,
    by: Instant,
    chars: usize,
) -> Result<FieldRead, FieldError> {
    let (kind, caret) = text_caret(el, by)?;
    let text = match caret {
        Some(caret) => words_before(&caret.range, max_words, by, chars)?,
        None => None,
    };
    Ok(FieldRead {
        kind,
        text: text.map(Redacted::new),
    })
}

/// At most `max_words` words before `caret`, from at most `chars` characters; `None` when the box does not tell.
fn words_before(
    caret: &IUIAutomationTextRange,
    max_words: u8,
    by: Instant,
    chars: usize,
) -> Result<Option<String>, FieldError> {
    let words = i32::from(max_words) + 1;
    if let Some(text) = tail(caret, TextUnit_Word, words, by, chars)? {
        return Ok(Some(last_words(&text, max_words).to_owned()));
    }
    let text = tail(caret, TextUnit_Character, span(chars), by, chars)?;
    Ok(text.map(|t| last_words(after_first(&t), max_words).to_owned()))
}

/// Whether the text before the caret of text box `el` ends with `want`, read within `chars` characters, with the kind
/// the check saw: no for a box that is no text box or holds a selection; `None` when it has no caret or its selection
/// is unknown. It steps back in UTF-16 units, UI Automation's character on Windows.
pub(super) fn ends_with(
    el: &IUIAutomationElement,
    want: &str,
    by: Instant,
    chars: usize,
) -> Result<FieldEnds, FieldError> {
    let (kind, caret) = text_caret(el, by)?;
    let ends = match (caret, i32::try_from(want.encode_utf16().count())) {
        _ if !kind.reads() => Some(false),
        (Some(caret), _) if caret.held == Held::Yes => return Ok(FieldEnds::selection(kind)),
        (Some(caret), Ok(back)) if caret.held == Held::No => {
            tail(&caret.range, TextUnit_Character, back, by, chars)?.map(|t| t.ends_with(want))
        }
        _ => None,
    };
    Ok(FieldEnds::new(kind, ends))
}

/// The kind of `el` checked now, and its caret, which only a text box with one gives; the check fetches it once.
fn text_caret(
    el: &IUIAutomationElement,
    by: Instant,
) -> Result<(FieldKind, Option<Caret>), FieldError> {
    let (kind, caret) = probed(el);
    if !kind.reads() {
        return Ok((kind, None));
    }
    late(by)?;
    Ok((kind, caret.unwrap_or(Ok(None))?))
}

/// The text from `back` units of `unit` before the caret up to it; `None` when it is longer than `chars` characters.
#[allow(unsafe_code, reason = "Plain UI Automation text range calls.")]
fn tail(
    caret: &IUIAutomationTextRange,
    unit: TextUnit,
    back: i32,
    by: Instant,
    chars: usize,
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
        range.GetText(span(units(chars + 1))).map_err(failed)?
    };
    let text = text.to_string();
    Ok((text.chars().count() <= chars).then_some(text))
}

/// The most UTF-16 units, Windows' string unit, that `chars` characters take.
fn units(chars: usize) -> usize {
    chars.saturating_mul(char::MAX_LEN_UTF16)
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
