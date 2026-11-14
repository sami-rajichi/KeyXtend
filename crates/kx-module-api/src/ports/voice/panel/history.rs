//! History: the shown page of texts, with the counts its header, Pin and Delete all need.

use crate::Redacted;
use crate::ports::When;

/// One text in History.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoryRow {
    /// The text's id, so the ui keeps a row's place as the list moves.
    pub id: u64,
    /// The start of the text.
    pub text: Redacted<String>,
    /// It reads right to left.
    pub rtl: bool,
    /// It needs the Arabic font.
    pub arabic: bool,
    /// When it was dictated.
    pub when: When,
    /// Its language's ISO 639 code, such as `fr`.
    pub lang: String,
    /// The shown name of the model that wrote it, or the user's own file name.
    pub model: Redacted<String>,
    /// It is pinned, outside `history_max`.
    pub pinned: bool,
    /// Its place in the tick order, from 1, while ticked; Insert follows that order.
    pub tick: Option<u16>,
}

/// The History page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistorySheet {
    /// The shown page's rows.
    pub rows: Vec<HistoryRow>,
    /// How many texts it holds, as in 12 texts · newest first.
    pub total: u32,
    /// How many are unpinned, which Delete all deletes; at 0 it stays dimmed.
    pub unpinned: u32,
    /// Pin reads Unpin: every ticked text is pinned.
    pub unpin: bool,
    /// The Delete-all question shows, naming `unpinned`.
    pub confirm: bool,
}
