//! Paths compared as Windows does: at their real path when they exist, without case, so a junction, a short name or
//! another case never slips a box into the data folder.

use std::fs;
use std::path::{self, Component, Path, Prefix};

/// `path`'s parts as compared: its real path when it exists, else made absolute, each part in lower case.
pub(super) fn key(path: &Path) -> Vec<String> {
    let real = fs::canonicalize(path).or_else(|_| path::absolute(path));
    let real = real.unwrap_or_else(|_| path.to_path_buf());
    real.components().flat_map(parts).collect()
}

/// Whether `folder` is `data`, holds it or lies in it.
pub(super) fn touches(folder: &Path, data: &Path) -> bool {
    let (folder, data) = (key(folder), key(data));
    folder.starts_with(&data) || data.starts_with(&folder)
}

/// The compared parts of one component: a drive or share without its `\\?\` form, and each name in lower case.
fn parts(component: Component<'_>) -> Vec<String> {
    let lower = |s: &std::ffi::OsStr| s.to_string_lossy().to_lowercase();
    match component {
        Component::Prefix(p) => match p.kind() {
            Prefix::Disk(d) | Prefix::VerbatimDisk(d) => {
                vec![char::from(d).to_lowercase().collect()]
            }
            // An empty part, which no name is, keeps a share apart from a drive of the same letter.
            Prefix::UNC(s, share) | Prefix::VerbatimUNC(s, share) => {
                vec![String::new(), lower(s), lower(share)]
            }
            Prefix::Verbatim(s) | Prefix::DeviceNS(s) => vec![lower(s)],
        },
        Component::Normal(name) => vec![lower(name)],
        Component::RootDir | Component::CurDir | Component::ParentDir => Vec::new(),
    }
}
