//! History: the shown page of texts, with the counts its header, Pin and Delete all need. Each text is wiped on drop.

use super::cloud::ServiceName;
use crate::ports::When;
use crate::{Redacted, Secret};

/// One text in History.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoryRow {
    /// The text's id, so the ui keeps a row's place as the list moves.
    pub id: u64,
    /// The text, wiped when the last view holding it drops.
    pub text: Secret,
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
    /// The cloud service that wrote it, which the row's cloud mark names; `None` for a model on this PC.
    pub cloud: Option<ServiceName>,
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
