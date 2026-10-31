//! F4: product code starts no process itself; a worker starts through the platform's spawner, which lowers its rights
//! and lets it inherit only its pipes.

use std::fs;
use std::ops::RangeInclusive;
use std::path::{Path, PathBuf};

use super::bridges::{Scan, part, whole};
use super::files::{Pick, relative_place, walk_dir};
use super::{F4, Severity, Violation};
use crate::workspace::{Package, Workspace};
use mods::{Mods, test_mods};

mod mods;

/// Cargo's folder of integration tests, beside the manifest.
const TESTS_DIR: &str = "tests";
/// Files whose child modules sit beside them, not in a folder named after them.
const ROOT_FILES: [&str; 3] = ["lib.rs", "main.rs", "mod.rs"];
/// Rust source files, the only ones with test modules.
const RUST_EXT: &str = "rs";
/// Separates the names of a path.
const PATH_SEP: &str = "::";
/// The keyword that renames an import.
const AS: &str = "as";
/// The import of a module itself, inside its braces.
const SELF: &str = "self";

/// One file F4 reads: its path, code lines and test-only modules.
struct Source {
    path: PathBuf,
    code: Vec<String>,
    mods: Mods,
}

/// F4: no `banned_spawns` in the product code of a package under `spawn_dirs`. Its integration tests, test-only
/// modules and `build_scripts` may use them; the test tools sit outside those folders.
pub(crate) fn f4_no_spawn(ws: &Workspace) -> Vec<Violation> {
    let under = |p: &&Package| {
        let dirs = ws.tidy.spawn_dirs.iter();
        dirs.map(|d| ws.root_dir.join(d))
            .any(|d| p.manifest_path.starts_with(d))
    };
    ws.members()
        .filter(under)
        .flat_map(|p| check_package(p, ws))
        .collect()
}

/// F4 violations in one package.
fn check_package(package: &Package, ws: &Workspace) -> Vec<Violation> {
    let Some(dir) = package.manifest_path.parent() else {
        return Vec::new();
    };
    let sources = sources(dir, ws);
    let shields = shields(dir, &sources, ws);
    sources
        .iter()
        .filter(|s| !shields.iter().any(|p| s.path.starts_with(p)))
        .flat_map(|s| report(s, ws))
        .collect()
}

/// The files of the package in `dir` that F4 reads.
fn sources(dir: &Path, ws: &Workspace) -> Vec<Source> {
    let tidy = &ws.tidy;
    let pick = Pick {
        exts: &tidy.spawn_extensions,
        skip: &tidy.skip_dirs,
    };
    let (mut paths, mut ignored) = (Vec::new(), Vec::new());
    walk_dir(dir, &ws.root_dir, &pick, &mut paths, &mut ignored);
    let source = |path: PathBuf| {
        let code = read(&path)?;
        let rust = path.extension().is_some_and(|e| e == RUST_EXT);
        let mods = if rust {
            test_mods(&code)
        } else {
            Mods::default()
        };
        Some(Source { path, code, mods })
    };
    paths.into_iter().filter_map(source).collect()
}

/// The files and folders of the package in `dir` that may start processes: tests, test modules and build scripts.
fn shields(dir: &Path, sources: &[Source], ws: &Workspace) -> Vec<PathBuf> {
    let scripts = ws.tidy.build_scripts.iter().map(|s| dir.join(s));
    let mods = sources.iter().flat_map(|s| {
        let names = s.mods.names.iter().filter(|n| !built(sources, s, n));
        names.flat_map(move |n| test_paths(&s.path, n))
    });
    scripts.chain(mods).chain([dir.join(TESTS_DIR)]).collect()
}

/// True when a file other than `file` whose child modules share its folder, as `lib.rs` does beside `main.rs`,
/// builds module `name` for the product.
fn built(sources: &[Source], file: &Source, name: &str) -> bool {
    let dir = child_dir(&file.path);
    let other = |s: &&Source| s.path != file.path && child_dir(&s.path) == dir;
    sources
        .iter()
        .filter(other)
        .any(|s| s.mods.live.iter().any(|n| n == name))
}

