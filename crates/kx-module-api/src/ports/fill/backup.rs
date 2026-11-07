//! The backup and import pages (design "The backup box" and "Import"): places by kind and short name, never by path,
//! each box's state and date, the folder list, and the password page, whose text shows while typed (owner).

use crate::ports::{PlaceKind, When};
use crate::{Redacted, Secret};

/// A place as a page names it: its kind, for the icon, and its short name, never its path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spot {
    /// What it is.
    pub kind: PlaceKind,
    /// Its short name, such as `OneDrive` or a drive's letter and label.
    pub name: Redacted<String>,
}

/// The box's state in one place.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BoxState {
    /// Written then: "Updated today 10:42".
    Updated(When),
    /// Waiting for the place to come back, or to have room.
    Waiting,
}

/// One place on the backup page, with Remove place.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaceRow {
    /// The place.
    pub place: Spot,
    /// Its box's state.
    pub state: BoxState,
}

/// A box found, or one from another computer: where it is and when it was written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FoundRow {
    /// Its place.
    pub place: Spot,
    /// When it was last written.
    pub when: When,
}

/// The backup page: each place's state, the memory checks switch, and buttons for the rest.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BackupSheet {
    /// The shown page's places.
    pub places: Vec<PlaceRow>,
    /// Memory checks are on.
    pub checks: bool,
    /// The question that shows, if any; `FillClick::Answer` answers it.
    pub ask: Option<BackupAsk>,
}

/// The one question the backup page asks at a time.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BackupAsk {
    /// Remove place, on the shown place row it names.
    Remove(u8),
    /// Back up now while the notebook did not read: replace the backup, which may hold the entries it lost?
    Replace,
}

listed! {
    /// What the folder list picks.
    pub enum Pick ("Every pick.") {
        /// A folder for the box, with Use this folder.
        Folder,
        /// A box to bring back.
        Backup,
        /// A `KeePass` file to bring in.
        Keepass,
    }
}

/// The folder list: the drives, then a folder's subfolders, and the files it picks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FolderSheet {
    /// What it picks.
    pub pick: Pick,
    /// The shown folder's name, or `None` at the drives.
    pub folder: Option<Redacted<String>>,
    /// The shown page's drives, folders and files.
    pub rows: Vec<Spot>,
}

/// What a typed password is for.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PasswordAsk {
    /// A new backup password, at setup or a change.
    New {
        /// The fewest characters it takes (`box_min_chars`).
        min: u8,
    },
    /// A memory check.
    Check {
        /// Tries left before a new password is set.
        tries: u8,
    },
    /// A found box's password, to bring it back.
    Open {
        /// This computer keeps its own padlock, so the box brought back is rewritten under it.
        own: bool,
    },
    /// The password of a box from another computer, to bring its entries in.
    Foreign,
    /// A `KeePass` file's password.
    Keepass,
}

/// The password page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PasswordSheet {
    /// What the password is for.
    pub ask: PasswordAsk,
    /// What is typed so far.
    pub text: Secret,
    /// Its caret, in UTF-16 units of the text it draws; it tells a secret's length, so never log it.
    pub caret: Redacted<u32>,
}
