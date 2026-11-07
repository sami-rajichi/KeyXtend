//! What a key or a click asks of the module: nothing, a draw, a close, a fill, a save, a paste or a backup page step;
//! and what a click may change, the notebook among it.

use kx_module_api::ports::{FillClick, PanelKey, Random};

use crate::book::Notebook;
use crate::entry::Stamp;

/// One part of a fill request.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Part {
    /// The value of the entry's field at this place.
    Value(usize),
    /// One Tab, as between a user and a password.
    Tab,
}

/// A fill the user asked for: an entry's values by their places, which the module types through `FillType`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fill {
    /// The entry.
    pub id: u64,
    /// What to type, in order.
    pub parts: Vec<Part>,
}

/// What the module does after a key or a click.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Do {
    /// Nothing changed.
    Stay,
    /// The panel changed, so it is drawn again.
    Show,
    /// The panel closed, so the module tells `FillPanel { open: false }`.
    Close,
    /// Type this fill; the panel stays open with the highlight on what was filled, so it is drawn again.
    Fill(Fill),
    /// The notebook changed, so it is saved, then the panel is drawn again.
    Saved,
    /// Put the clipboard's text into the highlighted field, which is a value line now.
    Paste,
    /// The backup pages take this step; the module draws them.
    Backup(Step),
}

/// What the backup pages take from the panel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Step {
    /// The header's Backup opens them.
    Open,
    /// The invitation's Import opens them on its folder list.
    Import,
    /// A click on the view the user saw, which the panel checked.
    Click(FillClick),
    /// A key from our keyboard.
    Key(PanelKey),
}

/// What a click may change: the notebook, the random source for ids and Make one, and the wall time of a save.
pub struct Desk<'a> {
    /// The notebook.
    pub book: &'a mut Notebook,
    /// The random source.
    pub random: &'a dyn Random,
    /// The wall time now.
    pub now: Stamp,
}