/// `path`'s lines without comments and with literals blanked; an unreadable file is F1's to report.
fn read(path: &Path) -> Option<Vec<String>> {
    let text = fs::read_to_string(path).ok()?;
    let mut scan = Scan::default();
    Some(text.lines().map(|l| scan.code(l)).collect())
}

/// F4 violations in one product file.
fn report(source: &Source, ws: &Workspace) -> Vec<Violation> {
    let place = relative_place(&source.path, &ws.root_dir);
    let found = findings(&source.code, &source.mods.spans, &ws.tidy.banned_spawns);
    let violation = |(line, entry): (usize, String)| Violation {
        rule: F4,
        place: format!("{place}:{line}"),
        message: format!("{entry} in product code; a worker starts through the platform's spawner"),
        severity: Severity::Error,
    };
    found.into_iter().map(violation).collect()
}

/// F4's findings in a file's code lines, as (line from 1, entry); lines in `spans` are test code.
/// An entry is found where its last name stands in a file whose live code also names the one before it, and wherever
/// that one is renamed, which would hide the last name in other files.
fn findings(
    code: &[String],
    spans: &[RangeInclusive<usize>],
    banned: &[String],
) -> Vec<(usize, String)> {
    let live: Vec<(usize, &String)> = code
        .iter()
        .enumerate()
        .filter(|(at, _)| !spans.iter().any(|s| s.contains(at)))
        .collect();
    let lines = |hit: &dyn Fn(&str) -> bool| -> Vec<usize> {
        let found = live.iter().filter(|(_, c)| hit(c));
        found.map(|&(at, _)| at).collect()
    };
    let mut found = Vec::new();
    for entry in banned {
        let mut names = entry.rsplit(PATH_SEP).filter(|n| !n.is_empty());
        let Some(name) = names.next() else {
            continue;
        };
        let module = names.next();
        let mut at = lines(&|c| names_word(c, name));
        if module.is_some_and(|m| lines(&|c| names_word(c, m)).is_empty()) {
            at.clear();
        }
        at.extend(module.map_or_else(Vec::new, |m| lines(&|c| renames(c, m))));
        found.extend(at.into_iter().map(|at| (at + 1, entry.clone())));
    }
    found.sort();
    found.dedup();
    found
}

/// True when code line `line` names `word` whole.
fn names_word(line: &str, word: &str) -> bool {
    line.match_indices(word)
        .any(|(at, _)| whole(line, at, word.len()))
}

/// True when code line `line` renames module `module`: `module as x`, or `self as x` in the braces after it.
fn renames(line: &str, module: &str) -> bool {
    line.match_indices(module).any(|(at, _)| {
        let rest = line[at + module.len()..].split(|c: char| !part(c));
        let words: Vec<&str> = rest.filter(|w| !w.is_empty()).collect();
        let renamed = words.first() == Some(&AS) || words.windows(2).any(|w| w == [SELF, AS]);
        whole(line, at, module.len()) && renamed
    })
}

/// The file and folder that a test module `name`, declared in `file`, may live in.
fn test_paths(file: &Path, name: &str) -> [PathBuf; 2] {
    let dir = child_dir(file);
    [dir.join(format!("{name}.{RUST_EXT}")), dir.join(name)]
}

/// The folder of `file`'s child modules: its own for `lib.rs`, `main.rs` and `mod.rs`, else one named after it.
fn child_dir(file: &Path) -> PathBuf {
    let dir = file.parent().unwrap_or(file);
    let name = file
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    match file.file_stem() {
        Some(stem) if !ROOT_FILES.contains(&name) => dir.join(stem),
        _ => dir.to_path_buf(),
    }
}

#[cfg(test)]
mod tests;
