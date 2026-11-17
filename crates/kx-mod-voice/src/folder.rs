//! The models folder (design "The models folder and downloads", ADR-0043 decisions 6 and 7): the model files a scan
//! found, each known by its fingerprint once read, other files' own facts, the room on its drive, and the folder
//! offered at the first download. Pure: the actor reads the folder, the files and the drives, and hands them in.

mod supply;

pub use supply::Stock;

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use kx_module_api::Redacted;
use kx_module_api::ports::{
    Drive, Fingerprint, FolderLine, FolderScan, MB, ModelError, ModelFacts, SpeechError, path_ok,
};

use crate::models::{Other, Row, file_ok, table};

/// The folder offered at the first download, under the root of the drive with the most room (ADR-0043 decision 6).
pub const OFFER: [&str; 2] = ["KeyXtend", "models"];

/// Where a file's fingerprint stands.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Print {
    /// Not read yet.
    Pending,
    /// Read.
    Known(Fingerprint),
    /// It could not be read, so the file is left out.
    Failed,
}

/// A model file in the folder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct File {
    /// Its name in the folder.
    pub name: String,
    /// Its size in bytes.
    pub bytes: u64,
    /// Its fingerprint.
    pub print: Print,
}

impl File {
    /// True when it is `name` of `bytes`, the name in any case, as the system's file names are.
    fn is(&self, name: &str, bytes: u64) -> bool {
        self.bytes == bytes && self.name.eq_ignore_ascii_case(name)
    }
}

/// The models folder and what it holds.
#[derive(Clone, Debug, Default)]
pub struct Folder {
    dir: Option<PathBuf>,
    /// The folder was scanned since it was chosen, so its files are known.
    read: bool,
    files: Vec<File>,
    /// This folder's fingerprints read, by lower-case name and size, so a new scan reads only new files.
    prints: BTreeMap<(String, u64), Print>,
    /// Other files' facts by fingerprint; `None` when the worker could not read them.
    facts: HashMap<Fingerprint, Option<ModelFacts>>,
}

impl Folder {
    /// The folder `models_dir` names; none while it is empty or no plain drive path (`path_ok`).
    #[must_use]
    pub fn new(models_dir: &str) -> Self {
        let mut folder = Self::default();
        folder.follow(models_dir);
        folder
    }

    /// The folder, once chosen.
    #[must_use]
    pub fn dir(&self) -> Option<&Path> {
        self.dir.as_deref()
    }

    /// Uses `dir` from now on, forgetting the old folder's files and fingerprints; false, changing nothing, unless
    /// `path_ok`.
    pub fn set_dir(&mut self, dir: &Path) -> bool {
        if !path_ok(dir) {
            return false;
        }
        if self.dir.as_deref() != Some(dir) {
            self.dir = Some(dir.to_path_buf());
            self.forget();
        }
        true
    }

    /// Follows `models_dir` after a settings change; true when the folder changed, so it is read again.
    pub fn follow(&mut self, models_dir: &str) -> bool {
        let before = self.dir.clone();
        if !self.set_dir(Path::new(models_dir)) {
            self.dir = None;
            self.forget();
        }
        before != self.dir
    }

    /// Forgets the files and fingerprints, as another folder may hold another file of the same name and size.
    fn forget(&mut self) {
        self.files.clear();
        self.prints.clear();
        self.read = false;
    }

    /// Takes a scan of the folder, keeping plain model file names; gives the files, with their sizes, whose fingerprint
    /// is still to be read, off the module's thread. A folder that cannot be read holds nothing.
    pub fn scanned(&mut self, scan: Result<FolderScan, ModelError>) -> Vec<(PathBuf, u64)> {
        self.files.clear();
        self.read = self.dir.is_some();
        let (Ok(scan), Some(dir), Ok(table)) = (scan, self.dir.clone(), table()) else {
            return Vec::new();
        };
        let plain = scan
            .files
            .into_iter()
            .filter(|f| file_ok(f.name.expose(), table.exts()));
        for file in plain {
            let (name, bytes) = (file.name.expose().clone(), file.bytes);
            let key = (name.to_lowercase(), bytes);
            let print = self.prints.get(&key).copied().unwrap_or(Print::Pending);
            self.files.push(File { name, bytes, print });
        }
        let unread = self.files.iter().filter(|f| f.print == Print::Pending);
        unread.map(|f| (dir.join(&f.name), f.bytes)).collect()
    }

    /// Takes the fingerprint of `file` of `bytes`, as `scanned` gave it; gives the other file whose facts are still to
    /// be read. A late answer for a file this folder no longer holds is dropped.
    pub fn printed(
        &mut self,
        file: &Path,
        bytes: u64,
        print: Result<Fingerprint, ModelError>,
    ) -> Option<Other> {
        let dir = self.dir.clone()?;
        let held = |f: &&mut File| f.bytes == bytes && dir.join(&f.name) == file;
        let found = self.files.iter_mut().find(held)?;
        let print = print.map_or(Print::Failed, Print::Known);
        found.print = print;
        let name = found.name.clone();
        self.prints.insert((name.to_lowercase(), bytes), print);
        let Print::Known(known) = print else {
            return None;
        };
        let other = table().is_ok_and(|t| t.printed(&known).is_none());
        (other && self.wants_facts(&known)).then_some(Other { name, bytes })
    }

    /// Takes an other file's facts, or the error that left them unread, which is not asked again.
    pub fn facts_read(&mut self, print: Fingerprint, facts: Result<ModelFacts, SpeechError>) {
        self.facts.insert(print, facts.ok());
    }

