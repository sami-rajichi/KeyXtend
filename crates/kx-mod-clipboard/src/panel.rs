//! The panel's state (spec §7 "Panel"): the filter, the search, the highlight, the ticks in tick order, the preview
//! and the Delete-all question. Keys and clicks change it and say what the module does next; it reads no disk.

mod input;

use kx_module_api::Redacted;
use kx_module_api::ports::{Filter, Preview};

use crate::entry::Entry;
use crate::find::Find;
use crate::order::History;

/// What the panel needs from the settings; the module fills it.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PanelConfig {
    /// Rows a page shows, the ui's `page_rows`; 0 counts as 1.
    pub rows: usize,
    /// The most characters the search line keeps (`search_chars`), as a longer query matches nothing.
    pub query_chars: usize,
}

/// What the module does after a key or a click.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Do {
    /// Nothing changed.
    Stay,
    /// The panel changed, so it is drawn again.
    Show,
    /// Load this entry's content and give it to `Panel::show`.
    Preview(u64),
    /// Insert these entries in this order.
    Insert(Vec<u64>),
    /// Pin these entries.
    Pin(Vec<u64>),
    /// Unpin these entries.
    Unpin(Vec<u64>),
    /// Delete these entries for good.
    Delete(Vec<u64>),
    /// The panel closed.
    Close,
}

/// Which page the panel shows; the Delete-all question may stand over either.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Mode {
    /// It is closed.
    Closed,
    /// The list shows.
    List,
    /// The preview page of this entry shows, with its content.
    Preview(u64, Preview),
}

/// The panel's state; it is closed until `open`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Panel {
    /// The page size and the search's length.
    cfg: PanelConfig,
    /// The page it shows.
    mode: Mode,
    /// The kinds the list shows.
    filter: Filter,
    /// The search line as typed.
    query: Redacted<String>,
    /// The highlighted row's place in the list, kept inside it when read.
    hi: usize,
    /// The highlighted entry, which the highlight follows as the list moves.
    hi_id: Option<u64>,
    /// The ticked entries, in tick order.
    ticks: Vec<u64>,
    /// The entries the Delete-all question counted, while it shows.
    asked: Option<Vec<u64>>,
    /// The round of the last view built.
    round: u32,
    /// The entries that view showed, by row.
    shown: Vec<u64>,
}

impl Panel {
    /// A closed panel.
    #[must_use]
    pub fn new(cfg: PanelConfig) -> Self {
        Self {
            cfg,
            mode: Mode::Closed,
            filter: Filter::All,
            query: Redacted::default(),
            hi: 0,
            hi_id: None,
            ticks: Vec::new(),
            asked: None,
            round: 0,
            shown: Vec::new(),
        }
    }

    /// Opens it fresh: every kind, no search, no tick, the top row highlighted; the rounds go on counting.
    pub fn open(&mut self) {
        self.reset(Mode::List);
    }

    /// Closes it and forgets its search, ticks, preview and question.
    pub fn close(&mut self) {
        self.reset(Mode::Closed);
    }

