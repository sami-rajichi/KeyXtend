//! The stage check: no banned plugin, no gate-only build, every import found in the stage or the system folder, the
//! network library imported only by the listed Qt libraries, no library that nothing imports, no file off the list,
//! and the download budget: the files deflated, a conservative stand-in for the installer's LZMA2, within `max_mb`.
//! The packs folder is the words check's (`words::check`).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use super::{Network, StageConfig, StageError, planned, words};
use crate::devtools::{DevError, paths};

/// The extension of Windows libraries, which Qt's plugins are too.
pub(super) const LIB_EXT: &str = "dll";

/// What the check found in a stage that passed.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Summary {
    /// Files in the stage.
    pub files: usize,
    /// Their total size in bytes, as installed.
    pub bytes: u64,
    /// Their total size deflated, as the download carries them.
    pub deflated: u64,
}

impl Summary {
    /// The download size in MB, rounded up: the figure the budget counts.
    pub(crate) fn mb(&self) -> u64 {
        paths::mb(self.deflated)
    }

    /// The installed size in MB, rounded up.
    pub(crate) fn raw_mb(&self) -> u64 {
        paths::mb(self.bytes)
    }
}

/// Checks `stage`, measuring its download deflated at `level`; `read` gives a file's imported library names and
/// `in_system` says whether Windows has a library.
pub(crate) fn check(
    stage: &Path,
    config: &StageConfig,
    level: u32,
    read: impl Fn(&Path) -> Result<Vec<String>, DevError>,
    in_system: impl Fn(&str) -> bool,
) -> Result<Summary, DevError> {
    let files = paths::files(stage)?;
    refuse_banned(&files, &config.banned)?;
    let program = PathBuf::from(config.program());
    if !files.contains(&program) {
        return Err(StageError::NoExe(paths::text(stage)).into());
    }
    refuse_gate_exe(&stage.join(&program), &program, &config.gate_flag)?;
    let libs = libraries(&files);
    let found = |lib: &str| is_api_set(lib, &config.api_sets) || in_system(lib);
    let resolves = |lib: &str| paths::is_plain_name(lib) && found(lib);
    let guarded = |file: &Path| {
        let imports = read(&stage.join(file))?;
        guard(file, &imports, &config.network).map(|()| imports)
    };
    let reached = closure(roots(&files, config), &libs, guarded, resolves)?;
    if let Some(unused) = libs.iter().find(|(key, _)| !reached.contains(*key)) {
        return Err(StageError::Unused(paths::text(unused.1)).into());
    }
    refuse_unplanned(&files, &planned(config))?;
    let summary = measure(stage, &files, level)?;
    if summary.mb() > config.max_mb {
        let (mb, max) = (summary.mb(), config.max_mb);
        return Err(StageError::OverBudget { mb, max }.into());
    }
    Ok(summary)
}

/// The count, installed bytes and deflated bytes of the stage-relative `files` in `stage`, each deflated at `level`.
pub(crate) fn measure(stage: &Path, files: &[PathBuf], level: u32) -> Result<Summary, DevError> {
    let (mut bytes, mut deflated) = (0, 0);
    for file in files {
        let path = stage.join(file);
        let read = std::fs::read(&path)
            .map_err(|e| DevError::Io(format!("{}: {e}", paths::text(&path))))?;
        bytes += read.len();
        deflated += crate::deflate::deflated(&read, level)?;
    }
    let total = |n: usize| u64::try_from(n).unwrap_or(u64::MAX);
    Ok(Summary {
        files: files.len(),
        bytes: total(bytes),
        deflated: total(deflated),
    })
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
    found.map_or(Ok(()), |f| Err(StageError::Banned(paths::text(f)).into()))
}

/// Reads the program at `path` and fails, naming it `exe`, when it holds the gate-only `flag`.
pub(crate) fn refuse_gate_exe(path: &Path, exe: &Path, flag: &str) -> Result<(), DevError> {
    let bytes = std::fs::read(path)?;
    refuse_gate(&bytes, flag, exe).map_err(DevError::from)
}

/// Fails when `bytes`, the program `exe`, hold the gate-only `flag` anywhere; an empty flag fails every program.
fn refuse_gate(bytes: &[u8], flag: &str, exe: &Path) -> Result<(), StageError> {
    let flag_bytes = flag.as_bytes();
    if flag.is_empty() || bytes.windows(flag_bytes.len()).any(|w| w == flag_bytes) {
        let (exe, flag) = (paths::text(exe), flag.to_string());
        return Err(StageError::GateFlag { exe, flag });
    }
    Ok(())
}

/// Fails on the first file outside the packs folder that is not one of the `planned` paths, in any letter case, so no
/// unlisted file is signed; the words check looks after the packs folder.
fn refuse_unplanned(files: &[PathBuf], planned: &[PathBuf]) -> Result<(), DevError> {
    let listed: BTreeSet<String> = planned.iter().map(|p| path_key(p)).collect();
    let mut outside = files.iter().filter(|f| !words::inside(f));
    let extra = outside.find(|f| !listed.contains(&path_key(f)));
    extra.map_or(
        Ok(()),
        |f| Err(StageError::Unplanned(paths::text(f)).into()),
    )
}

/// The key a stage-relative path is matched by: its parts in lower case, joined the same way on every system.
pub(super) fn path_key(path: &Path) -> String {
    let parts: Vec<String> = path.iter().map(|p| key(&p.to_string_lossy())).collect();
    parts.join("/")
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
        Some(lib) if !allowed => Err(StageError::NetworkImport {
            by: paths::text(file),
            lib: lib.clone(),
        }
        .into()),
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
                return Err(StageError::Unresolved { by, lib }.into());
            }
        }
    }
    Ok(reached)
}

#[cfg(test)]
mod tests;
