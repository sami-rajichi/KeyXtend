//! The selection port: the watch's gate, the kinds, the errors, and a spot whose `Debug` hides every box.

use super::*;
use crate::REDACTED_MARKER;
use std::collections::HashSet;

/// A line box and a caret box whose numbers appear nowhere else in the output.
const LINE: Rect = Rect {
    x: 1357,
    y: 2468,
    w: 9753,
    h: 8642,
};
const CARET: Rect = Rect {
    x: 4321,
    y: 8765,
    w: 3,
    h: 7531,
};
/// Their numbers, which no output may show.
const HIDDEN: [&str; 7] = ["1357", "2468", "9753", "8642", "4321", "8765", "7531"];

/// A spot in an editable box holding a one-line selection, with its caret at the end.
fn spot() -> SelSpot {
    let rect = Rect {
        x: 0,
        y: 0,
        w: 1920,
        h: 1040,
    };
    SelSpot {
        box_id: 9,
        kind: SelKind::Editable,
        selected: true,
        first: Some(LINE),
        last: Some(LINE),
        caret: Some(CARET),
        screen: Screen {
            rect,
            dpi: 144,
            primary: true,
        },
    }
}

#[test]
fn the_watch_is_named_and_gated_by_read_focused_text() {
    assert_eq!(SelectionWatchKey::ID.as_str(), "selection-watch");
    let gate = Some(Capability::ReadFocusedText);
    assert_eq!(SelectionWatchKey::CAPABILITY, gate, "ADR-0041 decision 3");
}

#[test]
fn the_kinds_are_listed_once_in_their_order() {
    let want = [
        SelKind::Editable,
        SelKind::ReadOnly,
        SelKind::Terminal,
        SelKind::Closed,
    ];
    assert_eq!(SelKind::ALL, want);
}

#[test]
fn a_spot_hides_every_box_and_shows_the_rest() {
    let text = format!("{:?}", spot());
    for number in HIDDEN {
        assert!(!text.contains(number), "{text} shows {number}");
    }
    assert!(text.contains(REDACTED_MARKER), "{text}");
    for shown in ["box_id: 9", "Editable", "selected: true", "dpi: 144"] {
        assert!(text.contains(shown), "{text} lacks {shown}");
    }
}

#[test]
fn a_spot_with_no_boxes_says_so() {
    let bare = SelSpot {
        first: None,
        last: None,
        caret: None,
        ..spot()
    };
    let text = format!("{bare:?}");
    assert!(
        text.contains("first: None") && text.contains("caret: None"),
        "{text}"
    );
}

#[test]
fn each_error_says_something_else() {
    let errors = [
        SelError::Unavailable,
        SelError::TimedOut,
        SelError::Refused,
        SelError::Closed,
    ];
    let texts: HashSet<_> = errors.iter().map(ToString::to_string).collect();
    assert_eq!(texts.len(), errors.len());
}

#[test]
fn only_two_equal_ids_name_the_same_box() {
    assert!(same_box(7, 7));
    assert!(!same_box(7, 8), "another box");
    assert!(!same_box(0, 0), "a box with no id is never the same");
    assert!(!same_box(7, 0) && !same_box(0, 7));
}
