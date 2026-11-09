//! What the selection watch asks UI Automation, through the field watch's readers: the element that holds the focused
//! box's text, its facts and kind, its selection or caret as the field watch finds it, their boxes, and the selection's
//! text at Translate's read only. Nothing here logs what it reads.

use std::time::Instant;

use kx_module_api::Redacted;
use kx_module_api::ports::{FieldError, Rect, Screen, SelError, SelKind, SelSpot};
use kx_platform::selection::{Ours, late};
use windows::Win32::Foundation::RECT;
use windows::Win32::UI::Accessibility::{
    IUIAutomation, IUIAutomationElement, IUIAutomationTextPattern, IUIAutomationTextRange,
    IUIAutomationTreeWalker, TextPatternRangeEndpoint_End, TextPatternRangeEndpoint_Start,
    TextUnit_Character, UIA_TextPatternId,
};

use super::follow::same;
use super::kinds::{Facts, Tree, capped, climb, kind, shut};
use super::spot::{Span, assemble, whole};
use super::table;
use super::win::Mine;
use crate::field::{self, Held, box_id, caret_of, named, range_fixed, value_fixed};
use crate::screen::{edges, rect_of, screen_of};
use crate::uia::{boxes, span};

/// The step a caret's empty range grows by: one character.
const STEP: i32 = 1;

#[cfg(test)]
thread_local! {
    /// How many selection texts this thread asked for, for the tests that prove a look asks none.
    pub(super) static READS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

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
        #[cfg(test)]
        READS.with(|n| n.set(n.get() + 1));
        let units = usize::from(max_chars).saturating_mul(char::MAX_LEN_UTF16);
        // SAFETY: a plain call on a live range; the text is ours, kept in `Redacted` and never logged.
        let text = unsafe { self.GetText(span(units)) }.map_err(failed)?;
        Ok(whole(&text, max_chars))
    }
}

/// What UI Automation and Win32 tell about `el` but its value pattern, through the field watch's readers; never its
/// text.
fn base(el: &IUIAutomationElement, mine: Mine) -> Facts {
    let named = named(el);
    Facts {
        control: named.control,
        password: named.password,
        class: named.class,
        style: named.style,
        top: named.top,
        fixed: None,
        ours: mine.ours(el),
    }
}

/// What UI Automation and Win32 tell about `el`, its value pattern too; never its text.
fn facts(el: &IUIAutomationElement, mine: Mine) -> Facts {
    Facts {
        fixed: value_fixed(el),
        ..base(el, mine)
    }
}

/// The walk up from a focused element: the client's control-view walker, what counts as ours, and the look's deadline.
struct Up<'a> {
    walker: Option<&'a IUIAutomationTreeWalker>,
    mine: Mine,
    by: Instant,
}

impl Tree for Up<'_> {
    type El = IUIAutomationElement;

    #[allow(unsafe_code, reason = "A plain UI Automation pattern read.")]
    fn texts(&self, el: &IUIAutomationElement) -> bool {
        // SAFETY: a plain pattern read on a live element; a failure reads as none.
        unsafe { el.GetCurrentPatternAs::<IUIAutomationTextPattern>(UIA_TextPatternId) }.is_ok()
    }

    /// Shut past the deadline too, so an app that stopped answering is asked nothing more.
    fn shut(&self, el: &IUIAutomationElement) -> bool {
        late(self.by).is_err() || shut(&base(el, self.mine), field::table(), table())
    }

    #[allow(unsafe_code, reason = "A plain UI Automation tree step.")]
    fn parent(&self, el: &IUIAutomationElement) -> Option<IUIAutomationElement> {
        late(self.by).ok()?;
        // SAFETY: a plain call on a live element; the root, a gone element or a timeout gives an error.
        unsafe { self.walker?.GetParentElement(el) }.ok()
    }
}

/// The element a look at focused `el` reads, and whether it climbed: `el`, or its nearest control-view ancestor with a
/// text pattern within the table's cap, as a web page's group or label holds none; a shut element ends the walk.
#[allow(unsafe_code, reason = "Asks our client for its walker.")]
pub(super) fn text_of(
    el: IUIAutomationElement,
    auto: &IUIAutomation,
    mine: Mine,
    by: Instant,
) -> (IUIAutomationElement, bool) {
    // SAFETY: a plain call on our live client; without a walker the focused element stands.
    let walker = unsafe { auto.ControlViewWalker() }.ok();
    let up = Up {
        walker: walker.as_ref(),
        mine,
        by,
    };
    climb(&up, el, table().walk_up())
}

/// The kind of `el`, capped when the look `climbed` to it, and its selection or caret when it is not closed; a closed
/// box is asked nothing more.
fn probed(
    el: &IUIAutomationElement,
    mine: Mine,
    by: Instant,
    climbed: bool,
) -> Result<(SelKind, Option<field::Caret>), SelError> {
    let mut found = None;
    let kind = kind(&facts(el, mine), field::table(), table(), || {
        let got = caret_of(el);
        let fixed = got.as_ref().ok().and_then(Option::as_ref);
        let fixed = fixed.and_then(|caret| range_fixed(&caret.range));
        found = Some(got);
        fixed
    });
    let kind = capped(kind, climbed);
    if kind == SelKind::Closed {
        return Ok((kind, None));
    }
    late(by)?;
    let caret = found.unwrap_or_else(|| caret_of(el)).map_err(sel)?;
    Ok((kind, caret))
}

/// What box `el` (whose id is `id`, reached by a climb when `climbed`) shows: its kind, whether its selection holds
/// text, and its boxes; no text call.
pub(super) fn spot(
    el: &IUIAutomationElement,
    id: u64,
    mine: Mine,
    by: Instant,
    climbed: bool,
) -> Result<SelSpot, SelError> {
    let (kind, caret) = probed(el, mine, by, climbed)?;
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
