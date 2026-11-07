//! The page the Quick-fill panel draws (design "The panel"): one sheet at a time, with the filter, the search, the
//! paging, the highlight and a note. Every name, value and search stays redacted, and a hidden value is carried only
//! while it is shown.

use super::act::RowButton;
use super::backup::{BackupSheet, FolderSheet, FoundRow, PasswordSheet, Spot};
use super::field::{FieldLine, FieldValue, Label};
use super::note::PageNote;
use crate::Redacted;
use crate::ports::Pages;

listed! {
    /// What an entry is, for its icon and starter fields.
    pub enum EntryKind ("Every kind.") {
        /// A person: full name, email, phone, address.
        Person,
        /// A login: user, password, website.
        Login,
        /// A Wi-Fi network and its password.
        Wifi,
        /// An ID or membership card.
        Card,
        /// A note.
        Note,
        /// Anything else, with no starter fields.
        Other,
    }
}

listed! {
    /// Which entries the list shows.
    pub enum FillFilter ("Every filter, in the order the chips show and the arrow keys step through.") {
        /// Every entry, Other ones among them.
        All,
        /// People.
        People,
        /// Logins.
        Logins,
        /// Wi-Fi networks.
        Wifi,
        /// Cards.
        Cards,
        /// Notes.
        Notes,
        /// The labels in use, then the values under one.
        Labels,
    }
}

/// A row's hint line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Hint {
    /// Values to glimpse, a hidden one as the mask and never an empty one: "alex@example.com · ••••••••".
    Values(Vec<FieldValue>),
    /// How many fields the entry has, after its kind's name: "Person · 6 fields".
    Fields(u8),
}

/// One entry's row in the list; a click on the row itself does nothing, so a slip never types.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntryRow {
    /// The entry's id, so the ui keeps a row's place as the list moves.
    pub id: u64,
    /// Its kind, for its icon.
    pub kind: EntryKind,
    /// Its name.
    pub name: Redacted<String>,
    /// Its hint line.
    pub hint: Hint,
    /// Its buttons in order: User, Password and Both for a login, else Insert for one field; every row ends with Open.
    pub buttons: Vec<RowButton>,
}

/// A label in use and how many values it has (By label).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LabelCount {
    /// The label.
    pub label: Label,
    /// How many values carry it.
    pub count: u32,
}

/// One value under a label: its entry's name and the value, with Insert.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LabelValue {
    /// The entry's name.
    pub name: Redacted<String>,
    /// The value, or the mask.
    pub value: FieldValue,
}

/// One entry's page: each field with Insert, and Show for a hidden one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntrySheet {
    /// Its kind.
    pub kind: EntryKind,
    /// Its name.
    pub name: Redacted<String>,
    /// The shown page's fields.
    pub fields: Vec<FieldLine>,
}

/// What takes the letters clicked on our keyboard, on the New or Edit page.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Typing {
    /// The entry's name.
    Name,
    /// The own label of the field at this place on the shown page.
    Label(u8),
    /// The value of the field at this place on the shown page.
    Value(u8),
}

/// The New or Edit page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditSheet {
    /// New, which shows the kind chips; else Edit, which offers Delete entry.
    pub new: bool,
    /// The kind, once picked; New shows only its chips until then.
    pub kind: Option<EntryKind>,
    /// The name typed so far.
    pub name: Redacted<String>,
    /// The shown page's fields.
    pub fields: Vec<FieldLine>,
    /// What takes the letters, if anything.
    pub typing: Option<Typing>,
    /// That line's caret, in what it draws: UTF-16 units of its text, or dots while masked. It may tell a secret's
    /// length, so never log it.
    pub caret: Redacted<u32>,
    /// The Delete-entry question shows.
    pub confirm: bool,
}

/// The page the panel shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Sheet {
    /// The entries under the filter and the search.
    List(Vec<EntryRow>),
    /// The labels in use (By label).
    Labels(Vec<LabelCount>),
    /// Every value under one label, across entries.
    Values {
        /// The label.
        label: Label,
        /// The shown page's values.
        rows: Vec<LabelValue>,
    },
    /// One entry.
    Entry(EntrySheet),
    /// New or Edit.
    Edit(EditSheet),
    /// The labels table, to add a field; Your own label is a button.
    AddField(Vec<Label>),
    /// The backup page.
    Backup(BackupSheet),
    /// The suggested places, to pick one; Another folder is a button.
    Places(Vec<Spot>),
    /// The folder list.
    Folders(FolderSheet),
    /// A password to type.
    Password(PasswordSheet),
    /// The boxes found, newest first, to bring one back.
    Found(Vec<FoundRow>),
    /// A box from another computer in a picked place, with the question whether to bring its entries in.
    Foreign(FoundRow),
    /// The same question about a box whose date the system does not tell.
    ForeignUndated(Spot),
}

/// Everything the panel draws.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FillPage {
    /// Counts each new view, so a click names the view it saw.
    pub round: u32,
    /// The page shown.
    pub sheet: Sheet,
    /// The filter, which the list pages' chips show.
    pub filter: FillFilter,
    /// The search line.
    pub search: Redacted<String>,
    /// The search line's caret, in UTF-16 units of the text it draws.
    pub caret: Redacted<u32>,
    /// Where the shown list is.
    pub pages: Pages,
    /// The highlighted row's place on the page.
    pub highlight: Option<u8>,
    /// A note to show, if any.
    pub note: Option<PageNote>,
    /// A place the backup waits for, which the footer names.
    pub waiting: Option<Spot>,
    /// The notebook holds no entry, so the list invites a first one.
    pub empty: bool,
}
