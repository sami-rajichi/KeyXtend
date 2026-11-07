//! A look's boxes over a scripted range: line boxes with no text call, a line split in parts joined, a selection
//! scrolled off, the caret's box when its range is empty, a closed box asking nothing, the screen picked, and a read cut
//! at a whole character.

use super::super::spot::{Span, assemble, line, whole};
use kx_module_api::ports::{Rect, Screen, SelError, SelKind, SelSpot};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// How often a scripted range was asked each thing.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
struct Asks {
    rects: usize,
    grown: usize,
    text: usize,
}

/// A scripted range: its boxes and the boxes of the ranges it grows into, forward and back (`None`: it cannot grow).
/// Clones share the count.
#[derive(Clone, Default)]
struct Fake {
    rects: Vec<Rect>,
    ahead: Option<Vec<Rect>>,
    behind: Option<Vec<Rect>>,
    asks: Rc<Cell<Asks>>,
}

impl Fake {
    /// A range with boxes `rects` that cannot grow.
    fn of(rects: &[Rect]) -> Self {
        Self {
            rects: rects.to_vec(),
            ..Self::default()
        }
    }

    /// Counts one ask.
    fn ask(&self, what: impl FnOnce(&mut Asks)) {
        let mut asks = self.asks.get();
        what(&mut asks);
        self.asks.set(asks);
    }
}

impl Span for Fake {
    fn rects(&self) -> Vec<Rect> {
        self.ask(|a| a.rects += 1);
        self.rects.clone()
    }

    fn grown(&self, back: bool) -> Option<Self> {
        self.ask(|a| a.grown += 1);
        let rects = if back { &self.behind } else { &self.ahead };
        rects.as_ref().map(|rects| Self {
            rects: rects.clone(),
            asks: Rc::clone(&self.asks),
            ..Self::default()
        })
    }

    fn text(&self, max_chars: u16) -> Result<String, SelError> {
        self.ask(|a| a.text += 1);
        Ok("x".repeat(usize::from(max_chars)))
    }
}

/// A box at `x`, `y` of `w` by `h`.
fn rect(x: i32, y: i32, w: i32, h: i32) -> Rect {
    Rect { x, y, w, h }
}

/// A screen whose work area is `rect`.
fn screen(rect: Rect) -> Screen {
    Screen {
        rect,
        dpi: 144,
        primary: false,
    }
}

/// The spot of box 7 of `kind` over `range`, every box on the screen named by the box itself, and the boxes the
/// screen was asked for.
fn spot_of(kind: SelKind, range: Option<(bool, Fake)>) -> (SelSpot, Vec<Rect>) {
    let seen = RefCell::new(Vec::new());
    let spot = assemble(7, kind, range, |r| {
        seen.borrow_mut().push(r);
        Some(screen(r))
    });
    (spot, seen.into_inner())
}

/// Three lines of a selection, 20 px apart.
fn lines() -> [Rect; 3] {
    [
        rect(100, 40, 300, 16),
        rect(10, 60, 500, 16),
        rect(10, 80, 120, 16),
    ]
}

#[test]
fn a_selection_gives_its_first_and_last_lines_with_no_text_call() {
    let range = Fake::of(&lines());
    let (spot, seen) = spot_of(SelKind::Editable, Some((true, range.clone())));
    let [first, _, last] = lines();
    assert!(spot.selected);
    assert_eq!(
        (spot.first, spot.last, spot.caret),
        (Some(first), Some(last), None)
    );
    assert_eq!((spot.box_id, spot.kind), (7, SelKind::Editable));
    assert_eq!(
        (spot.screen, seen),
        (screen(last), vec![last]),
        "the last line's screen"
    );
    let asks = range.asks.get();
    assert_eq!(
        (asks.text, asks.grown),
        (0, 0),
        "no text and no caret: {asks:?}"
    );
}

#[test]
fn a_selection_on_one_line_is_its_first_and_last() {
    let one = rect(40, 200, 90, 18);
    let (spot, _) = spot_of(SelKind::ReadOnly, Some((true, Fake::of(&[one]))));
    assert_eq!((spot.first, spot.last), (Some(one), Some(one)));
}

