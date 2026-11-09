//! Each call of the product's caret lookup, one by one, on the focused box and on the box with the text pattern above
//! it, through a plain client and one set up as the product's. Each answer is ok, a number or Windows' code; no text.
#![cfg(windows)]

use kx_platform_windows::uia::{self, UiaError};
use serde_json::{Value, json};
use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance};
use windows::Win32::System::Variant::VARIANT;
use windows::Win32::UI::Accessibility::{
    CUIAutomation, CUIAutomation8, IUIAutomation, IUIAutomation2, IUIAutomationElement,
    IUIAutomationTextPattern, IUIAutomationTextPattern2, IUIAutomationTextRange,
    TextPatternRangeEndpoint, TextPatternRangeEndpoint_End, TextPatternRangeEndpoint_Start,
    UIA_IsTextPatternAvailablePropertyId, UIA_TextPattern2Id, UIA_TextPatternId,
};
use windows::core::{BOOL, Interface};

/// What a call that went shows when its answer is no number or flag.
const OK: &str = "ok";
/// What a call that went but gave no object shows, such as a pattern the box lacks.
const NONE: &str = "none";
/// What the walk up shows when the focused box has the text pattern itself.
const SAME: &str = "same";
/// A range's start, which the caret lookup compares with its end.
const START: TextPatternRangeEndpoint = TextPatternRangeEndpoint_Start;
/// A range's end.
const END: TextPatternRangeEndpoint = TextPatternRangeEndpoint_End;

/// The calls made so far, each with its answer, in order.
#[derive(Default)]
struct Steps(Vec<Value>);

impl Steps {
    /// Notes call `call` with its answer as `shown` tells it, or its code; the answer goes back when the call went.
    fn note<T>(
        &mut self,
        call: &str,
        got: Result<T, String>,
        shown: impl Fn(&T) -> Value,
    ) -> Option<T> {
        let (answer, kept) = match got {
            Ok(v) => (shown(&v), Some(v)),
            Err(code) => (json!(code), None),
        };
        self.0.push(json!([call, answer]));
        kept
    }
}

/// Windows' code for error `e`, such as `0x80004005`; `none` when the call went but gave no object.
fn hr(e: &windows::core::Error) -> String {
    match e.code().0 {
        0 => NONE.into(),
        code => format!("{code:#010X}"),
    }
}

/// A Windows call's answer, its error told by its code.
fn win<T>(got: windows::core::Result<T>) -> Result<T, String> {
    got.map_err(|e| hr(&e))
}

/// A call's answer that is neither a number nor a flag.
fn done<T>(_: &T) -> Value {
    json!(OK)
}

/// A count or a comparison, as a number.
fn number<T: serde::Serialize>(n: &T) -> Value {
    json!(n)
}

/// The code of a reader's failed call; `none` when it failed with no Windows code.
fn uia_hr(e: &UiaError) -> String {
    match e {
        UiaError::Call { source, .. } => hr(source),
        _ => NONE.into(),
    }
}

/// Other ways to tell whether `range` holds text, each a candidate fix: against a copy, a copy shrunk to the start,
/// and its line boxes.
#[allow(unsafe_code, reason = "Plain UI Automation range calls.")]
fn others(s: &mut Steps, range: &IUIAutomationTextRange) {
    // SAFETY: plain calls on a live range; only our own copy moves.
    unsafe {
        if let Some(copy) = s.note("clone", win(range.Clone()), done) {
            let span = range.CompareEndpoints(START, &copy, END);
            s.note("compare_clone", win(span), number);
            let shrunk = copy.MoveEndpointByRange(END, range, START);
            if s.note("collapse", win(shrunk), done).is_some() {
                s.note("same", win(range.Compare(&copy)), |b: &BOOL| {
                    json!(b.as_bool())
                });
            }
        }
    }
    let lines = uia::boxes(range).map_err(|e| uia_hr(&e));
    s.note("boxes", lines, |b| json!(b.len()));
}

/// The caret lookup's selection calls on `text` in its order, then the candidates on the first range.
#[allow(unsafe_code, reason = "Plain UI Automation pattern and range calls.")]
fn selection_calls(s: &mut Steps, text: &IUIAutomationTextPattern) {
    // SAFETY: plain calls on live objects; the only index read is 0, below the array's length.
    let range = unsafe {
        let Some(all) = s.note("selection", win(text.GetSelection()), done) else {
            return;
        };
        match s.note("length", win(all.Length()), number) {
            Some(count) if count > 0 => {}
            _ => return,
        }
        let Some(range) = s.note("element", win(all.GetElement(0)), done) else {
            return;
        };
        s.note(
            "compare",
            win(range.CompareEndpoints(START, &range, END)),
            number,
        );
        range
    };
    others(s, &range);
}

