//! The strip's rules one by one: it shows, moves and hides on each (plan edge cases 1, 2, 5, 7, 13 and 14), and the
//! caret's Paste ends on its timer, all on a fake clock.

use super::*;
use crate::chips::Rules;
use SelChip::{Copy, Cut, Paste, Translate};
use kx_module_api::US_PER_MS;
use kx_module_api::ports::{Legend, Rect, Screen, StripSpot};
use std::time::Duration;

/// Every keycap on.
const RULES: Rules = Rules {
    on: true,
    copy: true,
    cut: true,
    paste: true,
    translate: true,
    caret_paste: true,
};
/// How long the caret's Paste stays in these tests.
const CARET: Duration = Duration::from_secs(5);
/// The smallest step of time.
const TICK: Duration = Duration::from_micros(1);

/// The settings these tests follow unless they say otherwise.
fn cfg() -> StripConfig {
    StripConfig {
        rules: RULES,
        minimised: true,
        caret: CARET,
        look: Legend::Both,
    }
}

/// The screen every box is on.
fn screen() -> Screen {
    let rect = Rect {
        x: 0,
        y: 0,
        w: 1920,
        h: 1040,
    };
    Screen {
        rect,
        dpi: 96,
        primary: true,
    }
}

/// A line box at row `y`.
fn line(y: i32) -> Rect {
    Rect {
        x: 100,
        y,
        w: 200,
        h: 20,
    }
}

/// Box `id` of `kind` with a selection on the line at `y`.
fn sel(id: u64, kind: SelKind, y: i32) -> SelSpot {
    SelSpot {
        box_id: id,
        kind,
        selected: true,
        first: Some(line(y)),
        last: Some(line(y)),
        caret: None,
        screen: screen(),
    }
}

/// The caret box at `x`.
fn caret_box(x: i32) -> Rect {
    Rect {
        x,
        y: 300,
        w: 1,
        h: 20,
    }
}

/// Editable box `id` with a bare caret at `x`.
fn caret(id: u64, x: i32) -> SelSpot {
    SelSpot {
        selected: false,
        first: None,
        last: None,
        caret: Some(caret_box(x)),
        ..sel(id, SelKind::Editable, 0)
    }
}

/// Where the strip goes for the line at `y`.
fn on_line(y: i32) -> StripSpot {
    StripSpot {
        first: line(y),
        last: line(y),
        screen: screen(),
    }
}

/// Where the strip goes for the caret at `x`.
fn by_caret(x: i32) -> StripSpot {
    StripSpot {
        first: caret_box(x),
        last: caret_box(x),
        screen: screen(),
    }
}

/// A look that no click came before.
fn look(spot: SelSpot) -> Input {
    Input::Look {
        spot,
        click: false,
        clip: true,
    }
}

/// A look just after a press and release, with something on the clipboard.
fn click(spot: SelSpot) -> Input {
    Input::Look {
        spot,
        click: true,
        clip: true,
    }
}

/// The strip and its fake clock.
struct Rig {
    strip: Strip,
    now: Mono,
}

impl Rig {
    fn new() -> Self {
        Self::with(cfg())
    }

    fn with(cfg: StripConfig) -> Self {
        let now = Mono::from_us(u64::from(US_PER_MS));
        Self {
            strip: Strip::new(cfg),
            now,
        }
    }

    fn step(&mut self, input: Input) -> Option<ChipsView> {
        self.strip.step(input, self.now)
    }

    /// Lets `by` pass, then ticks.
    fn wait(&mut self, by: Duration) -> Option<ChipsView> {
        self.now = self.now.saturating_add(by);
        self.step(Input::Tick)
    }

    /// Shows a strip through `input` and gives its round.
    fn show(&mut self, input: Input) -> u32 {
        shown(self.step(input)).0
    }
}

/// The round, keycaps and spot of a view that shows a new strip; it fails the test otherwise.
fn shown(view: Option<ChipsView>) -> (u32, Vec<SelChip>, StripSpot) {
    match view {
        Some(ChipsView::Shown {
            round,
            chips,
            spot,
            acted: None,
            ..
        }) => (round, chips, spot),
        other => panic!("not a new strip: {other:?}"),
    }
}

