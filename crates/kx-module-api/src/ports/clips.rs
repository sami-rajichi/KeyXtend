//! The clipboard history's bus events (ADR-0034 decisions 1 and 2): what carries copied content or search text needs
//! `ReadClipboard`, the paste request needs `WriteClipboard`, and the toggles and the chord request carry nothing.

mod view;

pub use view::{ClipKind, ClipPage, ClipRow, Filter, Pages, PanelNote, Preview, When};

use crate::{Capability, Event, Redacted};
use std::sync::Arc;

/// The gate of the events that carry content or search text.
const READ: Option<Capability> = Some(Capability::ReadClipboard);
/// The gate of the paste request and its answer.
const WRITE: Option<Capability> = Some(Capability::WriteClipboard);

/// Bytes in a megabyte, as Windows counts sizes: the one unit the history's limits and the panel's sizes use.
pub const MB: u64 = 1 << 20;

/// From the keyboard: the side key or the history chord toggles the panel.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ClipOpen;

impl Event for ClipOpen {
    const NAME: &'static str = "clip-open";
}

/// From the clipboard module: whether its panel is open, so the keyboard lights the key and sends panel keys.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ClipPanel {
    /// The panel is open.
    pub open: bool,
}

impl Event for ClipPanel {
    const NAME: &'static str = "clip-panel";
}

/// A key the panel takes from our keyboard while it is open.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PanelKey {
    /// Move the highlight up.
    Up,
    /// Move the highlight down.
    Down,
    /// The filter before.
    Left,
    /// The filter after.
    Right,
    /// Insert the ticked rows, or the highlighted one when none is ticked.
    Enter,
    /// Back out one step: close the Delete-all question, else the preview, else clear the search, else the panel.
    Esc,
    /// Tick or untick the highlighted row: the Space key.
    Tick,
    /// Erase the search's last character: the Backspace key.
    Erase,
    /// Characters for the search line.
    Text(Redacted<String>),
}

/// From the keyboard: a key clicked while the panel is open, which goes to the panel instead of the app.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClipKey(pub PanelKey);

impl Event for ClipKey {
    const NAME: &'static str = "clip-key";
    const CAPABILITY: Option<Capability> = READ;
}

/// From the clipboard module: the panel to draw, shared, so every subscriber clones it cheaply.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClipView(pub Arc<ClipPage>);

impl ClipView {
    /// Shares `page`.
    #[must_use]
    pub fn new(page: ClipPage) -> Self {
        Self(Arc::new(page))
    }
}

impl Event for ClipView {
    const NAME: &'static str = "clip-view";
    const CAPABILITY: Option<Capability> = READ;
}

/// From the ui: a click on the panel; the actions act on the ticked rows, or on the previewed entry while the preview
/// page is open.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ClipAct {
    /// Tick or untick a row of a shown page.
    Tick {
        /// The round of the page clicked.
        round: u32,
        /// The row's place on that page.
        row: u8,
    },
    /// Show this filter.
    Filter(Filter),
    /// Turn a page, from the panel's arrows or a scroll pad page tap, which the ui turns into this.
    Page {
        /// Turn back.
        back: bool,
    },
    /// Insert the ticked rows in tick order, or the previewed entry.
    Insert,
    /// Pin the ticked rows or the previewed entry, or unpin them when all are pinned.
    Pin,
    /// Delete the ticked rows or the previewed entry at once.
    Delete,
    /// Ask to delete every unpinned entry.
    DeleteAll,
    /// Answer the Delete-all question: yes on true.
    Confirm(bool),
    /// Open the preview page.
    Preview,
    /// Leave the preview page.
    Back,
    /// Close the panel.
    Close,
    /// The ui's rows per page; the module pages by the last one heard.
    Rows(u8),
}

impl Event for ClipAct {
    const NAME: &'static str = "clip-act";
    const CAPABILITY: Option<Capability> = READ;
}

/// From the clipboard module: the clipboard is set, so the keyboard lets its latches go and sends one paste.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PasteAsk {
    /// Names this request, so its answer names it too.
    pub round: u32,
}

impl Event for PasteAsk {
    const NAME: &'static str = "paste-ask";
    const CAPABILITY: Option<Capability> = WRITE;
}

/// From the keyboard: the paste a `PasteAsk` asked for went out, or not.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Pasted {
    /// The round of the request.
    pub round: u32,
    /// The paste went out.
    pub ok: bool,
}

impl Event for Pasted {
    const NAME: &'static str = "pasted";
    const CAPABILITY: Option<Capability> = WRITE;
}

/// From the clipboard module: while `on`, the keyboard turns the platform's history chord into `ClipOpen`; it carries
/// nothing, so it needs no capability.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct HistoryWanted {
    /// Take the chord.
    pub on: bool,
}

impl Event for HistoryWanted {
    const NAME: &'static str = "history-wanted";
}

/// Asks the clipboard module to send `HistoryWanted` again, for a keyboard that starts later; it carries nothing, so
/// it needs no capability.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct HistoryAsk;

impl Event for HistoryAsk {
    const NAME: &'static str = "history-ask";
}

#[cfg(test)]
mod tests;
