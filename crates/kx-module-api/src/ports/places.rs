//! Places for the backup box (ADR-0038 decision 7): the cloud folders and removable drives an adapter's table
//! suggests, the folder list, the boxes found in the suggested places, and writes in one step. It needs `VaultData`,
//! and no path or name prints in `Debug`.

use super::{BOX_EXT, BOX_MARK};
use crate::{Capability, Redacted, ServiceId, ServiceKey};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// What a box's file name holds, in any case, for a find to take it.
pub const BOX_HINT: &str = "keyxtend";

listed! {
    /// What a place is, for its icon.
    pub enum PlaceKind ("Every kind.") {
        /// A cloud app's folder, such as `OneDrive`.
        Cloud,
        /// A removable drive, such as a USB stick.
        Removable,
        /// Another drive, at the top of the folder list.
        Drive,
        /// A folder in the folder list.
        Folder,
        /// A file in the folder list, to pick a box or a `KeePass` file.
        File,
    }
}

/// A place the box may live in, or a drive, folder or file in the folder list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Place {
    /// What it is.
    pub kind: PlaceKind,
    /// Its short name to show, such as `OneDrive` or a drive's letter and label.
    pub name: Redacted<String>,
    /// Where it is.
    pub path: Redacted<PathBuf>,
}

/// A box found in a suggested place.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Found {
    /// The suggested place it was found in.
    pub place: Place,
    /// The box's file.
    pub path: Redacted<PathBuf>,
    /// When it was last written, in ms since the Unix epoch.
    pub modified_ms: i64,
}

/// Whether a place can take the box now.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PlaceState {
    /// It is there with room for the box.
    Ready,
    /// It is gone, as a stick unplugged or a folder removed, or cannot be read.
    Missing,
    /// It has less room than the box needs.
    Full,
    /// It holds or lies in the data folder, which a format wipes, so no box may go there.
    Refused,
}

/// Why a place could not be listed, read or written.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum PlaceError {
    /// The folder or its drive is gone.
    #[error("the place is gone")]
    Missing,
    /// The drive has no room for the box.
    #[error("the place is full")]
    Full,
    /// The system refused to write there.
    #[error("the place cannot be written")]
    ReadOnly,
    /// It holds or lies in the data folder, which a format wipes.
    #[error("the place holds or lies in the data folder")]
    Refused,
    /// The file is over the read's cap.
    #[error("the file is too large")]
    TooLarge,
    /// Another error of the file system.
    #[error("the place could not be read or written")]
    Io,
}

/// True for a box's file name: it holds `BOX_HINT` and ends with `.` and `BOX_EXT`, each in any case.
#[must_use]
pub fn is_box_name(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    let stem = name.strip_suffix(BOX_EXT).and_then(|s| s.strip_suffix('.'));
    stem.is_some_and(|s| s.contains(BOX_HINT))
}

/// True when a file's first bytes `head` start with the box's mark.
#[must_use]
pub fn is_box_head(head: &[u8]) -> bool {
    head.starts_with(&BOX_MARK)
}

/// Finds, lists, reads and writes the places the box may live in; the platform provides it.
pub trait Places: Send + Sync {
    /// The cloud folders found on this computer, in the table's order, then the removable drives.
    fn suggested(&self) -> Vec<Place>;

    /// The drives when `folder` is `None`, else its subfolders A to Z, without hidden ones or links.
    ///
    /// # Errors
    /// `Missing` when the folder is gone, `Io` when it cannot be listed.
    fn folders(&self, folder: Option<&Path>) -> Result<Vec<Place>, PlaceError>;

    /// The files in `folder` whose extension is `ext` in any case, A to Z, to pick a box or a `KeePass` file.
    ///
    /// # Errors
    /// `Missing` when the folder is gone, `Io` when it cannot be listed.
    fn files(&self, folder: &Path, ext: &str) -> Result<Vec<Place>, PlaceError>;

    /// The boxes in each suggested place's top level and one level down, by `is_box_name` and `is_box_head`, newest
    /// first.
    fn find(&self) -> Vec<Found>;

    /// When `file` was last written, in ms since the Unix epoch; `None` when it is gone or the system does not tell.
    fn modified(&self, file: &Path) -> Option<i64>;

    /// The bytes of `file`, refused over `cap` bytes before it is read whole.
    ///
    /// # Errors
    /// `TooLarge` over `cap`, `Missing` when the file is gone, `Io` when it cannot be read.
    fn read(&self, file: &Path, cap: u64) -> Result<Redacted<Vec<u8>>, PlaceError>;

    /// Writes `bytes` as the box (`box_file`) in `folder` in one step, so a failed write leaves the old box whole.
    ///
    /// # Errors
    /// `Missing`, `Full`, `ReadOnly`, `Refused` for the data folder, and `Io` for any other failure.
    fn write(&self, folder: &Path, bytes: &[u8]) -> Result<(), PlaceError>;

    /// Whether `folder` can take a box of `need` bytes now; `Refused` for the data folder, above it or below it.
    fn state(&self, folder: &Path, need: u64) -> PlaceState;
}

/// The places service; it needs `VaultData`.
pub struct PlacesKey;

impl ServiceKey for PlacesKey {
    type Api = dyn Places;
    const ID: ServiceId = ServiceId::new("places");
    const CAPABILITY: Option<Capability> = Some(Capability::VaultData);
}

#[cfg(test)]
mod tests;
