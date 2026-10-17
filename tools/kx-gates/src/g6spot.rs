//! Where G6 presses outside our windows on Explorer: inside the column header just right of every column, or the
//! status bar's centre, each only where UI Automation shows that part or a bare part of it, never a file, which
//! single-click-to-open opens, a column, whose header sorts, nor a button.
#![cfg(windows)]

use serde_json::{Value, json};
use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::UI::Accessibility::{
    UIA_ButtonControlTypeId, UIA_CONTROLTYPE_ID, UIA_DataItemControlTypeId, UIA_EditControlTypeId,
    UIA_GroupControlTypeId, UIA_HeaderControlTypeId, UIA_HeaderItemControlTypeId,
    UIA_HyperlinkControlTypeId, UIA_ListControlTypeId, UIA_ListItemControlTypeId,
    UIA_PaneControlTypeId, UIA_SplitButtonControlTypeId, UIA_StatusBarControlTypeId,
    UIA_TextControlTypeId, UIA_ThumbControlTypeId,
};

use crate::keymap::{Reader, centre};
use crate::{g6uia, win};

/// What may lie at a header spot besides the header itself: a container, so long as nothing in `NEVER` holds it.
const BARE: [UIA_CONTROLTYPE_ID; 3] = [
    UIA_ListControlTypeId,
    UIA_PaneControlTypeId,
    UIA_GroupControlTypeId,
];
/// What a press must never land in: a file, a column, or a control that acts on a click.
const NEVER: [UIA_CONTROLTYPE_ID; 8] = [
    UIA_ListItemControlTypeId,
    UIA_DataItemControlTypeId,
    UIA_HeaderItemControlTypeId,
    UIA_ButtonControlTypeId,
    UIA_SplitButtonControlTypeId,
    UIA_ThumbControlTypeId,
    UIA_HyperlinkControlTypeId,
    UIA_EditControlTypeId,
];

/// The part of Explorer's window a spot lies on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Part {
    /// The column header, past its last column.
    Header,
    /// The status bar.
    Status,
}

/// Where a press outside our windows lands, and the part UI Automation must still show there; none for no check.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spot {
    /// The screen point.
    pub at: POINT,
    /// The part it lies on.
    pub part: Option<Part>,
}

/// What UI Automation showed at a point tried: its control type, class and the window there; never a point or a name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tried {
    /// The part the point lies on.
    pub part: Part,
    /// The control type at the point.
    pub kind: Option<UIA_CONTROLTYPE_ID>,
    /// Its class name.
    pub class: String,
    /// The window at the point, by class and program.
    pub owner: String,
}

impl Tried {
    /// The try's note.
    pub fn json(&self) -> Value {
        let part = match self.part {
            Part::Header => "header",
            Part::Status => "status",
        };
        let kind = self.kind.map(|k| k.0);
        json!({ "part": part, "type": kind, "class": self.class, "owner": self.owner })
    }
}

/// True when boxes `a` and `b` share a pixel.
fn meets(a: &RECT, b: &RECT) -> bool {
    a.left < b.right && b.left < a.right && a.top < b.bottom && b.top < a.bottom
}

/// Where a press may land on column header `head`: `inset` right of every right edge of its `items`, those whose box
/// meets it, at mid height; none when no item reads or that point is past the header's end.
pub fn header_spot(head: RECT, items: &[RECT], inset: i32) -> Option<POINT> {
    let row = items.iter().filter(|r| meets(r, &head));
    let x = row.map(|r| r.right).max()?.saturating_add(inset);
    (x < head.right).then_some(POINT {
        x,
        y: centre(&head).y,
    })
}

/// The spots tried, in order: past the header's last column when there is a header, then the status bar's centre.
pub fn tries(head: Option<RECT>, items: &[RECT], status: Option<RECT>, inset: i32) -> Vec<Spot> {
    let past = head.and_then(|h| header_spot(h, items, inset));
    let at = |at, part| Spot {
        at,
        part: Some(part),
    };
    let past = past.map(|p| at(p, Part::Header));
    past.into_iter()
        .chain(status.map(|r| at(centre(&r), Part::Status)))
        .collect()
}

/// True when a press may land on `part` where UI Automation finds `chain`, the type there and those above it, and the
/// window there is the app's, `on_app`: on the header, it or a bare container; on the status bar, it or its own text.
pub fn safe(part: Part, chain: &[UIA_CONTROLTYPE_ID], on_app: bool) -> bool {
    let Some(&at) = chain.first() else {
        return false;
    };
    let ok = match part {
        Part::Header => {
            at == UIA_HeaderControlTypeId
                || (BARE.contains(&at) && !chain.iter().any(|k| NEVER.contains(k)))
        }
        Part::Status => {
            let above = chain.get(1).copied();
            at == UIA_StatusBarControlTypeId
                || (at == UIA_TextControlTypeId && above == Some(UIA_StatusBarControlTypeId))
        }
    };
    on_app && ok
}

/// What UI Automation and the window show at `p` on `part` now, and true when a press may land there on `hwnd`.
fn look(reader: &Reader, hwnd: HWND, p: POINT, part: Part) -> (Tried, bool) {
    let at = g6uia::chain_at(reader.auto(), p);
    let owner = win::root_at(p);
    let ok = at.whole && safe(part, &at.chain, owner == hwnd);
    let tried = Tried {
        part,
        kind: at.chain.first().copied(),
        class: at.class,
        owner: win::describe(owner),
    };
    (tried, ok)
}

/// Checks `spot` on window `hwnd`: none when a press may land there now, else what UI Automation showed there.
pub fn check(reader: &Reader, hwnd: HWND, spot: Spot) -> Option<Tried> {
    let part = spot.part?;
    let (tried, ok) = look(reader, hwnd, spot.at, part);
    (!ok).then_some(tried)
}

/// The first spot `tries` gives on `hwnd` where a press may land, `inset` past the last column; else what each try
/// showed.
pub fn find(reader: &Reader, hwnd: HWND, inset: i32) -> Result<Spot, Vec<Tried>> {
    let first = |kind| g6uia::boxes(reader.auto(), hwnd, kind).first().copied();
    let head = first(UIA_HeaderControlTypeId);
    let items = head.map_or_else(Vec::new, |_| {
        g6uia::boxes(reader.auto(), hwnd, UIA_HeaderItemControlTypeId)
    });
    let status = first(UIA_StatusBarControlTypeId);
    let mut tried = Vec::new();
    for spot in tries(head, &items, status, inset) {
        match check(reader, hwnd, spot) {
            None => return Ok(spot),
            Some(t) => tried.push(t),
        }
    }
    Err(tried)
}

#[cfg(test)]
mod tests;
