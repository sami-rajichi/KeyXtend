//! Where G6 presses outside our windows on Explorer: a spot UI Automation shows is the column header's empty end or the
//! status bar itself, never a file, which single-click-to-open opens, nor a column, whose header sorts.
#![cfg(windows)]

use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::UI::Accessibility::{
    UIA_CONTROLTYPE_ID, UIA_HeaderControlTypeId, UIA_StatusBarControlTypeId,
};

use crate::keymap::Reader;
use crate::win;

/// The parts a press may land on, and only where UI Automation finds the part itself there, never an item in it.
const SAFE: [UIA_CONTROLTYPE_ID; 2] = [UIA_HeaderControlTypeId, UIA_StatusBarControlTypeId];

/// The points tried on a part's box `r`: its centre, and `inset` in from its right end at mid height.
pub fn tries(r: RECT, inset: i32) -> [POINT; 2] {
    let y = r.top + (r.bottom - r.top) / 2;
    let centre = POINT {
        x: r.left + (r.right - r.left) / 2,
        y,
    };
    [
        centre,
        POINT {
            x: r.right - inset,
            y,
        },
    ]
}

/// True when a press may land where UI Automation finds `kind` and the window there is the app's, `on_app`.
pub fn safe(kind: Option<UIA_CONTROLTYPE_ID>, on_app: bool) -> bool {
    on_app && kind.is_some_and(|k| SAFE.contains(&k))
}

/// True when a press at `p` may land on window `hwnd` now, as UI Automation and the window there show.
pub fn ok_at(reader: &Reader, hwnd: HWND, p: POINT) -> bool {
    safe(reader.kind_at(p), win::root_at(p) == hwnd)
}

/// The first spot on `hwnd`'s safe parts where a press may land, `inset` in from a part's end; none when none shows.
pub fn find(reader: &Reader, hwnd: HWND, inset: i32) -> Option<POINT> {
    SAFE.iter()
        .filter_map(|&kind| reader.box_of(hwnd, kind))
        .flat_map(|r| tries(r, inset))
        .find(|&p| ok_at(reader, hwnd, p))
}

#[cfg(test)]
mod tests {
    use windows::Win32::UI::Accessibility::{
        UIA_HeaderItemControlTypeId, UIA_ListItemControlTypeId, UIA_TextControlTypeId,
    };

    use super::*;

    #[test]
    fn a_part_is_tried_at_its_centre_then_near_its_right_end() {
        let r = RECT {
            left: 100,
            top: 40,
            right: 900,
            bottom: 60,
        };
        let [centre, end] = tries(r, 16);
        assert_eq!((centre.x, centre.y), (500, 50));
        assert_eq!((end.x, end.y), (884, 50));
    }

    #[test]
    fn a_press_lands_only_on_the_header_or_status_bar_itself_on_the_apps_window() {
        assert!(
            safe(Some(UIA_HeaderControlTypeId), true),
            "the header's empty end"
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
}
