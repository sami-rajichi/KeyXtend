//! Moving the highlight (design "List", ADR-0008): a row up or down, or a whole page, on any page's list, stopping at
//! the ends. The page follows the highlight, and on the entry list the highlight follows its entry as the list moves.

use kx_module_api::ports::FillFilter;

use super::edit::Sub;
use super::{Do, Page, Panel};
use crate::book::Notebook;
use crate::kinds::tables;

/// The shown list as the highlight sees it.
pub(super) struct Held {
    /// The highlight's place, kept inside the list; `None` when it is empty.
    pub(super) at: Option<usize>,
    /// How many rows it has.
    pub(super) len: usize,
    /// The entries by row, on the entry list only.
    pub(super) ids: Vec<u64>,
}

impl Panel {
    /// The shown list, with its highlight kept inside it.
    pub(super) fn held(&mut self, book: &Notebook) -> Held {
        if matches!(self.page, Page::List) && self.filter != FillFilter::Labels {
            let ids: Vec<u64> = self.listed(book).iter().map(|e| e.id()).collect();
            let at = self.follow(&ids);
            return Held {
                at,
                len: ids.len(),
                ids,
            };
        }
        let len = self.len(book);
        Held {
            at: self.clamp(len),
            len,
            ids: Vec::new(),
        }
    }

    /// How many rows the shown list has, but the entry list's.
    fn len(&self, book: &Notebook) -> usize {
        match &self.page {
            Page::Closed | Page::Backup => 0,
            Page::List => self.labels(book).len(),
            Page::Values { label, .. } => self.values(book, label).len(),
            Page::Entry(open) => book.get(open.id).map_or(0, |e| e.fields().len()),
            Page::Edit(edit) => match edit.sub {
                Sub::Fields(_) => tables().labels().len(),
                Sub::Form | Sub::Asking => edit.slots.len(),
            },
        }
    }

    /// Moves the highlight a row down or up, stopping at the ends.
    pub(super) fn step(&mut self, down: bool, book: &Notebook) -> Do {
        let Held {
            at: Some(at),
            len,
            ids,
        } = self.held(book)
        else {
            return Do::Stay;
        };
        let to = if down {
            (at + 1).min(len - 1)
        } else {
            at.saturating_sub(1)
        };
        self.jump(at, to, &ids)
    }

    /// Turns to the first row of the page before or after, stopping at the ends.
    pub(super) fn turn(&mut self, back: bool, book: &Notebook) -> Do {
        let Held {
            at: Some(at),
            len,
            ids,
        } = self.held(book)
        else {
            return Do::Stay;
        };
        let rows = self.rows();
        let (page, end) = (at / rows, (len - 1) / rows);
        let to = if back {
            page.saturating_sub(1)
        } else {
            (page + 1).min(end)
        };
        if to == page {
            return Do::Stay;
        }
        self.jump(at, to * rows, &ids)
    }

    /// Highlights place `to` of the shown list when it is not `from`.
    fn jump(&mut self, from: usize, to: usize, ids: &[u64]) -> Do {
        if from == to {
            return Do::Stay;
        }
        self.put(to, ids);
        Do::Show
    }

    /// Highlights place `at` of the shown list: on the entry list its entry, on the form that field's first line.
    pub(super) fn put(&mut self, at: usize, ids: &[u64]) {
        if let Page::Edit(edit) = &mut self.page
            && edit.sub == Sub::Form
        {
            let _ = edit.focus(at, true) || edit.focus(at, false);
            return;
        }
        if matches!(self.page, Page::List) {
            self.list.id = ids.get(at).copied();
        }
        if let Some(cursor) = self.cursor() {
            *cursor = at;
        }
    }
}
