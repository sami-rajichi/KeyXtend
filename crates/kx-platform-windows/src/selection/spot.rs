//! A look's spot from a text range, pure over what the range tells: the selection's first and last line boxes, or the
//! caret's box, and their screen. No text is asked; only Translate's read asks it, cut at a whole character.

use kx_module_api::ports::{Rect, Screen, SelError, SelKind, SelSpot};

use crate::screen::{edges, rect_of};

/// What a look asks of a text range; only Translate's read asks its text.
pub(crate) trait Span: Sized {
    /// Its screen boxes, one per line part, each checked; empty when it is off screen or the app does not tell.
    fn rects(&self) -> Vec<Rect>;

    /// A copy grown by one character, back when `back`; `None` when it cannot grow.
    fn grown(&self, back: bool) -> Option<Self>;

    /// At most `max_chars` characters of its text, cut at a whole character.
    ///
    /// # Errors
    /// `TimedOut` or `Refused` when the app does not answer.
    fn text(&self, max_chars: u16) -> Result<String, SelError>;
}

/// The spot of box `box_id` of `kind`, with `range` the selection when it holds text (`true`), else the caret, whose
/// box is looked for only then; `screen` names the screen nearest a box. A closed box asks its range nothing, and no
/// range is asked its text.
pub(crate) fn assemble<S: Span>(
    box_id: u64,
    kind: SelKind,
    range: Option<(bool, S)>,
    screen: impl Fn(Rect) -> Option<Screen>,
) -> SelSpot {
    let mut spot = SelSpot {
        box_id,
        kind,
        selected: false,
        first: None,
        last: None,
        caret: None,
        screen: Screen::default(),
    };
    let Some((held, range)) = range.filter(|_| kind != SelKind::Closed) else {
        return spot;
    };
    if held {
        let rects = range.rects();
        spot.selected = true;
        spot.first = line(&rects, 0);
        spot.last = rects.len().checked_sub(1).and_then(|at| line(&rects, at));
    } else {
        spot.caret = caret(&range);
    }
    let near = spot.last.or(spot.caret).and_then(screen);
    spot.screen = near.unwrap_or_default();
    spot
}

/// The caret's box: its range's own, else the character after it, else the one before; `None` when none tells.
fn caret<S: Span>(range: &S) -> Option<Rect> {
    let first = |r: S| r.rects().first().copied();
    let last = |r: S| r.rects().last().copied();
    let own = range.rects().first().copied();
    own.or_else(|| range.grown(false).and_then(first))
        .or_else(|| range.grown(true).and_then(last))
}

/// The box of the line that holds `rects[at]`: it and every part whose middle lies within its height, joined.
pub(crate) fn line(rects: &[Rect], at: usize) -> Option<Rect> {
    let line = *rects.get(at)?;
    let [_, top, _, bottom] = edges(line);
    let on = rects
        .iter()
        .filter(|r| (top..bottom).contains(&middle(**r)));
    Some(on.fold(line, |joined, part| join(joined, *part)))
}

/// The height at the middle of `r`.
fn middle(r: Rect) -> i32 {
    r.y.saturating_add(r.h / 2)
}

/// The smallest box that holds `a` and `b`; a box past the range of `i32` stops at its end.
fn join(a: Rect, b: Rect) -> Rect {
    let ([al, at, ar, ab], [bl, bt, br, bb]) = (edges(a), edges(b));
    rect_of([al.min(bl), at.min(bt), ar.max(br), ab.max(bb)])
}

/// The text of `units`, at most `max_chars` characters: a half pair at the end, where a cap cut it, is dropped, and a
/// lone surrogate elsewhere reads as U+FFFD.
pub(crate) fn whole(units: &[u16], max_chars: u16) -> String {
    let mut chars = char::decode_utf16(units.iter().copied()).peekable();
    let mut text = String::new();
    for _ in 0..max_chars {
        match chars.next() {
            Some(Ok(c)) => text.push(c),
            Some(Err(_)) if chars.peek().is_some() => text.push(char::REPLACEMENT_CHARACTER),
            _ => break,
        }
    }
    text
}
