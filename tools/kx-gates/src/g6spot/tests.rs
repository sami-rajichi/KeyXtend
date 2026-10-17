//! Tests of where G6 may press on Explorer, from fixture boxes and control types.

use windows::Win32::UI::Accessibility::{
    UIA_ButtonControlTypeId, UIA_DataItemControlTypeId, UIA_EditControlTypeId,
    UIA_GroupControlTypeId, UIA_HyperlinkControlTypeId, UIA_ListControlTypeId,
    UIA_ListItemControlTypeId, UIA_PaneControlTypeId, UIA_SplitButtonControlTypeId,
    UIA_TextControlTypeId, UIA_ThumbControlTypeId, UIA_WindowControlTypeId,
};

use super::*;

/// A fixture `[g6] spot_px`.
const INSET: i32 = 16;
/// What lies above a part in Explorer's tree: a pane in its window.
const ABOVE: [UIA_CONTROLTYPE_ID; 2] = [UIA_PaneControlTypeId, UIA_WindowControlTypeId];

fn rect(left: i32, top: i32, right: i32, bottom: i32) -> RECT {
    RECT {
        left,
        top,
        right,
        bottom,
    }
}

/// A header row across 100 to 900, and its two columns, Name and Date.
fn header() -> (RECT, [RECT; 2]) {
    let head = rect(100, 40, 900, 60);
    (head, [rect(100, 40, 300, 60), rect(300, 40, 500, 60)])
}

/// `at`, then what lies above a part.
fn chain(at: &[UIA_CONTROLTYPE_ID]) -> Vec<UIA_CONTROLTYPE_ID> {
    at.iter().chain(&ABOVE).copied().collect()
}

#[test]
fn the_header_is_pressed_just_past_its_last_column_at_mid_height() {
    let (head, items) = header();
    let p = header_spot(head, &items, INSET).expect("room past the columns");
    assert_eq!((p.x, p.y), (516, 50));
}

#[test]
fn a_header_its_columns_fill_or_that_shows_none_gives_no_spot() {
    let head = rect(100, 40, 900, 60);
    let full = [rect(100, 40, 890, 60)];
    assert_eq!(header_spot(head, &full, INSET), None, "no room left");
    assert_eq!(header_spot(head, &[], INSET), None, "no column read");
    let elsewhere = [rect(100, 400, 800, 420)];
    assert_eq!(
        header_spot(head, &elsewhere, INSET),
        None,
        "another header's"
    );
}

#[test]
fn a_last_column_that_runs_past_the_header_counts_too() {
    let (head, [name, date]) = header();
    let wide = rect(500, 40, 1300, 60);
    assert_eq!(
        header_spot(head, &[name, date, wide], INSET),
        None,
        "its centre lies past the header, its box still covers the spot"
    );
}

#[test]
fn the_header_comes_first_and_the_status_bar_is_tried_only_at_its_centre() {
    let (head, items) = header();
    let status = rect(100, 700, 900, 720);
    let spots = tries(Some(head), &items, Some(status), INSET);
    let got: Vec<_> = spots.iter().map(|s| (s.at.x, s.at.y, s.part)).collect();
    assert_eq!(
        got,
        [
            (516, 50, Some(Part::Header)),
            (500, 710, Some(Part::Status))
        ]
    );
    let bare = tries(None, &items, Some(status), INSET);
    assert_eq!(bare.len(), 1, "no header, so its columns count for nothing");
    assert!(tries(None, &[], None, INSET).is_empty());
}

#[test]
fn past_the_columns_the_header_or_a_bare_container_is_safe() {
    let ok = [
        UIA_HeaderControlTypeId,
        UIA_ListControlTypeId,
        UIA_PaneControlTypeId,
        UIA_GroupControlTypeId,
    ];
    for kind in ok {
        assert!(safe(Part::Header, &chain(&[kind]), true), "{kind:?}");
    }
    let never = [
        UIA_HeaderItemControlTypeId,
        UIA_ListItemControlTypeId,
        UIA_TextControlTypeId,
        UIA_StatusBarControlTypeId,
    ];
    for kind in never {
        assert!(!safe(Part::Header, &chain(&[kind]), true), "{kind:?}");
    }
    assert!(
        !safe(Part::Header, &[], true),
        "UI Automation found nothing"
    );
    let head = chain(&[UIA_HeaderControlTypeId]);
    assert!(
        !safe(Part::Header, &head, false),
        "another window covers it"
    );
}

#[test]
fn a_container_inside_a_file_column_or_control_is_never_safe() {
    let inside = [
        UIA_ListItemControlTypeId,
        UIA_DataItemControlTypeId,
        UIA_HeaderItemControlTypeId,
        UIA_ButtonControlTypeId,
        UIA_SplitButtonControlTypeId,
        UIA_ThumbControlTypeId,
        UIA_HyperlinkControlTypeId,
        UIA_EditControlTypeId,
    ];
    for owner in inside {
        let walk = chain(&[UIA_PaneControlTypeId, owner]);
        assert!(!safe(Part::Header, &walk, true), "{owner:?}");
    }
}

#[test]
fn on_the_status_bar_only_the_bar_or_its_own_text_is_safe() {
    let bar = chain(&[UIA_StatusBarControlTypeId]);
    assert!(safe(Part::Status, &bar, true));
    let text = chain(&[UIA_TextControlTypeId, UIA_StatusBarControlTypeId]);
    assert!(safe(Part::Status, &text, true), "its item count");
    assert!(
        !safe(Part::Status, &text, false),
        "another window covers it"
    );
    let never = [
        chain(&[UIA_ButtonControlTypeId, UIA_StatusBarControlTypeId]),
        chain(&[UIA_SplitButtonControlTypeId, UIA_StatusBarControlTypeId]),
        chain(&[UIA_TextControlTypeId]),
        chain(&[UIA_PaneControlTypeId]),
        chain(&[UIA_HeaderControlTypeId]),
    ];
    for walk in never {
        assert!(!safe(Part::Status, &walk, true), "{walk:?}");
    }
}

#[test]
fn each_try_records_its_part_type_class_and_owner_only() {
    let tried = Tried {
        part: Part::Status,
        kind: Some(UIA_ButtonControlTypeId),
        class: "Button".to_string(),
        owner: "[CabinetWClass] of explorer.exe".to_string(),
    };
    let note = tried.json();
    assert_eq!(note["part"], "status");
    assert_eq!(note["type"], UIA_ButtonControlTypeId.0);
    assert_eq!(note["class"], "Button");
    assert_eq!(note["owner"], "[CabinetWClass] of explorer.exe");
    assert_eq!(
        note.as_object().map(serde_json::Map::len),
        Some(4),
        "no point"
    );
    let empty = Tried {
        kind: None,
        ..tried
    };
    assert!(
        empty.json()["type"].is_null(),
        "UI Automation found nothing"
    );
}
