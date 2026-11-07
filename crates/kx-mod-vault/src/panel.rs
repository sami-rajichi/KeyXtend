//! The panel's state (design "The panel"): one page at a time over the filter, the search and the list's highlight.
//! Keys and clicks change it and say what the module does next; keys never change the notebook, and nothing here types,
//! as a fill is a request the module turns into `FillType`.

mod asks;
mod backs;
mod clicks;
mod edit;
mod form;
mod input;
mod line;
mod lists;
mod moves;
mod put;
mod rows;

pub(crate) use lists::{Seen, Target};

pub use asks::{Desk, Do, Fill, Part, Step};
pub(crate) use rows::glimpsed;
pub use rows::{buttons, fill};

use std::collections::BTreeSet;

use kx_module_api::ports::{
    FillAsk, FillClose, FillFilter, FillNote, FillOpen, FillPanel, Label, PageNote,
};
use kx_module_api::{Redacted, Secret, Zeroizing};

use crate::book::{Limits, Notebook};
use crate::entry::{Line, Stamp};
use crate::find::LabelNames;
use crate::order::Uses;
use crate::settings::VaultSettings;
pub(crate) use edit::{Editor, Sub, Tag, Taking};

/// What the panel needs from the settings and the ui; the module fills it.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PanelConfig {
    /// Rows a page shows, the ui's `page_rows` once it tells them; 0 counts as 1.
    pub rows: usize,
    /// The notebook's limits: a typed line and the search hold at most `field_chars` characters.
    pub limits: Limits,
    /// The characters of a password Make one makes.
    pub make_chars: usize,
}

impl PanelConfig {
    /// The panel's part of `settings`; the rows wait for the ui.
    #[must_use]
    pub fn of(settings: &VaultSettings) -> Self {
        Self {
            rows: 0,
            limits: settings.limits(),
            make_chars: settings.make_chars,
        }
    }
}

/// The list's highlight: its place, and the entry it follows as the list moves.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Cursor {
    pub(crate) at: usize,
    pub(crate) id: Option<u64>,
}

/// One entry's page: the highlighted field and the hidden fields shown, until the page closes or the entry changes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Open {
    pub(crate) id: u64,
    pub(crate) at: usize,
    pub(crate) shown: BTreeSet<usize>,
    /// The entry's change time as the page last showed it.
    changed: Option<Stamp>,
}

impl Open {
    /// Entry `id`'s page, its top field highlighted and nothing shown.
    fn new(id: u64) -> Self {
        Self {
            id,
            at: 0,
            shown: BTreeSet::new(),
            changed: None,
        }
    }
}

/// The page the panel shows.
#[derive(Debug)]
pub(crate) enum Page {
    /// It is closed.
    Closed,
    /// The filter's entries, or By label's labels.
    List,
    /// The values under one label, with the highlighted one.
    Values { label: Label, at: usize },
    /// One entry.
    Entry(Open),
    /// New or Edit.
    Edit(Box<Editor>),
    /// The backup pages, which the module draws.
    Backup,
}

/// The panel's state; it is closed until `open`.
#[derive(Debug)]
pub struct Panel {
    cfg: PanelConfig,
    names: LabelNames,
    page: Page,
    filter: FillFilter,
    search: Line,
    list: Cursor,
    note: Option<PageNote>,
    round: u32,
    seen: Seen,
    /// The last-used times the list keeps while open, so a fill moves no row.
    held: Uses,
}

impl Panel {
    /// A closed panel.
    #[must_use]
    pub fn new(cfg: PanelConfig) -> Self {
        Self {
            cfg,
            names: LabelNames::new(),
            page: Page::Closed,
            filter: FillFilter::All,
            search: Line::new(cfg.limits.field_chars),
            list: Cursor::default(),
            note: None,
            round: 0,
            seen: Seen::default(),
            held: Uses::new(),
        }
    }

    /// Opens it fresh on the list: every entry, no search, the top row highlighted; the rounds go on counting.
    pub fn open(&mut self) {
        self.reset(Page::List);
    }

