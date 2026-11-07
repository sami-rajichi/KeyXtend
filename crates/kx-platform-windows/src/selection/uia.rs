//! What the selection watch asks UI Automation, through the field watch's readers: an element's facts and kind, its
//! selection or caret as the field watch finds it, their boxes, and the selection's text at Translate's read only.
//! Nothing here logs what it reads.

use std::time::Instant;

use kx_module_api::Redacted;
use kx_module_api::ports::{FieldError, Rect, Screen, SelError, SelKind, SelSpot};
use kx_platform::selection::{Ours, late};
use windows::Win32::Foundation::RECT;
use windows::Win32::UI::Accessibility::{
    IUIAutomationElement, IUIAutomationTextRange, TextPatternRangeEndpoint_End,
    TextPatternRangeEndpoint_Start, TextUnit_Character,
};

use super::follow::same;
use super::kinds::{Facts, kind};
use super::spot::{Span, assemble, whole};
use super::table;
use super::win::Mine;
use crate::field::{self, Held, box_id, caret_of, named, range_fixed, value_fixed};
use crate::screen::{edges, rect_of, screen_of};
use crate::uia::{boxes, span};

/// The step a caret's empty range grows by: one character.
const STEP: i32 = 1;

/// A text range as a look asks it: its checked boxes, growing by one character, and its text at a read.
impl Span for IUIAutomationTextRange {
    fn rects(&self) -> Vec<Rect> {
        let found = boxes(self).unwrap_or_default();
        found
            .iter()
            .map(|r| rect_of([r.left, r.top, r.right, r.bottom]))
            .collect()
    }

    #[allow(unsafe_code, reason = "Plain UI Automation range calls.")]
    fn grown(&self, back: bool) -> Option<Self> {
        let (end, step) = if back {
            (TextPatternRangeEndpoint_Start, -STEP)
        } else {
            (TextPatternRangeEndpoint_End, STEP)
        };
        // SAFETY: plain calls on a live range; only our own copy moves.
        unsafe {
            let copy = self.Clone().ok()?;
            let moved = copy
                .MoveEndpointByUnit(end, TextUnit_Character, step)
                .ok()?;
            (moved != 0).then_some(copy)
        }
    }

    #[allow(
        unsafe_code,
        reason = "Reads the selection's text at Translate's click."
    )]
    fn text(&self, max_chars: u16) -> Result<String, SelError> {
        let units = usize::from(max_chars).saturating_mul(char::MAX_LEN_UTF16);
        // SAFETY: a plain call on a live range; the text is ours, kept in `Redacted` and never logged.
        let text = unsafe { self.GetText(span(units)) }.map_err(failed)?;
        Ok(whole(&text, max_chars))
    }
}

/// What UI Automation and Win32 tell about `el`, through the field watch's readers; never its text.
fn facts(el: &IUIAutomationElement, mine: Mine) -> Facts {
    let base = named(el);
    Facts {
        control: base.control,
        password: base.password,
        class: base.class,
        style: base.style,
        top: base.top,
        fixed: value_fixed(el),
        ours: mine.ours(el),
    }
}

/// The kind of `el`, and its selection or caret when it is not closed; a closed box is asked nothing more.
fn probed(
    el: &IUIAutomationElement,
    mine: Mine,
    by: Instant,
) -> Result<(SelKind, Option<field::Caret>), SelError> {
    let mut found = None;
    let kind = kind(&facts(el, mine), field::table(), table(), || {
        let got = caret_of(el);
        let fixed = got.as_ref().ok().and_then(Option::as_ref);
        let fixed = fixed.and_then(|caret| range_fixed(&caret.range));
        found = Some(got);
        fixed
    });
    if kind == SelKind::Closed {
        return Ok((kind, None));
    }
    late(by)?;
    let caret = found.unwrap_or_else(|| caret_of(el)).map_err(sel)?;
    Ok((kind, caret))
}

/// What box `el` (whose id is `id`) shows: its kind, whether its selection holds text, and its boxes; no text call.
pub(super) fn spot(
    el: &IUIAutomationElement,
    id: u64,
    mine: Mine,
    by: Instant,
) -> Result<SelSpot, SelError> {
    let (kind, caret) = probed(el, mine, by)?;
    late(by)?;
    let range = caret.map(|caret| (caret.held == Held::Yes, caret.range));
    Ok(assemble(id, kind, range, screen))
}

/// Whether `el` is box `id` and its selection holds text; false on any doubt.
pub(super) fn still(el: &IUIAutomationElement, id: u64, by: Instant) -> bool {
    let held = || matches!(caret_of(el), Ok(Some(caret)) if caret.held == Held::Yes);
    same(id, box_id(el)) && late(by).is_ok() && held()
}

/// At most `max_chars` characters of `el`'s selection from its start; empty when it holds no text.
pub(super) fn read(
    el: &IUIAutomationElement,
    max_chars: u16,
    by: Instant,
) -> Result<Redacted<String>, SelError> {
    match caret_of(el).map_err(sel)? {
        Some(caret) if caret.held == Held::Yes => {
            late(by)?;
            caret.range.text(max_chars).map(Redacted::new)
        }
        _ => Ok(Redacted::new(String::new())),
    }
}

/// The screen nearest box `r`.
fn screen(r: Rect) -> Option<Screen> {
    let [left, top, right, bottom] = edges(r);
    screen_of(&RECT {
        left,
        top,
        right,
        bottom,
    })
}

/// A failed UI Automation call as the port's error, by the field watch's rule.
fn failed(e: windows::core::Error) -> SelError {
    sel(field::failed(e))
}

/// The field watch's error as the selection port's.
pub(super) fn sel(e: FieldError) -> SelError {
    match e {
        FieldError::Unavailable => SelError::Unavailable,
        FieldError::TimedOut => SelError::TimedOut,
        FieldError::Refused => SelError::Refused,
    }
}