#[test]
fn an_arabic_line_split_in_parts_is_one_box() {
    // Right to left with a Latin word inside: the parts come in reading order, and the second sits a pixel higher.
    let parts = [
        rect(300, 40, 100, 16),
        rect(220, 39, 60, 18),
        rect(100, 40, 100, 16),
    ];
    let next = rect(250, 60, 150, 16);
    let range = Fake::of(&[parts[0], parts[1], parts[2], next]);
    let (spot, _) = spot_of(SelKind::Editable, Some((true, range)));
    assert_eq!(spot.first, Some(rect(100, 39, 300, 18)));
    assert_eq!(spot.last, Some(next));
    assert_eq!(line(&parts, 1), spot.first, "any part names its line");
    assert_eq!(line(&parts, 3), None);
}

#[test]
fn a_selection_scrolled_off_holds_text_but_has_no_boxes() {
    let (spot, seen) = spot_of(SelKind::Editable, Some((true, Fake::of(&[]))));
    assert!(spot.selected);
    assert_eq!((spot.first, spot.last, spot.caret), (None, None, None));
    assert_eq!((spot.screen, seen), (Screen::default(), Vec::new()));
}

#[test]
fn a_caret_with_a_box_gives_it_and_no_text_call() {
    let caret = rect(64, 300, 1, 18);
    let range = Fake::of(&[caret]);
    let (spot, seen) = spot_of(SelKind::Editable, Some((false, range.clone())));
    assert!(!spot.selected);
    assert_eq!(
        (spot.first, spot.last, spot.caret),
        (None, None, Some(caret))
    );
    assert_eq!(
        (spot.screen, seen),
        (screen(caret), vec![caret]),
        "the caret's screen"
    );
    assert_eq!(
        range.asks.get(),
        Asks {
            rects: 1,
            ..Asks::default()
        }
    );
}

#[test]
fn the_caret_box_when_the_range_is_empty_is_the_next_character_else_the_one_before() {
    let (next, before) = (rect(64, 300, 9, 18), rect(55, 300, 9, 18));
    let ahead = Fake {
        ahead: Some(vec![next]),
        behind: Some(vec![before]),
        ..Fake::default()
    };
    assert_eq!(
        spot_of(SelKind::Editable, Some((false, ahead))).0.caret,
        Some(next)
    );
    let at_end = Fake {
        ahead: Some(Vec::new()),
        behind: Some(vec![before]),
        ..Fake::default()
    };
    assert_eq!(
        spot_of(SelKind::Terminal, Some((false, at_end.clone())))
            .0
            .caret,
        Some(before)
    );
    assert_eq!(at_end.asks.get().text, 0);
    let mute = Fake {
        ahead: None,
        behind: Some(Vec::new()),
        ..Fake::default()
    };
    let (spot, seen) = spot_of(SelKind::Editable, Some((false, mute)));
    assert_eq!(
        (spot.caret, spot.screen, seen),
        (None, Screen::default(), Vec::new())
    );
}

#[test]
fn a_closed_box_or_one_with_no_range_shows_nothing() {
    let range = Fake::of(&lines());
    let (closed, _) = spot_of(SelKind::Closed, Some((true, range.clone())));
    assert_eq!(
        range.asks.get(),
        Asks::default(),
        "a closed box's range is asked nothing"
    );
    let (bare, seen) = spot_of(SelKind::ReadOnly, None::<(bool, Fake)>);
    for spot in [closed, bare] {
        assert!(!spot.selected);
        assert_eq!((spot.first, spot.last, spot.caret), (None, None, None));
        assert_eq!(spot.screen, Screen::default());
    }
    assert!(seen.is_empty());
}

#[test]
fn a_read_is_cut_at_a_whole_character() {
    let units = |text: &str| text.encode_utf16().collect::<Vec<_>>();
    assert_eq!(whole(&units("alpha beta"), 5), "alpha");
    assert_eq!(whole(&units("أنا أحب القراءة"), 7), "أنا أحب");
    assert_eq!(whole(&units("a😀b"), 3), "a😀b", "a pair is one character");
    let mut cut = units("ab😀");
    cut.pop();
    assert_eq!(whole(&cut, 9), "ab", "half a pair at the end is dropped");
    let lone = [u16::from(b'a'), 0xD83D, u16::from(b'b')];
    assert_eq!(
        whole(&lone, 9),
        "a\u{FFFD}b",
        "a lone surrogate inside reads as U+FFFD"
    );
    assert_eq!(whole(&units("abc"), 0), "");
}