    /// Closes it, and every shown value and typed line with it.
    pub fn close(&mut self) {
        self.reset(Page::Closed);
    }

    /// True while it shows.
    #[must_use]
    pub fn is_open(&self) -> bool {
        !matches!(self.page, Page::Closed)
    }

    /// Takes `FillOpen`: opens it fresh, or closes it when open; gives the state the module tells.
    pub fn toggle(&mut self, _: FillOpen) -> FillPanel {
        if self.is_open() {
            self.close();
        } else {
            self.open();
        }
        self.state(FillAsk)
    }

    /// Takes `FillClose`, sent as the clipboard panel opens: when open it closes as the close button closes it, hiding
    /// shown values and dropping New or Edit unsaved; it never opens, so a late one is harmless.
    pub fn close_for(&mut self, _: FillClose) -> Do {
        if !self.is_open() {
            return Do::Stay;
        }
        self.shut()
    }

    /// Answers `FillAsk`, from a keyboard that starts later, with whether it is open.
    #[must_use]
    pub fn state(&self, _: FillAsk) -> FillPanel {
        FillPanel {
            open: self.is_open(),
        }
    }

    /// Takes the table labels' texts in the keyboard's language, which search matches.
    pub fn set_names(&mut self, names: LabelNames) {
        self.names = names;
    }

    /// Shows `note` until the next change.
    pub fn say(&mut self, note: FillNote) {
        self.note = Some(PageNote::plain(note));
    }

    /// Shows `note`, with its numbers, until the next change.
    pub fn tell(&mut self, note: PageNote) {
        self.note = Some(note);
    }

    /// Rows a page shows, at least one.
    #[must_use]
    pub fn rows(&self) -> usize {
        self.cfg.rows.max(1)
    }

    /// The filter, the search line and the note.
    pub(crate) fn heading(&self) -> (FillFilter, &str, Option<PageNote>) {
        (self.filter, self.search.text(), self.note)
    }

    /// The page shown.
    pub(crate) fn page(&self) -> &Page {
        &self.page
    }

    /// Notes what a new view shows; gives that view's round.
    pub(crate) fn shown(&mut self, seen: Seen) -> u32 {
        self.round = self.round.wrapping_add(1);
        self.seen = seen;
        self.round
    }

    /// Closes the page of an entry `book` no longer holds, and hides what it showed once the entry changed, as Bring
    /// back may replace it under its id; true when a page closed, so the key or click meant for it is swallowed.
    pub(crate) fn sync(&mut self, book: &Notebook) -> bool {
        let Page::Entry(open) = &mut self.page else {
            return false;
        };
        let Some(entry) = book.get(open.id) else {
            self.page = Page::List;
            return true;
        };
        if open
            .changed
            .replace(entry.changed())
            .is_some_and(|c| c != entry.changed())
        {
            open.shown.clear();
        }
        let len = entry.fields().len();
        open.shown.retain(|&i| i < len);
        false
    }

    /// Runs `act`, keeping the note shown when it changed nothing.
    fn noted(&mut self, act: impl FnOnce(&mut Self) -> Do) -> Do {
        let old = self.note.take();
        let done = act(self);
        if done == Do::Stay && self.note.is_none() {
            self.note = old;
        }
        done
    }

    /// Starts afresh on `page`, keeping the config, the label names and the round.
    fn reset(&mut self, page: Page) {
        self.page = page;
        self.filter = FillFilter::All;
        self.search = Line::new(self.cfg.limits.field_chars);
        self.list = Cursor::default();
        self.note = None;
        self.seen = Seen::default();
        self.held.clear();
    }
}

/// A wiped copy of `text`.
pub(crate) fn copy(text: &str) -> Secret {
    Redacted::new(Zeroizing::new(text.to_owned()))
}

#[cfg(test)]
mod props;
#[cfg(test)]
pub(crate) mod rig;
#[cfg(test)]
mod tests;
