//! Tests of where G6 may press on Explorer, from fixture boxes.

use windows::Win32::UI::Accessibility::{UIA_ListItemControlTypeId, UIA_TextControlTypeId};

use super::*;

/// A fixture `[g6] spot_px`.
const INSET: i32 = 16;

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
fn the_header_comes_first_and_the_status_bar_is_tried_only_at_its_centre() {
    let (head, items) = header();
    let status = rect(100, 700, 900, 720);
    let points = tries(Some(head), &items, Some(status), INSET);
    let xy: Vec<(i32, i32)> = points.iter().map(|p| (p.x, p.y)).collect();
    assert_eq!(xy, [(516, 50), (500, 710)]);
    let bare = tries(None, &items, Some(status), INSET);
    assert_eq!(bare.len(), 1, "no header, so its columns count for nothing");
    assert!(tries(None, &[], None, INSET).is_empty());
}

#[test]
fn a_press_lands_only_on_the_header_or_status_bar_itself_on_the_apps_window() {
    assert!(
        safe(Some(UIA_HeaderControlTypeId), true),
        "past the columns"
    );
    assert!(safe(Some(UIA_StatusBarControlTypeId), true));
    let never = [
        UIA_ListItemControlTypeId,
        UIA_HeaderItemControlTypeId,
        UIA_TextControlTypeId,
    ];
    for kind in never {
        assert!(!safe(Some(kind), true), "{kind:?}");
    }
    assert!(!safe(None, true), "UI Automation found nothing");
    assert!(
        !safe(Some(UIA_HeaderControlTypeId), false),
        "another window covers it"
    );
}