#[test]
fn a_selection_with_a_line_box_shows_its_kinds_keycaps() {
    let mut rig = Rig::new();
    let (_, chips, spot) = shown(rig.step(look(sel(1, SelKind::Editable, 40))));
    assert_eq!(
        (chips, spot),
        (vec![Copy, Cut, Paste, Translate], on_line(40))
    );
    let (_, chips, _) = shown(rig.step(look(sel(2, SelKind::ReadOnly, 40))));
    assert_eq!(chips, [Copy, Translate]);
    let (_, chips, _) = shown(rig.step(look(sel(3, SelKind::Terminal, 40))));
    assert_eq!(chips, [Copy, Paste, Translate], "never Cut");
}

#[test]
fn an_empty_selection_shows_nothing() {
    let mut rig = Rig::new();
    assert_eq!(rig.step(look(caret(1, 50))), None);
    let empty = SelSpot {
        selected: false,
        ..sel(1, SelKind::Editable, 40)
    };
    assert_eq!(rig.step(look(empty)), None, "line boxes but no text");
    assert_eq!(rig.strip.view(), ChipsView::Hidden);
}

#[test]
fn a_selection_scrolled_off_shows_nothing_until_it_comes_back() {
    let mut rig = Rig::new();
    let off = SelSpot {
        first: None,
        last: None,
        ..sel(1, SelKind::Editable, 40)
    };
    assert_eq!(rig.step(look(off)), None);
    let (_, _, spot) = shown(rig.step(look(sel(1, SelKind::Editable, 40))));
    assert_eq!(spot, on_line(40));
    assert_eq!(
        rig.step(look(off)),
        Some(ChipsView::Hidden),
        "scrolled off again"
    );
}

#[test]
fn one_line_box_told_serves_as_both() {
    let mut rig = Rig::new();
    let last_only = SelSpot {
        first: None,
        ..sel(1, SelKind::Editable, 40)
    };
    assert_eq!(shown(rig.step(look(last_only))).2, on_line(40));
    let first_only = SelSpot {
        last: None,
        ..sel(1, SelKind::Editable, 80)
    };
    assert_eq!(shown(rig.step(look(first_only))).2, on_line(80));
}

#[test]
fn a_closed_box_shows_nothing_and_hides_a_shown_strip() {
    let mut rig = Rig::new();
    assert_eq!(rig.step(look(sel(1, SelKind::Closed, 40))), None);
    rig.show(look(sel(2, SelKind::Editable, 40)));
    let closed = rig.step(look(sel(2, SelKind::Closed, 40)));
    assert_eq!(closed, Some(ChipsView::Hidden));
}

#[test]
fn a_selection_gone_at_the_next_look_hides() {
    let mut rig = Rig::new();
    rig.show(look(sel(1, SelKind::Editable, 40)));
    assert_eq!(rig.step(look(caret(1, 50))), Some(ChipsView::Hidden));
}

#[test]
fn the_same_look_again_changes_nothing() {
    let mut rig = Rig::new();
    let round = rig.show(look(sel(1, SelKind::Editable, 40)));
    assert_eq!(rig.step(look(sel(1, SelKind::Editable, 40))), None);
    assert_eq!(rig.step(Input::Key), None, "the next look decides");
    assert!(matches!(rig.strip.view(), ChipsView::Shown { round: r, .. } if r == round));
}

#[test]
fn a_moved_line_or_a_new_box_moves_the_strip_with_a_new_round() {
    let mut rig = Rig::new();
    let first = rig.show(look(sel(1, SelKind::Editable, 40)));
    let (moved, _, spot) = shown(rig.step(look(sel(1, SelKind::Editable, 90))));
    assert!(moved > first);
    assert_eq!(spot, on_line(90));
    let other = rig.show(look(sel(2, SelKind::Editable, 90)));
    assert!(other > moved, "same line, another box");
}

mod acts;
mod carets;
mod hides;
mod legend;
