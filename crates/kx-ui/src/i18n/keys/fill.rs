//! Quick-fill's translation keys and ids (design "The panel"): a group's key takes an id, as in `fill.kind-login`,
//! and a button's label is `fill.` then its id. Each match makes a new kind, filter, button or note name its id.

use kx_module_api::ports::{
    EntryKind, FieldButton, FillButton, FillFilter, FillNote, PasswordAsk, Pick, PlaceKind,
    RowButton,
};

/// A button's label, and the start of every Quick-fill key: this, then the button's id.
pub const FILL_WORD: &str = "fill.";
/// The panel's title.
pub const FILL_TITLE: &str = "fill.title";
/// The search line's hint while it is empty.
pub const FILL_SEARCH: &str = "fill.search";
/// The empty notebook's invitation.
pub const FILL_EMPTY: &str = "fill.empty";
/// A list the filter and the search leave empty.
pub const FILL_NONE: &str = "fill.none";
/// What a hidden value shows while it is not shown.
pub const FILL_MASK: &str = "fill.mask";
/// What joins a row's glimpsed values.
pub const FILL_JOIN: &str = "fill.join";
/// A row's hint with no glimpse: its kind's name `$kind` and `$count` fields, plural by `$form`.
pub const FILL_FIELDS: &str = "fill.fields";
/// A label's count of values: `$count`, plural by `$form`.
pub const FILL_VALUES: &str = "fill.values";
/// The footer while the backup waits for `$place`.
pub const FILL_WAITING: &str = "fill.waiting";
/// Show's label while the value shows.
pub const FILL_HIDE: &str = "fill.hide";
/// Open's label on a place or a file to pick.
pub const FILL_PICK: &str = "fill.pick";
/// Open's label on a table label to add.
pub const FILL_ADD: &str = "fill.add";
/// The New page's title.
pub const FILL_NEW_TITLE: &str = "fill.new-title";
/// The Edit page's title.
pub const FILL_EDIT_TITLE: &str = "fill.edit-title";
/// What New asks over its kind chips.
pub const FILL_KINDS: &str = "fill.kinds";
/// Delete entry's question.
pub const FILL_CONFIRM: &str = "fill.confirm";
/// The memory checks switch.
pub const FILL_CHECKS: &str = "fill.checks";
/// The backup page with no place yet.
pub const FILL_NO_PLACES: &str = "fill.no-places";
/// A place's box written today, at `$time`.
pub const FILL_UPDATED_TODAY: &str = "fill.updated-today";
/// A place's box written yesterday, at `$time`.
pub const FILL_UPDATED_YESTERDAY: &str = "fill.updated-yesterday";
/// A place's box written on `$date`.
pub const FILL_UPDATED_DATE: &str = "fill.updated-date";
/// Remove place's question on its row, naming `$place`.
pub const FILL_REMOVE_PLACE: &str = "fill.remove-place";
/// Back up now's question while the notebook did not read: replace the backup?
pub const FILL_REPLACE: &str = "fill.replace";
/// A place whose box waits for it to come back.
pub const FILL_PLACE_WAITING: &str = "fill.place-waiting";
/// The suggested places' heading.
pub const FILL_PLACES: &str = "fill.places";
/// The folder list's heading at the drives.
pub const FILL_DRIVES: &str = "fill.drives";
/// A found box: its `$date` and `$place`.
pub const FILL_FOUND: &str = "fill.found";
/// A box from another computer in `$place`, from `$date`.
pub const FILL_FOREIGN: &str = "fill.foreign";
/// The same box, when the system does not tell its date.
pub const FILL_FOREIGN_UNDATED: &str = "fill.foreign-undated";
/// A kind's name: this, then its `entry_kind_id`.
pub const FILL_KIND: &str = "fill.kind-";
/// A table label's text: this, then its id in the vault's labels table.
pub const FILL_LABEL: &str = "fill.label-";
/// A filter chip: this, then its `fill_filter_id`.
pub const FILL_FILTER: &str = "fill.filter-";
/// A note: this, then its `fill_note_id`.
pub const FILL_NOTE: &str = "fill.note-";
/// The folder list's heading: this, then what it picks, by `pick_id`.
pub const FILL_FOLDERS: &str = "fill.folders-";
/// The password page's text: this, then its `ask_id`; some take `$count` with `$form`.
pub const FILL_ASK: &str = "fill.password-";

/// A kind's id, as the vault's kinds table names it; QML picks the row's icon by it.
#[must_use]
pub const fn entry_kind_id(kind: EntryKind) -> &'static str {
    match kind {
        EntryKind::Person => "person",
        EntryKind::Login => "login",
        EntryKind::Wifi => "wifi",
        EntryKind::Card => "card",
        EntryKind::Note => "note",
        EntryKind::Other => "other",
    }
}

