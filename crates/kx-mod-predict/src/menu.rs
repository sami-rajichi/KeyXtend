//! The chip menu's state (spec §6 "Controls"): forget mode, the Undo chip until the next key, and the open email
//! list's page and rows, as the UI tells them.

/// The open email list: its shown page and the rows the UI shows per page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct List {
    /// The shown page.
    pub(crate) page: u16,
    /// Rows per page, at least one.
    pub(crate) rows: u8,
}

impl List {
    /// The place in the whole list of `row` on the shown page; `None` for a row past the page, which no tap can reach.
    pub(crate) fn at(self, row: u8) -> Option<usize> {
        (row < self.rows)
            .then(|| usize::from(self.page) * usize::from(self.rows) + usize::from(row))
    }
}

/// The menu's state; it starts with nothing open.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Menu {
    /// The next word chip tapped is forgotten.
    pub(crate) forgetting: bool,
    /// The Undo chip's place in the lane, from a forget until the next key.
    pub(crate) undo: Option<usize>,
    /// The email list, while it is open.
    pub(crate) list: Option<List>,
}

impl Menu {
    /// Forget mode starts; the list closes.
    pub(crate) fn forget(&mut self) {
        self.forgetting = true;
        self.list = None;
    }

    /// Forget mode ends and the list closes.
    pub(crate) fn cancel(&mut self) {
        self.forgetting = false;
        self.list = None;
    }

    /// The chip at `at` was forgotten, so the Undo chip takes its place; `None` when nothing was forgotten.
    pub(crate) fn forgot(&mut self, at: Option<usize>) {
        self.forgetting = false;
        self.undo = at;
    }

    /// Takes the Undo chip away; true when it was shown.
    pub(crate) fn undo(&mut self) -> bool {
        self.undo.take().is_some()
    }

    /// A key, a click elsewhere or a new box: the Undo chip goes and the list closes.
    pub(crate) fn key(&mut self) {
        self.undo = None;
        self.list = None;
    }

    /// Opens the list at its first page, `rows` to a page; forget mode ends.
    pub(crate) fn open(&mut self, rows: u8) {
        self.forgetting = false;
        self.list = Some(List {
            page: 0,
            rows: rows.max(1),
        });
    }

    /// Turns the open list by `by` pages, `rows` to a page, within the pages `count` emails fill.
    pub(crate) fn turn(&mut self, by: i8, rows: u8, count: usize) {
        let Some(list) = self.list.as_mut() else {
            return;
        };
        let rows = rows.max(1);
        let pages = count.div_ceil(usize::from(rows)).max(1);
        let end = i64::try_from(pages - 1).unwrap_or(i64::MAX);
        let page = (i64::from(list.page) + i64::from(by)).clamp(0, end);
        list.page = u16::try_from(page).unwrap_or(u16::MAX);
        list.rows = rows;
    }

    /// Everything closes, as when prediction turns off.
    pub(crate) fn off(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests;
