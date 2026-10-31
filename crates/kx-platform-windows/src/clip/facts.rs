//! What is at copied paths: only an absolute drive path is probed, so a copy never makes our process reach another
//! machine, and no link is followed, as Windows follows a local link to a share. Each drive is asked once, then each
//! folder on the way, then each file.

use std::collections::HashMap;
use std::io::{self, ErrorKind};
use std::path::{Component, Path, PathBuf};

use kx_module_api::Redacted;
use kx_module_api::ports::{FileFact, probe_ok};

/// What a look that follows no link found at a path.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Found {
    /// A symbolic link or a junction, which may lead to another machine.
    Link,
    /// A file this many bytes long.
    File(u64),
    /// A folder, or anything else that is no file.
    Other,
}

/// What is at each of `paths` now, in order: unchecked off a drive or through a link, away when its drive does not
/// answer, else a file's size, 0 for a folder or a path Windows cannot read, or gone.
pub(crate) fn facts(paths: &Redacted<Vec<PathBuf>>) -> Vec<FileFact> {
    facts_by(paths.expose(), reachable, seen)
}

/// `facts` over the filesystem calls `reach`, which asks a root, and `look`, which asks a path and follows no link.
fn facts_by(
    paths: &[PathBuf],
    mut reach: impl FnMut(&Path) -> bool,
    mut look: impl FnMut(&Path) -> io::Result<Found>,
) -> Vec<FileFact> {
    let mut roots: HashMap<PathBuf, bool> = HashMap::new();
    let mut linked: HashMap<PathBuf, bool> = HashMap::new();
    let mut fact_of = |path: &PathBuf| {
        if !probe_ok(path) {
            return FileFact::Unchecked;
        }
        let top = root(path);
        if !*roots.entry(top.clone()).or_insert_with_key(|r| reach(r)) {
            return FileFact::Away;
        }
        if through_link(path, &top, &mut linked, &mut look) {
            return FileFact::Unchecked;
        }
        told(look(path))
    };
    paths.iter().map(&mut fact_of).collect()
}

/// True when a folder between `top` and `path` is a link; folders are looked at from the top down, each once.
fn through_link(
    path: &Path,
    top: &Path,
    linked: &mut HashMap<PathBuf, bool>,
    look: &mut impl FnMut(&Path) -> io::Result<Found>,
) -> bool {
    let below = |d: &&Path| *d != top && !d.as_os_str().is_empty();
    let folders: Vec<&Path> = path.ancestors().skip(1).take_while(below).collect();
    folders.iter().rev().any(|folder| {
        let is_link = |_: &PathBuf| matches!(look(folder), Ok(Found::Link));
        *linked
            .entry(folder.to_path_buf())
            .or_insert_with_key(is_link)
    })
}

/// The fact a look's answer gives: a link stays unchecked, never followed.
fn told(found: io::Result<Found>) -> FileFact {
    match found {
        Ok(Found::Link) => FileFact::Unchecked,
        Ok(Found::File(len)) => FileFact::Size(len),
        Err(e) if e.kind() == ErrorKind::NotFound => FileFact::Gone,
        Ok(Found::Other) | Err(_) => FileFact::Size(0),
    }
}

/// What is at `path` itself, read with `symlink_metadata` so a link is never followed.
fn seen(path: &Path) -> io::Result<Found> {
    let m = std::fs::symlink_metadata(path)?;
    Ok(if m.file_type().is_symlink() {
        Found::Link
    } else if m.is_file() {
        Found::File(m.len())
    } else {
        Found::Other
    })
}

/// The drive `path` starts with, such as `C:\`; empty where the platform cannot parse it.
fn root(path: &Path) -> PathBuf {
    let top = |c: &Component<'_>| matches!(c, Component::Prefix(_) | Component::RootDir);
    path.components().take_while(top).collect()
}

/// Whether the drive `root` answers now; an empty root's does.
fn reachable(root: &Path) -> bool {
    root.as_os_str().is_empty() || std::fs::metadata(root).is_ok()
}

#[cfg(test)]
mod tests;
