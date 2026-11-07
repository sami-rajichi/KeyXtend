//! Keys from our keyboard (design "Keys"): ↑↓ move, ←→ work a line's caret or, on an empty search, the filters, Enter
//! presses the highlighted row's first button, Esc backs out (the page, the list's search, the panel), and the rest act
//! at the caret of the search where it shows, or of the highlighted line on New and Edit. A question takes only Esc (No).

use kx_module_api::caret::Side;
use kx_module_api::ports::{FillFilter, FillNote, Label, PageNote, PanelKey, RowButton};
use kx_module_api::text::SPACE;

use super::edit::{Editor, Sub, Tag};
use super::moves::Held;
use super::rows::buttons;
use super::{Cursor, Do, Fill, Open, Page, Panel, Part, rows};
use crate::book::Notebook;
use crate::entry::{Entry, Line};
use crate::kinds::tables;

impl Panel {
    /// Takes `key` from our keyboard; a key for an entry page whose entry left is swallowed.
    pub fn key(&mut self, key: &PanelKey, book: &Notebook) -> Do {
        if !self.is_open() {
            return Do::Stay;
        }
        if self.sync(book) {
            return Do::Show;
        }
        self.noted(|p| match &p.page {
            Page::Closed => Do::Stay,
            Page::List | Page::Values { .. } => p.list_key(key, book),
            Page::Entry(_) => p.entry_key(key, book),
            Page::Edit(_) => p.edit_key(key, book),
            Page::Backup => Self::backup_key(key),
        })
    }

    /// A key on a list page; a label's values draw no search, so letters there go nowhere.
    fn list_key(&mut self, key: &PanelKey, book: &Notebook) -> Do {
        let lined = !matches!(self.page, Page::Values { .. });
        match key {
            PanelKey::Tick
            | PanelKey::Erase
            | PanelKey::Delete
            | PanelKey::Text(_)
            | PanelKey::Word { .. }
                if !lined =>
            {
                Do::Stay
            }
            PanelKey::Up => self.step(false, book),
            PanelKey::Down => self.step(true, book),
            PanelKey::Left => self.list_arrow(Side::Left),
            PanelKey::Right => self.list_arrow(Side::Right),
            PanelKey::Enter => self.enter(book),
            PanelKey::Esc => self.esc(),
            PanelKey::Tick => self.search_with(|l| typed(l, [SPACE]).took),
            PanelKey::Erase => self.search_with(|l| l.pop().is_some()),
            PanelKey::Delete => self.search_with(Line::delete),
            PanelKey::Text(text) => self.search_with(|l| typed(l, text.expose().chars()).took),
            PanelKey::Word { erase, text } => self.search_with(|l| {
                let gone = (0..*erase).map_while(|_| l.pop()).count() > 0;
                typed(l, text.expose().chars()).took || gone
            }),
        }
    }

    /// A key on an entry page: letters go nowhere, as no line shows.
    fn entry_key(&mut self, key: &PanelKey, book: &Notebook) -> Do {
        match key {
            PanelKey::Up => self.step(false, book),
            PanelKey::Down => self.step(true, book),
            PanelKey::Enter => self.enter(book),
            PanelKey::Esc => self.back(),
            _ => Do::Stay,
        }
    }

    /// A key on New or Edit: the question takes Esc, the labels table moves and adds, the form types.
    fn edit_key(&mut self, key: &PanelKey, book: &Notebook) -> Do {
        let Page::Edit(edit) = &mut self.page else {
            return Do::Stay;
        };
        match (edit.sub, key) {
            (Sub::Asking | Sub::Fields(_), PanelKey::Esc) => self.back(),
            (Sub::Fields(_), PanelKey::Up) => self.step(false, book),
            (Sub::Fields(_), PanelKey::Down) => self.step(true, book),
            (Sub::Fields(at), PanelKey::Enter) => self.add_table(at),
            (Sub::Form, PanelKey::Up) => shown(edit.step(false)),
            (Sub::Form, PanelKey::Down) => shown(edit.step(true)),
            (Sub::Form, PanelKey::Left) => shown(edit.arrow(Side::Left)),
            (Sub::Form, PanelKey::Right) => shown(edit.arrow(Side::Right)),
            (Sub::Form, PanelKey::Tick) => self.write([SPACE]),
            (Sub::Form, PanelKey::Erase) => shown(edit.line().and_then(Line::pop).is_some()),
            (Sub::Form, PanelKey::Delete) => shown(edit.delete()),
            (Sub::Form, PanelKey::Text(text)) => self.write(text.expose().chars()),
            (Sub::Form, PanelKey::Esc) => self.cancel(),
            _ => Do::Stay,
        }
    }

    /// Types `text` at the highlighted line's caret; a refused character shows the Refused note.
    fn write(&mut self, text: impl IntoIterator<Item = char>) -> Do {
        let Page::Edit(edit) = &mut self.page else {
            return Do::Stay;
        };
        let Some(line) = edit.line() else {
            return Do::Stay;
        };
        let done = typed(line, text);
        if done.refused {
            self.note = Some(PageNote::plain(FillNote::Refused));
        }
        shown(done.took || done.refused)
    }

    /// Adds a field under the labels table's label at `at`.
    pub(super) fn add_table(&mut self, at: usize) -> Do {
        let id = tables().labels().get(at).map(|l| l.id.clone());
        id.map_or(Do::Stay, |id| self.add_field(Tag::Table(id)))
    }