/// A filter's id, as the vault's kinds table names the chip that lists a kind.
#[must_use]
pub const fn fill_filter_id(filter: FillFilter) -> &'static str {
    match filter {
        FillFilter::All => "all",
        FillFilter::People => "people",
        FillFilter::Logins => "logins",
        FillFilter::Wifi => "wifi",
        FillFilter::Cards => "cards",
        FillFilter::Notes => "notes",
        FillFilter::Labels => "labels",
    }
}

/// A row button's id, which QML sends back.
#[must_use]
pub const fn row_button_id(button: RowButton) -> &'static str {
    match button {
        RowButton::User => "user",
        RowButton::Password => "password",
        RowButton::Both => "both",
        RowButton::Insert => "insert",
        RowButton::Open => "open",
        RowButton::Remove => "remove",
    }
}

/// A field button's id, which QML sends back.
#[must_use]
pub const fn field_button_id(button: FieldButton) -> &'static str {
    match button {
        FieldButton::Value => "value",
        FieldButton::Label => "label",
        FieldButton::Insert => "insert",
        FieldButton::Show => "show",
        FieldButton::Hidden => "hidden",
        FieldButton::Paste => "paste",
        FieldButton::Make => "make",
    }
}

/// A panel button's id, which QML sends back.
#[must_use]
pub const fn fill_button_id(button: FillButton) -> &'static str {
    match button {
        FillButton::New => "new",
        FillButton::Backup => "backup",
        FillButton::Close => "close",
        FillButton::Back => "back",
        FillButton::Edit => "edit",
        FillButton::Name => "name",
        FillButton::AddField => "add-field",
        FillButton::OwnLabel => "own-label",
        FillButton::Save => "save",
        FillButton::Cancel => "cancel",
        FillButton::Delete => "delete",
        FillButton::BackUpNow => "back-up-now",
        FillButton::AddPlace => "add-place",
        FillButton::ChangePassword => "change-password",
        FillButton::BringBack => "bring-back",
        FillButton::Import => "import",
        FillButton::OtherFolder => "other-folder",
        FillButton::Here => "here",
        FillButton::Done => "done",
    }
}

/// A note's id.
#[must_use]
pub const fn fill_note_id(note: FillNote) -> &'static str {
    match note {
        FillNote::Typed => "typed",
        FillNote::NotTyped => "not-finished",
        FillNote::NoText => "paste-empty",
        FillNote::ClipOff => "paste-off",
        FillNote::ClipBusy => "paste-busy",
        FillNote::WrongPassword => "wrong-password",
        FillNote::Damaged => "damaged",
        FillNote::Unsupported => "unsupported",
        FillNote::TooShort => "too-short",
        FillNote::Refused => "refused",
        FillNote::Full => "full",
        FillNote::Imported => "imported",
        FillNote::Remembered => "remembered",
        FillNote::TriesUsed => "tries-used",
        FillNote::Unread => "unread",
        FillNote::FileLocked => "file-locked",
        FillNote::FileTooLarge => "file-too-large",
        FillNote::FileUnread => "file-unread",
        FillNote::DataFolder => "data-folder",
        FillNote::ReadOnly => "read-only",
        FillNote::TooLarge => "too-large",
        FillNote::BoxFailed => "box-failed",
        FillNote::Paused => "paused",
        FillNote::SaveFailed => "save-failed",
    }
}

/// A place's kind id; QML picks its icon by it.
#[must_use]
pub const fn place_kind_id(kind: PlaceKind) -> &'static str {
    match kind {
        PlaceKind::Cloud => "cloud",
        PlaceKind::Removable => "removable",
        PlaceKind::Drive => "drive",
        PlaceKind::Folder => "folder",
        PlaceKind::File => "file",
    }
}

/// What the folder list picks, by id.
#[must_use]
pub const fn pick_id(pick: Pick) -> &'static str {
    match pick {
        Pick::Folder => "folder",
        Pick::Backup => "backup",
        Pick::Keepass => "keepass",
    }
}

/// What a password page is for, by id; bringing a box back to a computer with its own padlock says so.
#[must_use]
pub const fn ask_id(ask: PasswordAsk) -> &'static str {
    match ask {
        PasswordAsk::New { .. } => "new",
        PasswordAsk::Check { .. } => "check",
        PasswordAsk::Open { own: false } => "open",
        PasswordAsk::Open { own: true } => "own",
        PasswordAsk::Foreign => "foreign",
        PasswordAsk::Keepass => "keepass",
    }
}
