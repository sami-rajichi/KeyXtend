//! The models folder on disk: made with the folders above it, the topmost one this host makes guarded from the start
//! and those below inheriting it; a scan of its subfolders and model files, A to Z, without hidden entries or links;
//! and a file's removal.

use super::{Disk, fault, read_fault};
use kx_module_api::Redacted;
use kx_module_api::ports::{FolderScan, ModelError, ModelFile};
use std::fs;
use std::io;
use std::path::Path;

/// Makes `folder` and the missing folders above it, top first, the first one made guarded; a folder another program
/// made meanwhile keeps its rights. A failure removes the empty folders this call made.
pub(super) fn make(disk: &dyn Disk, folder: &Path) -> Result<(), ModelError> {
    let missing = |p: &&Path| !matches!(p.try_exists(), Ok(true));
    let todo: Vec<&Path> = folder.ancestors().take_while(missing).collect();
    if todo.is_empty() && !folder.is_dir() {
        return Err(fault(&io::ErrorKind::AlreadyExists.into()));
    }
    let mut made: Vec<&Path> = Vec::new();
    for dir in todo.into_iter().rev() {
        let step = if made.is_empty() {
            disk.make_guarded(dir)
        } else {
            fs::create_dir(dir)
        };
        match step {
            Ok(()) => made.push(dir),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists && dir.is_dir() => {}
            Err(e) => {
                made.iter().rev().for_each(|dir| drop(fs::remove_dir(dir)));
                return Err(fault(&e));
            }
        }
    }
    Ok(())
}

/// The subfolders of `folder` and its files with one of `exts`, each A to Z; hidden entries, links and names that
/// are not UTF-8 are left out.
pub(super) fn scan(
    disk: &dyn Disk,
    folder: &Path,
    exts: &[String],
) -> Result<FolderScan, ModelError> {
    let mut found = FolderScan::default();
    for entry in fs::read_dir(folder).map_err(|e| read_fault(&e))? {
        let entry = entry.map_err(|e| read_fault(&e))?;
        let (Ok(kind), Ok(name)) = (entry.file_type(), entry.file_name().into_string()) else {
            continue;
        };
        if kind.is_symlink() || disk.hidden(&entry.path()) {
            continue;
        }
        if kind.is_dir() {
            found.folders.push(Redacted::new(name));
        } else if kind.is_file() && wanted(&name, exts) {
            let bytes = entry.metadata().map_err(|e| read_fault(&e))?.len();
            let name = Redacted::new(name);
            found.files.push(ModelFile { name, bytes });
        }
    }
    found.folders.sort_by_cached_key(|n| order(n.expose()));
    found.files.sort_by_cached_key(|f| order(f.name.expose()));
    Ok(found)
}

/// Deletes `file`; one already gone is no error.
pub(super) fn remove(file: &Path) -> Result<(), ModelError> {
    match fs::remove_file(file) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => Err(fault(&e)),
        _ => Ok(()),
    }
}

/// True when `name`'s extension is one of `exts`, given without a dot, in any case.
fn wanted(name: &str, exts: &[String]) -> bool {
    let ext = Path::new(name).extension().and_then(|e| e.to_str());
    ext.is_some_and(|ext| exts.iter().any(|want| want.eq_ignore_ascii_case(ext)))
}

/// The order A to Z: by name in any case, then as written.
fn order(name: &str) -> (String, String) {
    (name.to_lowercase(), name.to_owned())
}
