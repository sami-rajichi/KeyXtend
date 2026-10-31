//! What is at copied paths: a share or a device path is left unchecked, so a copy never makes our process reach
//! another machine; each local drive is asked once, then each file.

use std::collections::HashMap;
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};

use kx_module_api::Redacted;
use kx_module_api::ports::{FileFact, is_share_or_device};

/// What is at each of `paths` now, in order: unchecked for a share or device path, away when its drive does not
/// answer, else a file's size, 0 for a folder or a path Windows cannot read, or gone. Each drive is asked once.
pub(crate) fn facts(paths: &Redacted<Vec<PathBuf>>) -> Vec<FileFact> {
    facts_by(paths.expose(), reachable, fact)
}

/// `facts` over the filesystem calls `reach`, which asks a root, and `look`, which asks a path; a share or device
/// path makes neither.
fn facts_by(
    paths: &[PathBuf],
    mut reach: impl FnMut(&Path) -> bool,
    mut look: impl FnMut(&Path) -> FileFact,
) -> Vec<FileFact> {
    let mut roots: HashMap<PathBuf, bool> = HashMap::new();
    let mut fact_of = |path: &PathBuf| {
        if is_share_or_device(path) {
            return FileFact::Unchecked;
        }
        let near = *roots.entry(root(path)).or_insert_with_key(|r| reach(r));
        if near { look(path) } else { FileFact::Away }
    };
    paths.iter().map(&mut fact_of).collect()
}

/// What is at `path` now, on a drive that answers.
fn fact(path: &Path) -> FileFact {
    match std::fs::metadata(path) {
        Ok(m) if m.is_file() => FileFact::Size(m.len()),
        Err(e) if e.kind() == ErrorKind::NotFound => FileFact::Gone,
        Ok(_) | Err(_) => FileFact::Size(0),
    }
}

/// The drive `path` starts with, such as `C:\`; empty for a relative path.
fn root(path: &Path) -> PathBuf {
    let top = |c: &Component<'_>| matches!(c, Component::Prefix(_) | Component::RootDir);
    path.components().take_while(top).collect()
}

/// Whether the drive `root` answers now; a relative path's does.
fn reachable(root: &Path) -> bool {
    root.as_os_str().is_empty() || std::fs::metadata(root).is_ok()
}

#[cfg(test)]
mod tests;