    /// True while an other file of fingerprint `print` has no facts and none were refused.
    #[must_use]
    pub fn wants_facts(&self, print: &Fingerprint) -> bool {
        !self.facts.contains_key(print)
    }

    /// An other file's facts, once read.
    #[must_use]
    pub fn facts(&self, print: &Fingerprint) -> Option<&ModelFacts> {
        self.facts.get(print).and_then(Option::as_ref)
    }

    /// A download of `row` took its name, checked by the fetch job, so its file is known without reading it.
    pub fn fetched(&mut self, row: &Row) {
        let print = Print::Known(row.print);
        self.prints
            .insert((row.file.to_lowercase(), row.bytes), print);
        self.files
            .retain(|f| !f.name.eq_ignore_ascii_case(&row.file));
        let file = File {
            name: row.file.clone(),
            bytes: row.bytes,
            print,
        };
        self.files.push(file);
    }

    /// The file of `row`: the one with its fingerprint, else one with its name and size not read yet.
    #[must_use]
    pub fn of(&self, row: &Row) -> Option<&File> {
        let printed = self
            .files
            .iter()
            .find(|f| f.print == Print::Known(row.print));
        let named = || {
            let unread = |f: &&File| f.print == Print::Pending && f.is(&row.file, row.bytes);
            self.files.iter().find(unread)
        };
        printed.or_else(named)
    }

    /// The file named `name`, in any case.
    #[must_use]
    pub fn file(&self, name: &str) -> Option<&File> {
        self.files
            .iter()
            .find(|f| f.name.eq_ignore_ascii_case(name))
    }

    /// The file holding `row`'s name when it is not `row`'s, which a download never replaces.
    #[must_use]
    pub fn taken(&self, row: &Row) -> Option<&File> {
        self.of(row).map_or_else(|| self.file(&row.file), |_| None)
    }

    /// The files no row's fingerprint names, A to Z as scanned.
    pub fn others(&self) -> impl Iterator<Item = &File> {
        let rowless = |f: &&File| match f.print {
            Print::Known(p) => table().is_ok_and(|t| t.printed(&p).is_none()),
            Print::Pending | Print::Failed => false,
        };
        self.files.iter().filter(rowless)
    }

    /// True while the chosen folder is not scanned yet or a fingerprint is still being read, so Download waits: a
    /// copy of the model may be there under another name.
    #[must_use]
    pub fn reading(&self) -> bool {
        let pending = self.files.iter().any(|f| f.print == Print::Pending);
        self.dir.is_some() && (!self.read || pending)
    }

    /// True when a file with fingerprint `print` was read, so a card's check may go by it.
    #[must_use]
    pub fn known(&self, print: &Fingerprint) -> bool {
        self.files.iter().any(|f| f.print == Print::Known(*print))
    }

    /// The folder's line on the Models page, with the room of its drive among `drives`; `None` until one is chosen.
    #[must_use]
    pub fn line(&self, drives: &[Drive]) -> Option<FolderLine> {
        let dir = self.dir.as_deref()?;
        let free = drive_of(drives, dir).map_or(0, |d| d.free);
        Some(line(dir, free))
    }

    /// Fine when the folder's drive among `drives` has `bytes` and `min_free_mb` free.
    ///
    /// # Errors
    /// `Missing` when no folder is chosen or no fixed drive holds it, `Full` under that room.
    pub fn room(&self, drives: &[Drive], bytes: u64, min_free_mb: u32) -> Result<(), ModelError> {
        let dir = self.dir.as_deref().ok_or(ModelError::Missing)?;
        let drive = drive_of(drives, dir).ok_or(ModelError::Missing)?;
        let need = bytes.saturating_add(u64::from(min_free_mb).saturating_mul(MB));
        (drive.free >= need).then_some(()).ok_or(ModelError::Full)
    }
}

/// The folder offered at the first download, `OFFER` on the drive with the most room; `None` without a drive.
#[must_use]
pub fn offer(drives: &[Drive]) -> Option<(PathBuf, FolderLine)> {
    let roomy = drives.iter().max_by_key(|d| d.free)?;
    let dir = OFFER
        .iter()
        .fold(roomy.root.expose().clone(), |p, part| p.join(part));
    let shown = line(&dir, roomy.free);
    Some((dir, shown))
}

/// The fixed drive whose root holds `dir`, the longest root first, compared as text in any case with either
/// separator, so it reads a drive path the same on every system.
fn drive_of<'d>(drives: &'d [Drive], dir: &Path) -> Option<&'d Drive> {
    let text = |p: &Path| p.to_string_lossy().to_lowercase().replace(SLASH, BACKSLASH);
    let dir = text(dir);
    let holds = |d: &&Drive| {
        let root = text(d.root.expose());
        let rest = dir.strip_prefix(root.trim_end_matches(BACKSLASH));
        rest.is_some_and(|r| r.is_empty() || r.starts_with(BACKSLASH))
    };
    let longest = |d: &&Drive| d.root.expose().as_os_str().len();
    drives.iter().filter(holds).max_by_key(longest)
}

/// A drive path's forward separator, which `drive_of` reads as `BACKSLASH`.
const SLASH: char = '/';
/// A drive path's own separator.
const BACKSLASH: &str = "\\";

/// A folder line for `dir` with `free` bytes on its drive.
fn line(dir: &Path, free: u64) -> FolderLine {
    let path = Redacted::new(dir.to_string_lossy().into_owned());
    FolderLine { path, free }
}

#[cfg(test)]
mod tests;
