//! The shown lists (design "List" and "By label"): the entries the filter and the search keep, By label's labels and a
//! label's values over them, in an order a fill leaves still while open, and each page's highlight kept inside its list.

use kx_module_api::ports::{Label, LabelCount};

use super::{Cursor, Page, Panel};
use crate::book::Notebook;
use crate::entry::{Entry, Field, Stamp};
use crate::find::Find;
use crate::order;

/// What a row or field of the last view named, so a click acts on what the user saw.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Target {
    /// An entry's row.
    Entry(u64),
    /// A label of By label.
    Label(Label),
    /// The value of an entry's field.
    Value(u64, usize),
    /// A field of the shown entry or draft.
    Field(usize),
    /// A label of the labels table, to add a field.
    Table(String),
}

/// What the last view showed: the place of its first row in the list, and each row.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Seen {
    pub(crate) start: usize,
    pub(crate) rows: Vec<Target>,
}

impl Panel {
    /// The entries the filter and the search keep, in the list's order; By label keeps every kind.
    pub(crate) fn listed<'b>(&self, book: &'b Notebook) -> Vec<&'b Entry> {
        let find = Find::new(self.search.text(), &self.names);
        order::listed(book.entries(), self.filter, &find, &self.held)
    }

    /// By label's labels and their counts, over the entries the search keeps.
    pub(crate) fn labels(&self, book: &Notebook) -> Vec<LabelCount> {
        order::labels(self.listed(book), &self.held)
    }

    /// Each value under `label` with its entry and field's place, over the entries the search keeps.
    pub(crate) fn values<'b>(&self, book: &'b Notebook, label: &Label) -> Vec<(&'b Entry, usize)> {
        let all = order::values(self.listed(book), label, &self.held);
        let place = |(e, f): (&'b Entry, &'b Field)| {
            let at = e.fields().iter().position(|g| std::ptr::eq(g, f));
            at.map(|i| (e, i))
        };
        all.into_iter().filter_map(place).collect()
    }

    /// Notes in `book` that entry `id` was filled at `now`; while open the list keeps the entry's place, and the next
    /// open shows the new order. False when the entry is gone.
    pub fn filled(&mut self, book: &mut Notebook, id: u64, now: Stamp) -> bool {
        let used = book.get(id).map(Entry::used);
        if let Some(used) = used.filter(|_| self.is_open()) {
            self.held.entry(id).or_insert(used);
        }
        book.use_at(id, now)
    }

    /// The list's highlight over the entries `ids`: on its entry while listed, else kept inside; `None` when empty.
    pub(crate) fn follow(&mut self, ids: &[u64]) -> Option<usize> {
        let same = self
            .list
            .id
            .and_then(|id| ids.iter().position(|&i| i == id));
        let at = inside(same.unwrap_or(self.list.at), ids.len());
        self.list = Cursor {
            at: at.unwrap_or_default(),
            id: at.map(|a| ids[a]),
        };
        at
    }

    /// The shown page's highlight kept inside its `len` rows; `None` when there is none.
    pub(crate) fn clamp(&mut self, len: usize) -> Option<usize> {
        let at = self.cursor()?;
        let kept = inside(*at, len);
        *at = kept.unwrap_or_default();
        kept
    }

    /// The shown page's highlight, none while closed; a backup page without its sheet shows the list, so the list's.
    pub(super) fn cursor(&mut self) -> Option<&mut usize> {
        match &mut self.page {
            Page::Closed => None,
            Page::List | Page::Backup => Some(&mut self.list.at),
            Page::Values { at, .. } => Some(at),
            Page::Entry(open) => Some(&mut open.at),
            Page::Edit(edit) => Some(edit.cursor()),
        }
    }
}

/// Place `at` kept inside `len` rows; `None` when there is none.
fn inside(at: usize, len: usize) -> Option<usize> {
    len.checked_sub(1).map(|end| at.min(end))
}
