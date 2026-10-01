//! The stage check: no banned plugin, every import found in the stage or the system folder, the network library
//! imported only by the listed Qt libraries, no library that nothing imports, and the size under the cap.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use super::{Network, StageConfig};
use crate::devtools::{DevError, paths};

/// The extension of Windows libraries, which Qt's plugins are too.
pub(super) const LIB_EXT: &str = "dll";

/// What the check found in a stage that passed.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Summary {
    /// Files in the stage.
    pub files: usize,
    /// Their total size in bytes.
    pub bytes: u64,
}

impl Summary {
    /// The size in MB, rounded up.
    pub(crate) fn mb(&self) -> u64 {
        self.bytes.div_ceil(paths::BYTES_PER_MB)
    }
}

/// Checks `stage`; `read` gives a file's imported library names and `in_system` says whether Windows has a library.
pub(crate) fn check(
    stage: &Path,
    config: &StageConfig,
    read: impl Fn(&Path) -> Result<Vec<String>, DevError>,
    in_system: impl Fn(&str) -> bool,
) -> Result<Summary, DevError> {
    let files = paths::files(stage)?;
    refuse_banned(&files, &config.banned)?;
    if !files.contains(&PathBuf::from(config.program())) {
        return Err(DevError::NoExe(paths::text(stage)));
    }
    let libs = libraries(&files);
    let found = |lib: &str| is_api_set(lib, &config.api_sets) || in_system(lib);
    let resolves = |lib: &str| paths::is_plain_name(lib) && found(lib);
    let guarded = |file: &Path| {
        let imports = read(&stage.join(file))?;
        guard(file, &imports, &config.network).map(|()| imports)
    };
    let reached = closure(roots(&files, config), &libs, guarded, resolves)?;
    if let Some(unused) = libs.iter().find(|(key, _)| !reached.contains(*key)) {
        return Err(DevError::Unused(paths::text(unused.1)));
    }
    let bytes = paths::folder_size(stage)?;
    let summary = Summary {
        files: files.len(),
        bytes,
    };
    if summary.mb() > config.max_mb {
        return Err(DevError::TooBig {
            mb: summary.mb(),
            max: config.max_mb,
        });
    }
    Ok(summary)
}

/// Fails on the first file with a `banned` folder anywhere in its path, in any letter case.
fn refuse_banned(files: &[PathBuf], banned: &[String]) -> Result<(), DevError> {
    let is_banned = |part: &std::ffi::OsStr| {
        let part = part.to_string_lossy();
        banned.iter().any(|b| b.eq_ignore_ascii_case(&part))
    };
    let found = files
        .iter()
        .find(|f| f.parent().is_some_and(|dir| dir.iter().any(is_banned)));
    found.map_or(Ok(()), |f| Err(DevError::Banned(paths::text(f))))
}

/// The key the loader matches a library name by: its lower-case text.
pub(super) fn key(name: &str) -> String {
    name.to_lowercase()
}

/// True if `path` has the library extension, in any letter case; the walk reads only such files.
pub(super) fn is_lib(path: &Path) -> bool {
    path.extension()
        .is_some_and(|e| e.to_string_lossy().eq_ignore_ascii_case(LIB_EXT))
}

/// The libraries directly beside the program, by key; only these satisfy an import by name.
fn libraries(files: &[PathBuf]) -> BTreeMap<String, PathBuf> {
    files
        .iter()
        .filter(|f| f.parent() == Some(Path::new("")) && is_lib(f))
        .map(|f| (key(&f.to_string_lossy()), f.clone()))
        .collect()
}

/// Where the walk starts: the program, and every library in a subfolder, which Qt loads as a plugin.
fn roots(files: &[PathBuf], config: &StageConfig) -> Vec<PathBuf> {
    let plugin = |f: &&PathBuf| f.parent() != Some(Path::new("")) && is_lib(f);
    let plugins = files.iter().filter(plugin).cloned();
    std::iter::once(PathBuf::from(config.program()))
        .chain(plugins)
        .collect()
}

/// Fails if `file` imports the network library but is not a library beside the program that `network` allows.
fn guard(file: &Path, imports: &[String], network: &Network) -> Result<(), DevError> {
    let name = key(&file.to_string_lossy());
    let beside = file.parent() == Some(Path::new(""));
    let allowed = beside && network.by.iter().any(|b| key(b) == name);
    match imports.iter().find(|lib| key(lib) == key(&network.lib)) {
        Some(lib) if !allowed => Err(DevError::NetworkImport {
            by: paths::text(file),
            lib: lib.clone(),
        }),
        _ => Ok(()),
    }
}

/// True if `lib` names a Windows API set by one of `prefixes`.
fn is_api_set(lib: &str, prefixes: &[String]) -> bool {
    let lib = key(lib);
    prefixes.iter().any(|p| lib.starts_with(&key(p)))
}

/// Walks the imports from `roots`, by stage-relative path; returns the keys of the staged libraries reached, or the
/// first import found nowhere.
fn closure(
    roots: Vec<PathBuf>,
    libs: &BTreeMap<String, PathBuf>,
    read: impl Fn(&Path) -> Result<Vec<String>, DevError>,
    resolves: impl Fn(&str) -> bool,
) -> Result<BTreeSet<String>, DevError> {
    let mut reached = BTreeSet::new();
    let mut queue = roots;
    while let Some(file) = queue.pop() {
        for lib in read(&file)? {
            if let Some(staged) = libs.get(&key(&lib)) {
                if reached.insert(key(&lib)) {
                    queue.push(staged.clone());
                }
            } else if !resolves(&lib) {
                let by = paths::text(&file);
                return Err(DevError::Unresolved { by, lib });
            }
        }
    }
    Ok(reached)
}

#[cfg(test)]
mod tests;
