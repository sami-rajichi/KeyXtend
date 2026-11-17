//! F5: the worker's jobs kept apart (ADR-0042): no file of a module named in `apart`, its own `.rs` file or any file
//! under its folder, names a crate banned there, outside comments and literals.

use std::fs;
use std::path::{Path, PathBuf};

use super::bridges::{Scan, whole};
use super::files::{Pick, relative_place, walk_dir};
use super::{F5, Severity, Violation};
use crate::config::Apart;
use crate::devtools::paths::join_rel;
use crate::workspace::Workspace;

/// The only source files F5 reads.
const RUST_EXT: &str = "rs";
/// The dash a crate name may hold.
const NAME_DASH: char = '-';
/// What code spells that dash as.
const CODE_DASH: &str = "_";

/// F5: each `apart` entry's module names none of its banned crates; a missing package or file-less module fails closed.
pub(crate) fn f5_apart(ws: &Workspace) -> Vec<Violation> {
    ws.tidy.apart.iter().flat_map(|a| check(a, ws)).collect()
}

/// F5's findings for one entry.
fn check(apart: &Apart, ws: &Workspace) -> Vec<Violation> {
    let dir = ws
        .find_member(&apart.package)
        .and_then(|p| p.manifest_path.parent());
    let Some(dir) = dir else {
        return vec![error(
            &apart.package,
            "no workspace member; F5 reads nothing".to_string(),
        )];
    };
    let base = join_rel(dir.to_path_buf(), &apart.module);
    let files = module_files(&base, ws);
    if files.is_empty() {
        let place = relative_place(&base, &ws.root_dir);
        return vec![error(
            &place,
            "holds no Rust file; F5 reads nothing".to_string(),
        )];
    }
    let words: Vec<String> = apart
        .banned
        .iter()
        .map(|c| c.replace(NAME_DASH, CODE_DASH))
        .collect();
    files
        .iter()
        .flat_map(|f| check_file(f, &words, ws))
        .collect()
}

/// The module's own `.rs` file, then every Rust file under its folder, in name order.
fn module_files(base: &Path, ws: &Workspace) -> Vec<PathBuf> {
    let own = base.with_extension(RUST_EXT);
    let (exts, mut files, mut ignored) = ([RUST_EXT.to_string()], Vec::new(), Vec::new());
    let pick = Pick {
        exts: &exts,
        skip: &ws.tidy.skip_dirs,
    };
    walk_dir(base, &ws.root_dir, &pick, &mut files, &mut ignored);
    files.sort();
    own.is_file()
        .then_some(own)
        .into_iter()
        .chain(files)
        .collect()
}

/// One violation per line of `path` whose code names a word in `words`. An unreadable file is F1's to report.
fn check_file(path: &Path, words: &[String], ws: &Workspace) -> Vec<Violation> {
    let Ok(text) = fs::read_to_string(path) else {
        return Vec::new();
    };
    let place = relative_place(path, &ws.root_dir);
    let mut scan = Scan::default();
    let lines = text.lines().map(|l| scan.code(l)).enumerate();
    let named = |code: &str| words.iter().find(|w| names(code, w)).cloned();
    let found = lines.filter_map(|(at, code)| named(&code).map(|w| (at + 1, w)));
    let message = |w: String| format!("names {w}, which this job must not use");
    found
        .map(|(line, w)| error(&format!("{place}:{line}"), message(w)))
        .collect()
}

/// True when `code` holds `word` as a whole name.
fn names(code: &str, word: &str) -> bool {
    code.match_indices(word)
        .any(|(at, _)| whole(code, at, word.len()))
}

/// An F5 error at `place`.
fn error(place: &str, message: String) -> Violation {
    Violation {
        rule: F5,
        place: place.to_string(),
        message,
        severity: Severity::Error,
    }
}

#[cfg(test)]
mod tests;