    /// Adds a field under `tag`, or shows the Full note at `max_fields`.
    pub(super) fn add_field(&mut self, tag: Tag) -> Do {
        let most = self.cfg.limits.max_fields;
        let Page::Edit(edit) = &mut self.page else {
            return Do::Stay;
        };
        if !edit.add(tag, most) {
            edit.sub = Sub::Form;
            self.note = Some(PageNote::plain(FillNote::Full));
        }
        Do::Show
    }

    /// Esc on a list page: a label's values, which draw no search, step back at once; the list clears its search, else
    /// closes the panel.
    fn esc(&mut self) -> Do {
        if matches!(self.page, Page::Values { .. }) {
            return self.back();
        }
        if self.search.text().is_empty() {
            return self.shut();
        }
        let max = self.cfg.limits.field_chars;
        self.search_with(|l| {
            *l = Line::new(max);
            true
        })
    }

    /// Steps back one page: a label's values or an entry to the list, the labels table or the question to the form.
    /// It never leaves New or Edit, never clears the search and never closes, so it is safe from an older view.
    pub(super) fn back(&mut self) -> Do {
        match &mut self.page {
            Page::Values { .. } | Page::Entry(_) => self.page = Page::List,
            Page::Edit(edit) if edit.sub != Sub::Form => edit.sub = Sub::Form,
            _ => return Do::Stay,
        }
        Do::Show
    }

    /// Leaves New for the list, or Edit for its entry's page, saving nothing.
    pub(super) fn cancel(&mut self) -> Do {
        let Page::Edit(edit) = &self.page else {
            return Do::Stay;
        };
        self.page = edit.id.map_or(Page::List, |id| Page::Entry(Open::new(id)));
        Do::Show
    }

    /// Closes the panel.
    pub(super) fn shut(&mut self) -> Do {
        self.close();
        Do::Close
    }

    /// Opens a New page, unless New or Edit shows.
    pub(super) fn start_new(&mut self) -> Do {
        if matches!(self.page, Page::Edit(_)) {
            return Do::Stay;
        }
        let max = self.cfg.limits.field_chars;
        self.page = Page::Edit(Box::new(Editor::new(max)));
        Do::Show
    }

    /// Changes the search with `change`, which says whether it did; a new search starts at the top.
    fn search_with(&mut self, change: impl FnOnce(&mut Line) -> bool) -> Do {
        if !change(&mut self.search) {
            return Do::Stay;
        }
        self.top();
        Do::Show
    }

    /// Shows the list under `filter`, highlighting its top row.
    pub(super) fn set_filter(&mut self, filter: FillFilter) -> Do {
        if filter == self.filter && matches!(self.page, Page::List) {
            return Do::Stay;
        }
        (self.filter, self.page) = (filter, Page::List);
        self.top();
        Do::Show
    }

    /// Highlights the top row of the shown list.
    fn top(&mut self) {
        self.list = Cursor::default();
        if let Page::Values { at, .. } = &mut self.page {
            *at = 0;
        }
    }

    /// Enter: presses the highlighted row's first button, or the highlighted field's Insert.
    fn enter(&mut self, book: &Notebook) -> Do {
        let Held {
            at: Some(at), ids, ..
        } = self.held(book)
        else {
            return Do::Stay;
        };
        match &self.page {
            Page::List if self.filter == FillFilter::Labels => {
                let label = self.labels(book).into_iter().nth(at).map(|c| c.label);
                label.map_or(Do::Stay, |label| self.open_label(label))
            }
            Page::List => {
                let entry = ids.get(at).and_then(|&id| book.get(id));
                let first = entry.and_then(|e| buttons(e).first().copied());
                match (entry, first) {
                    (Some(e), Some(button)) => self.press(e, button),
                    _ => Do::Stay,
                }
            }
            Page::Values { label, .. } => {
                let value = self.values(book, label).get(at).map(|(e, i)| (e.id(), *i));
                value.map_or(Do::Stay, |(id, i)| one(id, i))
            }
            Page::Entry(open) => one(open.id, at),
            _ => Do::Stay,
        }
    }

    /// Presses `button` of `entry`'s row: Open shows its page, the rest ask their fill.
    pub(super) fn press(&mut self, entry: &Entry, button: RowButton) -> Do {
        if button == RowButton::Open {
            self.page = Page::Entry(Open::new(entry.id()));
            return Do::Show;
        }
        rows::fill(entry, button).map_or(Do::Stay, Do::Fill)
    }

    /// Shows the values under `label`.
    pub(super) fn open_label(&mut self, label: Label) -> Do {
        self.page = Page::Values { label, at: 0 };
        Do::Show
    }
}

/// The fill of entry `id`'s field at `i`.
pub(super) fn one(id: u64, i: usize) -> Do {
    Do::Fill(Fill {
        id,
        parts: vec![Part::Value(i)],
    })
}

/// Show when `changed`, else Stay.
pub(super) const fn shown(changed: bool) -> Do {
    if changed { Do::Show } else { Do::Stay }
}

/// What typing did: whether a character went in, and whether one was refused.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Typed {
    pub(super) took: bool,
    pub(super) refused: bool,
}

/// Types `text` into `line` letter by letter.
pub(super) fn typed(line: &mut Line, text: impl IntoIterator<Item = char>) -> Typed {
    let mut done = Typed::default();
    for c in text {
        match line.push(c) {
            Ok(()) => done.took = true,
            Err(_) => done.refused = true,
        }
    }
    done
}
