//! Quick-fill's icons, by role (design "The panel", "The backup box" and "Import").

use serde::{Deserialize, Serialize};

/// Quick-fill: its header, pages, kinds, row and field buttons, the backup's waiting line, and the backup and import
/// pages.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FillIcons {
    /// The title, which the empty notebook's invitation shares.
    pub title: String,
    /// The search line.
    pub search: String,
    /// New.
    pub new: String,
    /// Backup.
    pub backup: String,
    /// Close.
    pub close: String,
    /// Back.
    pub back: String,
    /// Edit.
    pub edit: String,
    /// Save.
    pub save: String,
    /// Cancel and the question's No.
    pub cancel: String,
    /// Delete entry and the question's Yes.
    pub delete: String,
    /// Add a field.
    pub add_field: String,
    /// Your own label.
    pub own_label: String,
    /// The shown filter's mark.
    pub shown: String,
    /// A person.
    pub kind_person: String,
    /// A login.
    pub kind_login: String,
    /// A Wi-Fi network.
    pub kind_wifi: String,
    /// A card.
    pub kind_card: String,
    /// A note.
    pub kind_note: String,
    /// Any other entry.
    pub kind_other: String,
    /// A label's row and tile.
    pub label: String,
    /// A login's User.
    pub user: String,
    /// A login's Password.
    pub password: String,
    /// A login's Both, which types a Tab between them.
    pub both: String,
    /// Insert.
    pub insert: String,
    /// Open.
    pub open: String,
    /// Show a hidden value.
    pub show: String,
    /// Hide it again.
    pub hide: String,
    /// The hidden switch while on.
    pub hidden_on: String,
    /// The hidden switch while off.
    pub hidden_off: String,
    /// Paste.
    pub paste: String,
    /// Make one.
    pub make: String,
    /// The footer's line while the backup waits for a place.
    pub waiting: String,
    /// Remove place.
    pub remove: String,
    /// Choose, on a place or a file to pick.
    pub pick: String,
    /// A cloud folder.
    pub place_cloud: String,
    /// A removable drive.
    pub place_removable: String,
    /// A drive.
    pub place_drive: String,
    /// A folder.
    pub place_folder: String,
    /// A file to pick.
    pub place_file: String,
    /// Back up now.
    pub back_up_now: String,
    /// Add place.
    pub add_place: String,
    /// Change password.
    pub change_password: String,
    /// The memory checks switch while on.
    pub checks_on: String,
    /// The memory checks switch while off.
    pub checks_off: String,
    /// Bring back.
    pub bring_back: String,
    /// Import from `KeePass`.
    pub keepass: String,
    /// Another folder.
    pub other_folder: String,
    /// Use this folder.
    pub here: String,
    /// Done.
    pub done: String,
    /// A backup password's page.
    pub lock: String,
}

impl FillIcons {
    /// Every name with where it is used, such as `fill.make`.
    pub(super) fn named(&self) -> impl Iterator<Item = (&'static str, &str)> + '_ {
        self.notebook().into_iter().chain(self.safe_keeping())
    }

    /// The notebook's names: its header, pages, kinds, and row and field buttons.
    fn notebook(&self) -> [(&'static str, &str); 32] {
        [
            ("fill.title", &self.title),
            ("fill.search", &self.search),
            ("fill.new", &self.new),
            ("fill.backup", &self.backup),
            ("fill.close", &self.close),
            ("fill.back", &self.back),
            ("fill.edit", &self.edit),
            ("fill.save", &self.save),
            ("fill.cancel", &self.cancel),
            ("fill.delete", &self.delete),
            ("fill.add_field", &self.add_field),
            ("fill.own_label", &self.own_label),
            ("fill.shown", &self.shown),
            ("fill.kind_person", &self.kind_person),
            ("fill.kind_login", &self.kind_login),
            ("fill.kind_wifi", &self.kind_wifi),
            ("fill.kind_card", &self.kind_card),
            ("fill.kind_note", &self.kind_note),
            ("fill.kind_other", &self.kind_other),
            ("fill.label", &self.label),
            ("fill.user", &self.user),
            ("fill.password", &self.password),
            ("fill.both", &self.both),
            ("fill.insert", &self.insert),
            ("fill.open", &self.open),
            ("fill.show", &self.show),
            ("fill.hide", &self.hide),
            ("fill.hidden_on", &self.hidden_on),
            ("fill.hidden_off", &self.hidden_off),
            ("fill.paste", &self.paste),
            ("fill.make", &self.make),
            ("fill.waiting", &self.waiting),
        ]
    }

    /// The backup and import pages' names.
    fn safe_keeping(&self) -> [(&'static str, &str); 18] {
        [
            ("fill.remove", &self.remove),
            ("fill.pick", &self.pick),
            ("fill.place_cloud", &self.place_cloud),
            ("fill.place_removable", &self.place_removable),
            ("fill.place_drive", &self.place_drive),
            ("fill.place_folder", &self.place_folder),
            ("fill.place_file", &self.place_file),
            ("fill.back_up_now", &self.back_up_now),
            ("fill.add_place", &self.add_place),
            ("fill.change_password", &self.change_password),
            ("fill.checks_on", &self.checks_on),
            ("fill.checks_off", &self.checks_off),
            ("fill.bring_back", &self.bring_back),
            ("fill.keepass", &self.keepass),
            ("fill.other_folder", &self.other_folder),
            ("fill.here", &self.here),
            ("fill.done", &self.done),
            ("fill.lock", &self.lock),
        ]
    }
}
