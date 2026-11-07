//! The page the panel draws (design "The panel"): the shown page's rows, fields or labels, with the filter, the
//! search, the pages and the highlight. It holds no words, as the ui words it; a hidden value with text goes in only
//! while shown, and each view counts a new round, so a click names the view it saw.

use kx_module_api::Redacted;
use kx_module_api::ports::{
    EditSheet, EntryRow, EntrySheet, FieldLine, FieldValue, FillFilter, FillNote, FillPage,
    FillView, Hint, Label, LabelValue, PageNote, Pages, Sheet, Spot, Typing,
};

use crate::book::Notebook;
use crate::entry::{Entry, Field};
use crate::kinds::tables;
use crate::panel::{
    Editor, Open, Page, Panel, Seen, Sub, Tag, Taking, Target, buttons, copy, glimpsed,
};

/// One page of a list: where it starts and ends, where it is, and its highlighted row.
pub(crate) struct Window {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pages: Pages,
    highlight: Option<u8>,
}

impl Window {
    /// The page of `len` rows, `rows` a page, that holds place `at`, highlighting it when `lit`.
    pub(crate) fn of(at: Option<usize>, len: usize, rows: usize, lit: bool) -> Self {
        let page = at.unwrap_or_default() / rows;
        let start = (page * rows).min(len);
        let highlight = at.filter(|_| lit).map(|a| small(a - start));
        Self {
            start,
            end: (start + rows).min(len),
            pages: Pages {
                at: short(page),
                count: short(len.div_ceil(rows).max(1)),
            },
            highlight,
        }
    }

    /// What the window shows by row, each named by `target`.
    fn seen(&self, target: impl Fn(usize) -> Target) -> Seen {
        let rows = (self.start..self.end).map(target).collect();
        Seen {
            start: self.start,
            rows,
        }
    }
}

/// A sheet with its window and what it shows by row.
type Drawn = (Sheet, Window, Seen);

/// What the actor adds to a view: the backup's page while it shows, the place the footer names while it waits, the hold
/// and a failed save.
#[derive(Default)]
pub(crate) struct Backdrop {
    pub(crate) paged: Option<Paged>,
    pub(crate) waiting: Option<Spot>,
    /// The boxes are held, so the note line says Paused when no other note shows.
    pub(crate) paused: bool,
    /// A save failed, so the note line says so when no other note shows, ahead of Paused.
    pub(crate) failed: bool,
}

/// A backup page: its sheet, its window and its note.
pub(crate) struct Paged {
    pub(crate) sheet: Sheet,
    pub(crate) window: Window,
    pub(crate) note: Option<PageNote>,
}

/// The view of `panel`'s page over `book`, counting a new round.
pub fn build(panel: &mut Panel, book: &Notebook) -> FillView {
    draw(panel, book, Backdrop::default())
}

/// The view of `panel`'s page over `book` with what the backup adds, counting a new round.
pub(crate) fn draw(panel: &mut Panel, book: &Notebook, back: Backdrop) -> FillView {
    panel.sync(book);
    let mut note = panel.heading().2;
    let (sheet, window, seen) = match back.paged.filter(|_| panel.on_backup()) {
        Some(paged) => {
            note = paged.note;
            (paged.sheet, paged.window, Seen::default())
        }
        None => page(panel, book),
    };
    let note = note
        .or(back.failed.then(|| PageNote::plain(FillNote::SaveFailed)))
        .or(back.paused.then(|| PageNote::plain(FillNote::Paused)));
    let round = panel.shown(seen);
    let (filter, search, _) = panel.heading();
    FillView::new(FillPage {
        round,
        sheet,
        filter,
        search: Redacted::new(search.to_owned()),
        caret: Redacted::new(panel.search_caret()),
        pages: window.pages,
        highlight: window.highlight,
        note,
        waiting: back.waiting,
        empty: book.is_empty(),
    })
}

/// The panel's own page; the backup pages without their sheet show the list.
fn page(panel: &mut Panel, book: &Notebook) -> Drawn {
    let rows = panel.rows();
    match panel.page() {
        Page::Closed | Page::List | Page::Backup if panel.heading().0 == FillFilter::Labels => {
            labels(panel, book, rows)
        }
        Page::Closed | Page::List | Page::Backup => list(panel, book, rows),
        Page::Values { label, .. } => {
            let label = label.clone();
            values(panel, book, label, rows)
        }
        Page::Entry(_) => entry(panel, book, rows),
        Page::Edit(_) => edit(panel, rows),
    }
}

/// The filter's entries.
fn list(panel: &mut Panel, book: &Notebook, rows: usize) -> Drawn {
    let entries = panel.listed(book);
    let ids: Vec<u64> = entries.iter().map(|e| e.id()).collect();
    let w = Window::of(panel.follow(&ids), entries.len(), rows, true);
    let shown = &entries[w.start..w.end];
    let seen = w.seen(|at| Target::Entry(ids[at]));
    (Sheet::List(shown.iter().map(|e| row(e)).collect()), w, seen)
}

/// By label's labels with their counts.
fn labels(panel: &mut Panel, book: &Notebook, rows: usize) -> Drawn {
    let all = panel.labels(book);
    let w = Window::of(panel.clamp(all.len()), all.len(), rows, true);
    let seen = w.seen(|at| Target::Label(all[at].label.clone()));
    (Sheet::Labels(all[w.start..w.end].to_vec()), w, seen)
}

