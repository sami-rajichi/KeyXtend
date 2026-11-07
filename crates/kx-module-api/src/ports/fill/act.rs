//! The clicks the Quick-fill panel sends: a row's or a field's button by its place on the shown page, a filter, a
//! kind, a page turn, a panel button, an answer, the memory checks switch or the caret; and the label names it shows.

use super::GATE;
use super::view::{EntryKind, FillFilter};
use crate::{Capability, Event, Redacted};
use std::collections::BTreeMap;

listed! {
    /// A row's button.
    pub enum RowButton ("Every row button.") {
        /// Type the login's user.
        User,
        /// Type the login's password.
        Password,
        /// Type the user, a Tab, then the password, and never Enter.
        Both,
        /// Type the one value: an entry with one field, or a value under a label.
        Insert,
        /// Open it: an entry, a label, a place, a folder, a file or a found box.
        Open,
        /// Remove the place from the backup.
        Remove,
    }
}

listed! {
    /// A field's button on the entry, New or Edit page.
    pub enum FieldButton ("Every field button.") {
        /// Its value line, which then takes the letters.
        Value,
        /// Its own label line, which then takes the letters.
        Label,
        /// Type its value.
        Insert,
        /// Show its hidden value, or hide it again.
        Show,
        /// Switch whether it is hidden, on the New or Edit page.
        Hidden,
        /// Put the clipboard's text into it.
        Paste,
        /// Make a random password for it.
        Make,
    }
}

listed! {
    /// One of the panel's buttons outside its rows and fields.
    pub enum FillButton ("Every panel button.") {
        /// The header's New.
        New,
        /// The header's Backup.
        Backup,
        /// The header's close.
        Close,
        /// Back out of the shown page.
        Back,
        /// Edit the shown entry.
        Edit,
        /// The name line on New or Edit, which then takes the letters.
        Name,
        /// Add a field, from the labels table.
        AddField,
        /// Add a field with the user's own label.
        OwnLabel,
        /// Save the entry.
        Save,
        /// Leave New or Edit without saving.
        Cancel,
        /// Ask to delete the entry.
        Delete,
        /// Write every box now.
        BackUpNow,
        /// Add a place.
        AddPlace,
        /// Set a new backup password.
        ChangePassword,
        /// Bring entries back from a box.
        BringBack,
        /// Bring entries in from a `KeePass` file.
        Import,
        /// Another folder: open the folder list.
        OtherFolder,
        /// Use this folder: pick the shown folder.
        Here,
        /// Go on with the typed password.
        Done,
    }
}

/// What a click on the panel names.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FillClick {
    /// A row's button.
    Row {
        /// The row's place on the shown page.
        row: u8,
        /// The button clicked.
        button: RowButton,
    },
    /// A field's button.
    Field {
        /// The field's place on the shown page.
        field: u8,
        /// The button clicked.
        button: FieldButton,
    },
    /// Show this filter.
    Filter(FillFilter),
    /// Pick this kind on the New page.
    Kind(EntryKind),
    /// Turn a page, from the panel's arrows or a scroll pad page tap, which the ui turns into this.
    Page {
        /// Turn back.
        back: bool,
    },
    /// A panel button.
    Button(FillButton),
    /// Answer the shown question: yes on true.
    Answer(bool),
    /// Turn memory checks on, or off on false.
    Checks(bool),
    /// The ui's rows per page; the module pages by the last one heard.
    Rows(u8),
    /// A click placed the caret in the line that takes the letters.
    Caret {
        /// Where, in what the line draws: UTF-16 units of its text, or dots while masked; it tells a secret's length.
        at: Redacted<u32>,
    },
}

/// From the ui: each table label's shown text by its id, in the keyboard's language, so search and By label match it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FillNames(pub BTreeMap<String, String>);

impl Event for FillNames {
    const NAME: &'static str = "fill-names";
    const CAPABILITY: Option<Capability> = GATE;
}
