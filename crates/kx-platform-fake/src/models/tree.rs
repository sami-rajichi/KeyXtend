//! The fake's folders, files and part files, kept by their paths read as Windows reads them on any host: either
//! separator, doubled or trailing ones ignored, and names in any case.

use kx_module_api::Redacted;
use kx_module_api::ports::{Fingerprint, FolderScan, ModelFile, PART_EXT};
use std::collections::BTreeMap;
use std::path::Path;

/// The separator a key uses.
const SEP: char = '\\';
/// Every separator a drive path may use.
const SEPS: [char; 2] = ['\\', '/'];
/// What sits between a file's stem and its extension.
const DOT: char = '.';

/// A path's parts as given, in order from its drive, without the empty ones doubled separators leave.
fn parts(path: &Path) -> Vec<String> {
    let text = path.to_string_lossy();
    let named = text.split(SEPS).filter(|part| !part.is_empty());
    named.map(str::to_owned).collect()
}

/// `path` as the tree keeps it: parts joined by `\`, in lower case.
pub(super) fn key(path: &Path) -> String {
    parts(path).join(&SEP.to_string()).to_lowercase()
}

/// What a key names.
#[derive(Copy, Clone)]
enum Kind {
    Folder,
    File { bytes: u64, print: Fingerprint },
}

/// One folder or file, with its name as given.
struct Entry {
    name: String,
    kind: Kind,
}

/// The folders and files by key, and each part file's bytes by its final file's key.
#[derive(Default)]
pub(super) struct Tree {
    entries: BTreeMap<String, Entry>,
    parts: BTreeMap<String, u64>,
}

impl Tree {
    /// How many folders and files it holds.
    pub(super) fn len(&self) -> usize {
        self.entries.len()
    }

    /// Adds the folder at `path` and every folder above it, down to its drive's root.
    pub(super) fn add_folder(&mut self, path: &Path) {
        let mut key = String::new();
        for (depth, name) in parts(path).into_iter().enumerate() {
            if depth > 0 {
                key.push(SEP);
            }
            key.push_str(&name.to_lowercase());
            let kind = Kind::Folder;
            self.entries
                .entry(key.clone())
                .or_insert(Entry { name, kind });
        }
    }

    /// Adds the file at `path` with its size and fingerprint, and the folders above it.
    pub(super) fn add_file(&mut self, path: &Path, bytes: u64, print: Fingerprint) {
        let mut parts = parts(path);
        let name = parts.pop().unwrap_or_default();
        self.add_folder(Path::new(&parts.join(&SEP.to_string())));
        let kind = Kind::File { bytes, print };
        self.entries.insert(key(path), Entry { name, kind });
    }

    /// Whether `path` is a folder.
    pub(super) fn folder(&self, path: &Path) -> bool {
        let found = self.entries.get(&key(path));
        found.is_some_and(|e| matches!(e.kind, Kind::Folder))
    }

    /// Whether the drive `path` names is a folder.
    pub(super) fn drive(&self, path: &Path) -> bool {
        let root = parts(path).into_iter().next().unwrap_or_default();
        self.folder(Path::new(&root))
    }

    /// Whether the folder above `path` is a folder.
    pub(super) fn parent(&self, path: &Path) -> bool {
        let mut parts = parts(path);
        parts.pop();
        self.folder(Path::new(&parts.join(&SEP.to_string())))
    }

    /// The fingerprint of the file at `path`, if it is one.
    pub(super) fn print(&self, path: &Path) -> Option<Fingerprint> {
        match self.entries.get(&key(path))?.kind {
            Kind::File { print, .. } => Some(print),
            Kind::Folder => None,
        }
    }

    /// What the folder at `path` holds, its files cut to `exts` in any case; `None` when it is no folder.
    pub(super) fn scan(&self, path: &Path, exts: &[String]) -> Option<FolderScan> {
        if !self.folder(path) {
            return None;
        }
        let at = key(path);
        let mut scan = FolderScan::default();
        let under = |k: &String| k.rsplit_once(SEP).is_some_and(|(up, _)| up == at);
        for entry in self
            .entries
            .iter()
            .filter(|(k, _)| under(k))
            .map(|(_, e)| e)
        {
            let name = Redacted::new(entry.name.clone());
            match entry.kind {
                Kind::Folder => scan.folders.push(name),
                Kind::File { bytes, .. } if model(&entry.name, exts) => {
                    scan.files.push(ModelFile { name, bytes });
                }
                Kind::File { .. } => {}
            }
        }
        Some(scan)
    }

    /// Removes `path` and everything under it, a part file by its own path.
    pub(super) fn vanish(&mut self, path: &Path) {
        let at = key(path);
        let below = format!("{at}{SEP}");
        self.entries
            .retain(|k, _| *k != at && !k.starts_with(&below));
        let part = part_of(&at);
        self.parts
            .retain(|k, _| Some(k.as_str()) != part && !k.starts_with(&below));
    }

    /// Removes the file at `path`, or the part file it names; anything else stays.
    pub(super) fn remove(&mut self, path: &Path) {
        let at = key(path);
        if let Some(file) = part_of(&at) {
            self.parts.remove(file);
        }
        if self.print(path).is_some() {
            self.entries.remove(&at);
        }
    }

    /// The bytes of the part file beside the file at `path`, if one is there.
    pub(super) fn part(&self, path: &Path) -> Option<u64> {
        self.parts.get(&key(path)).copied()
    }

    /// Sets the part file beside `path` to `bytes`, or deletes it with `None`.
    pub(super) fn set_part(&mut self, path: &Path, bytes: Option<u64>) {
        match bytes {
            Some(bytes) => self.parts.insert(key(path), bytes),
            None => self.parts.remove(&key(path)),
        };
    }
}

/// The key of the file whose part file `at` keys, if it names one.
fn part_of(at: &str) -> Option<&str> {
    at.strip_suffix(&PART_EXT.to_lowercase())
}

/// True when `name`'s extension is one of `exts` in any case; each ext is given without its dot, as `gguf`.
fn model(name: &str, exts: &[String]) -> bool {
    let ext = name.rsplit_once(DOT).map(|(_, ext)| ext);
    ext.is_some_and(|ext| exts.iter().any(|e| e.eq_ignore_ascii_case(ext)))
}