/// The caret lookup's calls on `el` in its order: `TextPattern2` and its caret, then `TextPattern` and the selection
/// calls; true when `el` has `TextPattern` itself.
#[allow(unsafe_code, reason = "Plain UI Automation pattern calls.")]
fn caret_calls(s: &mut Steps, el: &IUIAutomationElement) -> bool {
    // SAFETY: plain pattern calls on a live element; `active` is a local that outlives its call.
    let text = unsafe {
        let two = win(el.GetCurrentPatternAs::<IUIAutomationTextPattern2>(UIA_TextPattern2Id));
        if let Some(two) = s.note("pattern2", two, done) {
            let mut active = BOOL::default();
            s.note("caret", win(two.GetCaretRange(&raw mut active)), done);
        }
        win(el.GetCurrentPatternAs::<IUIAutomationTextPattern>(UIA_TextPatternId))
    };
    let Some(text) = s.note("pattern", text, done) else {
        return false;
    };
    selection_calls(s, &text);
    true
}

/// The calls on the nearest box above `el` in the control view with the text pattern, as the gate's reader walks up.
#[allow(unsafe_code, reason = "A UI Automation tree walk.")]
fn above(auto: &IUIAutomation, el: &IUIAutomationElement) -> Value {
    // SAFETY: plain calls on a live client and element; the condition is copied from a live VARIANT.
    let up = unsafe {
        let has = auto
            .CreatePropertyCondition(UIA_IsTextPatternAvailablePropertyId, &VARIANT::from(true));
        let view = auto.ControlViewCondition();
        let both = has.and_then(|h| view.and_then(|v| auto.CreateAndCondition(&h, &v)));
        both.and_then(|c| auto.CreateTreeWalker(&c))
            .and_then(|w| w.NormalizeElement(el))
    };
    let mut s = Steps::default();
    if let Some(up) = s.note("walk", win(up), done) {
        caret_calls(&mut s, &up);
    }
    json!(s.0)
}

/// The calls through client `auto` on the focused box, and on the box above when it lacks the text pattern.
#[allow(unsafe_code, reason = "A UI Automation lookup.")]
fn on(auto: &IUIAutomation) -> Value {
    let mut s = Steps::default();
    // SAFETY: a plain call on a live client.
    let focused = win(unsafe { auto.GetFocusedElement() });
    let Some(el) = s.note("focus", focused, done) else {
        return json!({ "steps": s.0 });
    };
    let own = caret_calls(&mut s, &el);
    let up = if own { json!(SAME) } else { above(auto, &el) };
    json!({ "steps": s.0, "up": up })
}

/// A client like the gate's reader.
#[allow(unsafe_code, reason = "Makes a UI Automation client.")]
fn plain() -> windows::core::Result<IUIAutomation> {
    // SAFETY: makes the system's client on this thread, where the gate keeps COM.
    unsafe { CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER) }
}

/// A client set up as the product's selection watch sets its own: the newer one, its calls ending at `read_ms`.
#[allow(unsafe_code, reason = "Makes and sets up a UI Automation client.")]
fn timed(read_ms: u32) -> windows::core::Result<IUIAutomation> {
    // SAFETY: makes the system's client on this thread, where the gate keeps COM.
    let auto: IUIAutomation =
        unsafe { CoCreateInstance(&CUIAutomation8, None, CLSCTX_INPROC_SERVER) }?;
    if let Ok(two) = auto.cast::<IUIAutomation2>() {
        // SAFETY: plain settings on our own client; a refusal keeps Windows' defaults, as the product's does.
        unsafe {
            let _ = two.SetConnectionTimeout(read_ms);
            let _ = two.SetTransactionTimeout(read_ms);
        }
    }
    Ok(auto)
}

/// Each caret call on the focused box through both clients; `read_ms` is the product's call limit.
pub fn of(read_ms: u32) -> Value {
    let through = |made: windows::core::Result<IUIAutomation>| {
        made.map_or_else(|e| json!({ "client": hr(&e) }), |a| on(&a))
    };
    json!({ "plain": through(plain()), "timed": through(timed(read_ms)) })
}

#[cfg(test)]
mod tests {
    use windows::Win32::Foundation::E_FAIL;

    use super::*;

    #[test]
    fn each_call_shows_ok_a_number_or_windows_code_in_order() {
        let mut s = Steps::default();
        assert_eq!(s.note("selection", Ok(()), done), Some(()));
        assert_eq!(s.note("length", Ok(1), number), Some(1));
        let failed = win::<()>(Err(windows::core::Error::from_hresult(E_FAIL)));
        assert_eq!(s.note("compare", failed, done), None);
        let missing = win::<()>(Err(windows::core::Error::empty()));
        assert_eq!(s.note("pattern2", missing, done), None);
        let want = json!([
            ["selection", "ok"],
            ["length", 1],
            ["compare", "0x80004005"],
            ["pattern2", "none"]
        ]);
        assert_eq!(json!(s.0), want);
    }
}