/// The values under `label`, each with its entry's name.
fn values(panel: &mut Panel, book: &Notebook, label: Label, rows: usize) -> Drawn {
    let all = panel.values(book, &label);
    let w = Window::of(panel.clamp(all.len()), all.len(), rows, true);
    let seen = w.seen(|at| Target::Value(all[at].0.id(), all[at].1));
    let shown = all[w.start..w.end].iter().map(|(e, i)| LabelValue {
        name: e.name().clone(),
        value: read(&e.fields()[*i], false),
    });
    let rows = shown.collect();
    (Sheet::Values { label, rows }, w, seen)
}

/// The shown entry's page; `build` synced it, so the entry is there.
fn entry(panel: &mut Panel, book: &Notebook, rows: usize) -> Drawn {
    let Page::Entry(Open { id, shown, .. }) = panel.page() else {
        return list(panel, book, rows);
    };
    let (id, shown) = (*id, shown.clone());
    let Some(entry) = book.get(id) else {
        return list(panel, book, rows);
    };
    let len = entry.fields().len();
    let w = Window::of(panel.clamp(len), len, rows, true);
    let line = |i: usize| FieldLine {
        label: entry.fields()[i].label.clone(),
        value: read(&entry.fields()[i], shown.contains(&i)),
        make: false,
    };
    let sheet = EntrySheet {
        kind: entry.kind(),
        name: entry.name().clone(),
        fields: (w.start..w.end).map(line).collect(),
    };
    let seen = w.seen(Target::Field);
    (Sheet::Entry(sheet), w, seen)
}

/// New or Edit, or its labels table.
fn edit(panel: &mut Panel, rows: usize) -> Drawn {
    let len = match panel.page() {
        Page::Edit(e) if matches!(e.sub, Sub::Fields(_)) => tables().labels().len(),
        Page::Edit(e) => e.slots.len(),
        _ => 0,
    };
    let at = panel.clamp(len);
    let Page::Edit(e) = panel.page() else {
        let w = Window::of(None, 0, rows, false);
        return (Sheet::List(Vec::new()), w, Seen::default());
    };
    if matches!(e.sub, Sub::Fields(_)) {
        let w = Window::of(at, len, rows, true);
        let ids = tables().labels();
        let seen = w.seen(|i| Target::Table(ids[i].id.clone()));
        let shown = ids[w.start..w.end]
            .iter()
            .map(|l| Label::Table(l.id.clone()));
        return (Sheet::AddField(shown.collect()), w, seen);
    }
    let w = Window::of(at, len, rows, e.taking != Taking::Name);
    let seen = w.seen(Target::Field);
    (Sheet::Edit(form(e, &w)), w, seen)
}

/// The form's sheet over window `w`.
fn form(e: &Editor, w: &Window) -> EditSheet {
    let line = |i: usize| {
        let slot = &e.slots[i];
        let label = match &slot.tag {
            Tag::Table(id) => Label::Table(id.clone()),
            Tag::Own(own) => Label::Own(Redacted::new(own.text().to_owned())),
        };
        FieldLine {
            label,
            value: edited(slot.value.text(), slot.hidden, e.shown.contains(&i)),
            make: e.makes(i),
        }
    };
    let place = small(e.at.saturating_sub(w.start));
    let typing = e.kind.map(|_| match e.taking {
        Taking::Name => Typing::Name,
        Taking::Label => Typing::Label(place),
        Taking::Value => Typing::Value(place),
    });
    EditSheet {
        new: e.id.is_none(),
        kind: e.kind,
        name: Redacted::new(e.name.text().to_owned()),
        fields: (w.start..w.end).map(line).collect(),
        typing,
        caret: Redacted::new(e.typed_caret()),
        confirm: e.sub == Sub::Asking,
    }
}

/// `entry`'s row: its kind, name, hint and buttons.
fn row(entry: &Entry) -> EntryRow {
    let shown = glimpsed(entry);
    let hint = if shown.is_empty() {
        Hint::Fields(small(entry.fields().len()))
    } else {
        let fields = entry.fields();
        Hint::Values(shown.iter().map(|&i| read(&fields[i], false)).collect())
    };
    EntryRow {
        id: entry.id(),
        kind: entry.kind(),
        name: entry.name().clone(),
        hint,
        buttons: buttons(entry),
    }
}

/// `field`'s value as a read page shows it, copied only when it shows, so a masked value is never copied.
fn read(field: &Field, shown: bool) -> FieldValue {
    let text: &str = field.value.expose();
    if shows(text, field.hidden, shown) {
        FieldValue::new(copy(text), field.hidden, shown)
    } else {
        FieldValue::masked()
    }
}

/// `text` as the form shows it, copied only when it shows; masked it gives only its count of characters.
fn edited(text: &str, hidden: bool, shown: bool) -> FieldValue {
    if shows(text, hidden, shown) {
        FieldValue::edit(copy(text), hidden, shown)
    } else {
        FieldValue::masked_edit(text.chars().count())
    }
}

/// True when `text`, `hidden` or not and `shown` or not, goes into the page; an empty one has nothing to mask.
const fn shows(text: &str, hidden: bool, shown: bool) -> bool {
    !hidden || shown || text.is_empty()
}

/// `n` as a count the page holds, or the largest one.
fn short(n: usize) -> u16 {
    u16::try_from(n).unwrap_or(u16::MAX)
}

/// `n` as a place or count of one page, or the largest one.
pub(crate) fn small(n: usize) -> u8 {
    u8::try_from(n).unwrap_or(u8::MAX)
}

#[cfg(test)]
mod tests;
