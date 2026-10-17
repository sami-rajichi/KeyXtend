//! Where G6 presses outside our windows on Explorer: just right of the column header's last column, or the status
//! bar's centre, each only where UI Automation then shows that part itself, never a file, which single-click-to-open
//! opens, nor a column, whose header sorts.
#![cfg(windows)]

use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::UI::Accessibility::{
    UIA_CONTROLTYPE_ID, UIA_HeaderControlTypeId, UIA_HeaderItemControlTypeId,
    UIA_StatusBarControlTypeId,
};

use crate::keymap::{Reader, centre};
use crate::{g6uia, win};

/// The parts a press may land on, and only where UI Automation finds the part itself there, never an item in it.
const SAFE: [UIA_CONTROLTYPE_ID; 2] = [UIA_HeaderControlTypeId, UIA_StatusBarControlTypeId];

/// True when `p` lies inside box `r`.
fn inside(p: POINT, r: RECT) -> bool {
    r.left <= p.x && p.x < r.right && r.top <= p.y && p.y < r.bottom
}

/// Where a press may land on column header `head`: `inset` right of the last right edge of its `items`, those whose
/// centre lies in it, at mid height; none when no item reads or that point is past the header's end.
pub fn header_spot(head: RECT, items: &[RECT], inset: i32) -> Option<POINT> {
    let row = items.iter().filter(|r| inside(centre(r), head));
    let x = row.map(|r| r.right).max()?.saturating_add(inset);
    (x < head.right).then_some(POINT {
        x,
        y: centre(&head).y,
    })
}

/// The points tried, in order: past the header's last column when there is a header, then the status bar's centre.
pub fn tries(head: Option<RECT>, items: &[RECT], status: Option<RECT>, inset: i32) -> Vec<POINT> {
    let past = head.and_then(|h| header_spot(h, items, inset));
    past.into_iter().chain(status.map(|r| centre(&r))).collect()
}

/// True when a press may land where UI Automation finds `kind` and the window there is the app's, `on_app`.
pub fn safe(kind: Option<UIA_CONTROLTYPE_ID>, on_app: bool) -> bool {
    on_app && kind.is_some_and(|k| SAFE.contains(&k))
}

/// True when a press at `p` may land on window `hwnd` now, as UI Automation and the window there show.
pub fn ok_at(reader: &Reader, hwnd: HWND, p: POINT) -> bool {
    safe(g6uia::kind_at(reader.auto(), p), win::root_at(p) == hwnd)
}

/// The first point `tries` gives on `hwnd` where a press may land, `inset` past the last column; none when none shows.
pub fn find(reader: &Reader, hwnd: HWND, inset: i32) -> Option<POINT> {
    let first = |kind| g6uia::boxes(reader.auto(), hwnd, kind).first().copied();
    let head = first(UIA_HeaderControlTypeId);
    let items = head.map_or_else(Vec::new, |_| {
        g6uia::boxes(reader.auto(), hwnd, UIA_HeaderItemControlTypeId)
    });
    let status = first(UIA_StatusBarControlTypeId);
    let points = tries(head, &items, status, inset);
    points.into_iter().find(|&p| ok_at(reader, hwnd, p))
}

#[cfg(test)]
mod tests;
