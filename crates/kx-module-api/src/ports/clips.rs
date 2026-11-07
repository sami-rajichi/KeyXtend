//! The clipboard history's bus events (ADR-0034 decision 2, ADR-0035): the panel's state and what carries copied
//! content or search text need `ReadClipboard`, Quick-fill's text hand-over too, the paste request needs
//! `WriteClipboard`, and the open toggle and the chord flags carry nothing.

mod view;

pub use view::{ClipKind, ClipPage, ClipRow, Filter, Pages, PanelNote, Preview, When};

use super::MAX_FIELD_CHARS;
use crate::{Capability, Event, Redacted, Secret};
use std::sync::Arc;

/// The gate of the panel's state and of the events that carry content or search text.
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

/// From the clipboard module: whether its panel is open, so the keyboard lights the key and sends panel keys. It turns
/// every key into `ClipKey`, so it needs `ReadClipboard` (ADR-0035).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ClipPanel {
    /// The panel is open.
    pub open: bool,
}

impl Event for ClipPanel {
    const NAME: &'static str = "clip-panel";
    const CAPABILITY: Option<Capability> = READ;
}

/// A key a panel, the clipboard's or Quick-fill's, takes from our keyboard while it is open.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PanelKey {
    /// Move the highlight up.
    Up,
    /// Move the highlight down.
    Down,
    /// The left arrow: it moves the caret while the line holds text; an empty search shows the filter before.
    Left,
    /// The right arrow: it moves the caret while the line holds text; an empty search shows the filter after.
    Right,
    /// The highlighted row's act: the clipboard inserts the ticked rows or the highlighted one, Quick-fill presses the
    /// row's first button.
    Enter,
    /// Back out one step: the clipboard closes its question, else the preview, else clears the search, else closes.
    Esc,
    /// The Space key, sent as this while any panel is open: a line with text, or Quick-fill's lines even when empty, takes
    /// a space at the caret; the clipboard's empty search ticks or unticks the highlighted row.
    Tick,
    /// The Backspace key: erase the character before the caret, in the search or Quick-fill's highlighted line.
    Erase,
    /// The Delete key: remove the grapheme after the caret, in the search or Quick-fill's highlighted line.
    Delete,
    /// Characters to insert at the caret of the search line, or of Quick-fill's highlighted line.
    Text(Redacted<String>),
    /// A chip's whole change to a search line at its caret, as one change; a page with no search ignores it.
    Word {
        /// The characters to erase before the caret first, stopping at the line's start.
        erase: u16,
        /// The characters to insert at the caret after them.
        text: Redacted<String>,
    },
}

/// From the keyboard: a key clicked while the panel is open, which goes to the panel instead of the app.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClipKey(pub PanelKey);

impl Event for ClipKey {
    const NAME: &'static str = "clip-key";
    const CAPABILITY: Option<Capability> = READ;
}

/// The line a panel's page shows, as its module tells the keyboard (ADR-0040); `Debug` never shows its text.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum PanelLine {
    /// No line shows, as on an entry page, a preview or a question.
    #[default]
    None,
    /// The search line, with its text up to the caret, as prediction reads what is before it.
    Search(Redacted<String>),
    /// A name, label, value or password line, whose text never leaves its module.
    Private,
}

/// From the clipboard module: the line its panel shows now, sent as it changes and on `HistoryAsk`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClipLine(pub PanelLine);

impl Event for ClipLine {
    const NAME: &'static str = "clip-line";
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
    /// A click placed the search line's caret.
    Caret {
        /// Where, in UTF-16 units of the text the line draws.
        at: Redacted<u32>,
    },
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

/// A CR LF is two characters until Paste turns it into one space, so a take reads twice a field's characters.
const SLACK: usize = 2;

/// From Quick-fill: its Paste asks the clipboard module for the clipboard's current text, and for its history to forget
/// that copy.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ClipTake {
    /// Names this request, so its answer names it too.
    pub round: u32,
    /// The most characters to read; a longer text is refused.
    pub max_chars: usize,
}

impl ClipTake {
    /// The take of round `round` for a field of `field_chars` characters, with room for each break to be two.
    #[must_use]
    pub const fn new(round: u32, field_chars: usize) -> Self {
        Self {
            round,
            max_chars: field_chars.saturating_mul(SLACK),
        }
    }

    /// The most characters the clipboard reads: `max_chars`, but never past the room of the longest field.
    #[must_use]
    pub const fn cap(&self) -> usize {
        let most = MAX_FIELD_CHARS.saturating_mul(SLACK);
        if self.max_chars < most {
            self.max_chars
        } else {
            most
        }
    }
}

impl Event for ClipTake {
    const NAME: &'static str = "clip-take";
    const CAPABILITY: Option<Capability> = READ;
}

/// What a take found on the clipboard.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClipText {
    /// The clipboard's text.
    Text(Secret),
    /// The clipboard holds no text, such as an image.
    Empty,
    /// The text is past the take's cap, so it was refused.
    TooLong,
    /// Another app held the clipboard, the system refused it, or its text could not be read.
    Busy,
}

/// From the clipboard module: what the take found. Its history forgets its newest entry only when the take read the
/// text in full and that entry is this copy; it is not for a copy left out of history (as a password manager's), one
/// refused as too big or under the disk floor, or one made while the keyboard was off.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClipTaken {
    /// The round of the request.
    pub round: u32,
    /// What the take found.
    pub text: ClipText,
}

impl Event for ClipTaken {
    const NAME: &'static str = "clip-taken";
    const CAPABILITY: Option<Capability> = READ;
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
