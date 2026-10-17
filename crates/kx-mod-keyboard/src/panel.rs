//! The language list of the List design: the board's arrows, Enter and Esc move its highlight, pick and close it
//! (spec §3.3), and it pages at the UI's page size with no scrolling (ADR-0008).

use crate::spec::{BoardSpec, Nav, RowItem};
use kx_module_api::ports::{LangLabel, LayoutId};
use serde::Serialize;

/// The icon on the current row, by its `icons.toml` key.
pub const CURRENT_ICON: &str = "lang.current";
/// The move that turns back a page, for the board's key and the list's page button alike.
pub const PAGE_BACK: Nav = Nav::Left;
/// The move that turns on a page.
pub const PAGE_ON: Nav = Nav::Right;

/// What a panel key did.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Moved {
    /// Nothing changed.
    Stay,
    /// The highlight moved.
    Hi,
    /// Row `n` was picked.
    Pick(usize),
    /// The panel closes.
    Close,
}

/// The open language list: which row is highlighted.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct LangList {
    hi: usize,
}

/// The page that holds the highlight, which the UI shows and its page buttons turn.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Page {
    /// The page, from 0.
    pub at: usize,
    /// How many pages there are, at least one.
    pub count: usize,
    /// A page lies before it.
    pub back: bool,
    /// A page lies after it.
    pub on: bool,
}

impl LangList {
    /// The list opened with row `at` highlighted, the current layout's.
    #[must_use]
    pub fn open(at: usize) -> Self {
        Self { hi: at }
    }

    /// The highlighted row of a list `len` rows long.
    #[must_use]
    pub fn hi(&self, len: usize) -> usize {
        self.hi.min(len.saturating_sub(1))
    }

    /// The page that holds the highlight of a list `len` rows long, shown `rows` to a page; a page size of 0 counts
    /// as 1. It is the only page maths: the UI and the pad read it from the state.
    #[must_use]
    pub fn page(&self, rows: usize, len: usize) -> Page {
        let rows = rows.max(1);
        let count = len.div_ceil(rows).max(1);
        let at = self.hi(len) / rows;
        Page {
            at,
            count,
            back: at > 0,
            on: at + 1 < count,
        }
    }

    /// Key `nav` on a list `len` rows long, shown `rows` to a page; a page size of 0 counts as 1.
    /// Up and Down move one row, Left and Right turn to the first row of the page before or after, as the mock-up
    /// pages; each stops at the ends.
    pub fn key(&mut self, nav: Nav, rows: usize, len: usize) -> Moved {
        let Some(last) = len.checked_sub(1) else {
            return if nav == Nav::Esc {
                Moved::Close
            } else {
                Moved::Stay
            };
        };
        let (page, hi) = (self.page(rows, len), self.hi(len));
        let rows = rows.max(1);
        let to = match nav {
            Nav::Up => hi.saturating_sub(1),
            Nav::Down => (hi + 1).min(last),
            PAGE_BACK => page.at.saturating_sub(1) * rows,
            PAGE_ON => (page.at + 1).min(page.count - 1) * rows,
            Nav::Enter => return Moved::Pick(hi),
            Nav::Esc => return Moved::Close,
        };
        self.hi = to;
        if to == hi { Moved::Stay } else { Moved::Hi }
    }
}

/// The board keys that drive an open panel, by id, in board order.
#[must_use]
pub fn navs(spec: &BoardSpec) -> Vec<(String, Nav)> {
    let named = spec.rows.iter().flatten().filter_map(|i| match i {
        RowItem::Key(k) => k.nav.map(|n| (k.id.clone(), n)),
        RowItem::Chars(_) => None,
    });
    named.collect()
}

/// One layout as the list shows it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LangRow {
    /// The badge, such as `EN` or `ع`.
    pub short: String,
    /// The language, then the layout; the UI joins them.
    pub name: Vec<String>,
    /// Written right to left.
    pub rtl: bool,
    /// Needs the Arabic font.
    pub arabic: bool,
    /// The layout shown now.
    pub current: bool,
    /// The current row's tick, by its `icons.toml` key.
    pub icon: Option<&'static str>,
}

/// The open language list: every layout in the list's order, the highlighted row and the page that shows.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ListView {
    /// Every layout; the UI shows the rows of `page`, padded to whole pages.
    pub rows: Vec<LangRow>,
    /// The highlighted row.
    pub hi: usize,
    /// The page that holds `hi`, at the UI's page size.
    pub page: Page,
    /// The board keys the UI sends as `panel_key` while the list is open.
    pub keys: Vec<String>,
    /// The board key that turns back a page, which the list's page button sends; `None` when the board has none.
    pub page_back: Option<String>,
    /// The board key that turns on a page; `None` when the board has none.
    pub page_on: Option<String>,
}

/// The list `open` over `labels`, with layout `cur` shown now, the board keys `keys` driving it and `rows` to a page.
#[must_use]
pub fn list_view(
    open: &LangList,
    labels: &[LangLabel],
    cur: LayoutId,
    keys: &[(String, Nav)],
    rows: usize,
) -> ListView {
    let key_of = |nav: Nav| {
        keys.iter()
            .find(|(_, n)| *n == nav)
            .map(|(id, _)| id.clone())
    };
    let row = |l: &LangLabel| {
        let current = l.id == cur;
        LangRow {
            short: l.short.clone(),
            name: l.name.clone(),
            rtl: l.rtl,
            arabic: l.arabic,
            current,
            icon: current.then_some(CURRENT_ICON),
        }
    };
    ListView {
        rows: labels.iter().map(row).collect(),
        hi: open.hi(labels.len()),
        page: open.page(rows, labels.len()),
        keys: keys.iter().map(|(id, _)| id.clone()).collect(),
        page_back: key_of(PAGE_BACK),
        page_on: key_of(PAGE_ON),
    }
}

#[cfg(test)]
mod tests;