    /// True while it shows.
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.mode != Mode::Closed
    }

    /// The kinds the list shows.
    #[must_use]
    pub fn filter(&self) -> Filter {
        self.filter
    }

    /// The search line as typed.
    #[must_use]
    pub fn query(&self) -> &Redacted<String> {
        &self.query
    }

    /// The ticked entries, in tick order.
    #[must_use]
    pub fn ticks(&self) -> &[u64] {
        &self.ticks
    }

    /// The previewed entry and its content, while the preview page shows.
    #[must_use]
    pub fn preview(&self) -> Option<(u64, &Preview)> {
        match &self.mode {
            Mode::Preview(id, p) => Some((*id, p)),
            Mode::Closed | Mode::List => None,
        }
    }

    /// True while the Delete-all question shows.
    #[must_use]
    pub fn asking(&self) -> bool {
        self.asked.is_some()
    }

    /// The entries the Delete-all question counted and Yes deletes, while it shows.
    #[must_use]
    pub fn asked(&self) -> Option<&[u64]> {
        self.asked.as_deref()
    }

    /// Rows a page shows, at least one.
    #[must_use]
    pub fn rows(&self) -> usize {
        self.cfg.rows.max(1)
    }

    /// The entries of `history` the list shows, in order.
    #[must_use]
    pub fn list<'h>(&self, history: &'h History) -> Vec<&'h Entry> {
        Find::new(self.filter, self.query.expose()).list(history.entries())
    }

    /// The highlighted row's place in `list`, on its entry while listed, else kept inside the list; `None` when empty.
    pub fn highlight(&mut self, list: &[&Entry]) -> Option<usize> {
        let same = self
            .hi_id
            .and_then(|id| list.iter().position(|e| e.id == id));
        let at = same.unwrap_or(self.hi).min(list.len().saturating_sub(1));
        self.go(at, list);
        self.hi_id.map(|_| at)
    }

    /// The entries the footer acts on: the previewed one while the preview shows, else the ticked ones.
    #[must_use]
    pub fn targets(&self) -> Vec<u64> {
        match self.preview() {
            Some((id, _)) => vec![id],
            None => self.ticks.clone(),
        }
    }

    /// True when Pin reads Unpin: every entry the footer acts on is pinned, and there is one.
    #[must_use]
    pub fn unpins(&self, history: &History) -> bool {
        let targets = self.targets();
        let pinned = |id: &u64| history.get(*id).is_some_and(Entry::pinned);
        !targets.is_empty() && targets.iter().all(pinned)
    }

    /// Opens the preview page of `id` with its content, while the panel shows.
    pub fn show(&mut self, id: u64, preview: Preview) {
        if self.is_open() {
            self.mode = Mode::Preview(id, preview);
        }
    }

    /// Puts `preview` in the preview page when it shows `id`, as when the large image is drawn; true when it did.
    pub fn redraw(&mut self, id: u64, preview: Preview) -> bool {
        let shows = self.preview().is_some_and(|(at, _)| at == id);
        if shows {
            self.mode = Mode::Preview(id, preview);
        }
        shows
    }

    /// Drops the ticks of entries `history` no longer holds and the preview page of one; the question forgets those
    /// gone or pinned since, and closes when none is left.
    pub fn sync(&mut self, history: &History) {
        self.ticks.retain(|id| history.get(*id).is_some());
        if self
            .preview()
            .is_some_and(|(id, _)| history.get(id).is_none())
        {
            self.mode = Mode::List;
        }
        if let Some(ids) = &mut self.asked {
            ids.retain(|id| history.get(*id).is_some_and(|e| !e.pinned()));
        }
        if self.asked.as_ref().is_some_and(Vec::is_empty) {
            self.asked = None;
        }
    }

    /// Syncs with `history`; true when that closed the question or the preview, so the key or click meant for them is
    /// swallowed rather than reaching the list.
    pub(crate) fn resync(&mut self, history: &History) -> bool {
        let (asked, peeked) = (self.asking(), self.preview().is_some());
        self.sync(history);
        (asked && !self.asking()) || (peeked && self.preview().is_none())
    }

    /// Notes the entries a new view shows by row; gives that view's round.
    pub(crate) fn shown(&mut self, ids: Vec<u64>) -> u32 {
        self.round = self.round.wrapping_add(1);
        self.shown = ids;
        self.round
    }

    /// Starts afresh in `mode`, keeping the page size and the round.
    fn reset(&mut self, mode: Mode) {
        let round = self.round;
        *self = Self {
            mode,
            round,
            ..Self::new(self.cfg)
        };
    }

    /// Highlights place `at` of `list`, or nothing in an empty list.
    fn go(&mut self, at: usize, list: &[&Entry]) {
        self.hi = at;
        self.hi_id = list.get(at).map(|e| e.id);
    }

    /// Highlights the top row, as a new filter or search does.
    fn top(&mut self) {
        (self.hi, self.hi_id) = (0, None);
    }

    /// Keeps the highlight on its place in the list, not on its entry, as a pin or a delete moves rows; the page
    /// stays.
    fn hold(&mut self, history: &History) {
        let list = self.list(history);
        self.highlight(&list);
        self.hi_id = None;
    }
}

/// Every unpinned entry of `history`, which the Delete-all question counts; Yes deletes the ids counted when asked,
/// minus those gone or pinned since.
#[must_use]
pub fn unpinned(history: &History) -> Vec<u64> {
    let free = history.entries().iter().filter(|e| !e.pinned());
    free.map(|e| e.id).collect()
}

#[cfg(test)]
mod props;
#[cfg(test)]
pub(crate) mod rig;
#[cfg(test)]
mod tests;
