//! F1-F2: the file-size and unsafe-attribute rules from `ARCHITECTURE.md`.

use std::fs::{self, FileType};
use std::path::{Path, PathBuf};

use super::{F1, F2, Severity, Violation};
use crate::config::TidyConfig;
use crate::workspace::{Package, Target, Workspace};

/// F1: every scanned source file must stay within `warn_file_lines`/`max_file_lines`.
pub(crate) fn f1_file_length(ws: &Workspace) -> Vec<Violation> {
    let (files, mut violations) = walk_source_files(&ws.root_dir, &ws.tidy);
    violations.extend(files.iter().filter_map(|path| check_file_length(path, ws)));
    violations
}

/// Reads one file and reports a violation if its line count is over a limit.
fn check_file_length(path: &Path, ws: &Workspace) -> Option<Violation> {
    let place = relative_place(path, &ws.root_dir);
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(err) => return Some(f1_violation(place, err.to_string(), Severity::Error)),
    };
    let Ok(text) = std::str::from_utf8(&bytes) else {
        return Some(f1_violation(
            place,
            "not valid UTF-8".into(),
            Severity::Error,
        ));
    };
    let lines = count_lines(text);
    let severity = length_severity(lines, ws.tidy.warn_file_lines, ws.tidy.max_file_lines)?;
    Some(f1_violation(place, format!("{lines} lines"), severity))
}

/// Counts lines in `text`: CRLF and LF count the same, and a final line with no
/// trailing newline still counts.
fn count_lines(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }
    let newlines = text.bytes().filter(|&b| b == b'\n').count();
    if text.ends_with('\n') {
        newlines
    } else {
        newlines + 1
    }
}

/// The severity for a file of `lines` lines against `warn`/`max`, or `None` if it's fine.
///
/// Above `warn` up to and including `max` warns, which never fails tidy; above `max` errors.
fn length_severity(lines: usize, warn: usize, max: usize) -> Option<Severity> {
    if lines > max {
        Some(Severity::Error)
    } else if lines > warn {
        Some(Severity::Warning)
    } else {
        None
    }
}

/// Builds one F1 [`Violation`].
fn f1_violation(place: String, message: String, severity: Severity) -> Violation {
    Violation {
        rule: F1,
        place,
        message,
        severity,
    }
}

/// Recursively lists files under each of `tidy.scan_dirs` with an extension in
/// `tidy.extensions`, skipping any folder named in `tidy.skip_dirs`. A missing
/// scan dir is silent; any other read error becomes an F1 [`Violation`].
fn walk_source_files(root: &Path, tidy: &TidyConfig) -> (Vec<PathBuf>, Vec<Violation>) {
    let mut files = Vec::new();
    let mut violations = Vec::new();
    for dir in &tidy.scan_dirs {
        walk_dir(&root.join(dir), root, tidy, &mut files, &mut violations);
    }
    (files, violations)
}

/// Recursive step of [`walk_source_files`]; never panics on an unreadable directory.
fn walk_dir(
    dir: &Path,
    root: &Path,
    tidy: &TidyConfig,
    files: &mut Vec<PathBuf>,
    violations: &mut Vec<Violation>,
) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return,
        Err(err) => {
            violations.push(read_dir_violation(dir, root, &err));
            return;
        }
    };
    for entry in entries {
        match entry.and_then(|e| e.file_type().map(|kind| (e.path(), kind))) {
            Ok((path, kind)) => visit_entry(&path, kind, root, tidy, files, violations),
            Err(err) => violations.push(read_dir_violation(dir, root, &err)),
        }
    }
}

/// Adds `path` to `files` if it's a scanned source file, or recurses if it's a folder.
/// Links are skipped, because a folder link can loop back on itself.
fn visit_entry(
    path: &Path,
    kind: FileType,
    root: &Path,
    tidy: &TidyConfig,
    files: &mut Vec<PathBuf>,
    violations: &mut Vec<Violation>,
) {
    if kind.is_symlink() {
        return;
    }
    if kind.is_dir() {
        if !is_skipped(path, tidy) {
            walk_dir(path, root, tidy, files, violations);
        }
    } else if has_scanned_extension(path, tidy) {
        files.push(path.to_path_buf());
    }
}

/// Builds an F1 [`Violation`] for a directory that could not be read.
fn read_dir_violation(dir: &Path, root: &Path, err: &std::io::Error) -> Violation {
    f1_violation(relative_place(dir, root), err.to_string(), Severity::Error)
}

/// True if `dir`'s folder name is in `tidy.skip_dirs`.
fn is_skipped(dir: &Path, tidy: &TidyConfig) -> bool {
    dir.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|name| tidy.skip_dirs.iter().any(|s| s == name))
}

/// True if `path`'s extension is in `tidy.extensions`.
fn has_scanned_extension(path: &Path, tidy: &TidyConfig) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| tidy.extensions.iter().any(|e| e == ext))
}

/// F2: every non-platform crate's `root_kinds` target must carry `unsafe_attr`.
///
/// A test tool may carry `tool_unsafe_attr` instead.
pub(crate) fn f2_unsafe_attr(ws: &Workspace) -> Vec<Violation> {
    ws.members()
        .filter(|p| !p.name.starts_with(ws.tidy.platform_prefix.as_str()))
        .flat_map(|p| check_package(p, ws))
        .collect()
}

/// F2 violations in `package`'s crate roots.
fn check_package(package: &Package, ws: &Workspace) -> Vec<Violation> {
    let attrs = accepted_attrs(package, ws);
    root_targets(package, ws)
        .filter_map(|target| check_unsafe_attr(target, &attrs, ws))
        .collect()
}

/// The attributes `package`'s crate roots may carry, the tool one first for a tool.
fn accepted_attrs<'a>(package: &Package, ws: &'a Workspace) -> Vec<&'a str> {
    let tidy = &ws.tidy;
    if ws.is_tool(package) {
        vec![&tidy.tool_unsafe_attr, &tidy.unsafe_attr]
    } else {
        vec![&tidy.unsafe_attr]
    }
}

/// `package`'s targets whose kind is one of `tidy.root_kinds`.
fn root_targets<'a>(package: &'a Package, ws: &'a Workspace) -> impl Iterator<Item = &'a Target> {
    package
        .targets
        .iter()
        .filter(|t| t.kinds.iter().any(|k| ws.tidy.root_kinds.contains(k)))
}

/// Reads one target's root file and reports a violation if none of `attrs` is present.
fn check_unsafe_attr(target: &Target, attrs: &[&str], ws: &Workspace) -> Option<Violation> {
    let place = relative_place(&target.src_path, &ws.root_dir);
    let text = match fs::read_to_string(&target.src_path) {
        Ok(text) => text,
        Err(err) => return Some(f2_violation(place, err.to_string())),
    };
    if attrs.iter().any(|attr| has_active_line(&text, attr)) {
        None
    } else {
        Some(f2_violation(
            place,
            format!("missing {}", attrs.join(" or ")),
        ))
    }
}

/// Builds one F2 [`Violation`]; F2 is always an error.
fn f2_violation(place: String, message: String) -> Violation {
    Violation {
        rule: F2,
        place,
        message,
        severity: Severity::Error,
    }
}

/// True if some line of `text`, trimmed, equals `attr`, outside a `//` or `/* */` comment.
/// Block comments are tracked line by line; nesting is not supported, but this never panics.
fn has_active_line(text: &str, attr: &str) -> bool {
    let mut in_block = false;
    for raw_line in text.lines() {
        let line = raw_line.trim();
        if in_block {
            if line.contains("*/") {
                in_block = false;
            }
            continue;
        }
        if active_code(line, &mut in_block).trim() == attr {
            return true;
        }
    }
    false
}

/// The part of `line` before any `//` or `/*`; opens `in_block` if a block comment starts.
fn active_code<'a>(line: &'a str, in_block: &mut bool) -> &'a str {
    match (line.find("//"), line.find("/*")) {
        (Some(ss), Some(bs)) if bs < ss => block_prefix(line, bs, in_block),
        (Some(ss), _) => &line[..ss],
        (None, Some(bs)) => block_prefix(line, bs, in_block),
        (None, None) => line,
    }
}

/// The part of `line` before a block comment starting at `idx`; opens `in_block` if it
/// doesn't also close on this line.
fn block_prefix<'a>(line: &'a str, idx: usize, in_block: &mut bool) -> &'a str {
    if !line[idx..].contains("*/") {
        *in_block = true;
    }
    &line[..idx]
}

/// `path` relative to `root`, with forward slashes; used as-is if it's not under `root`.
fn relative_place(path: &Path, root: &Path) -> String {
    let rel = path.strip_prefix(root).unwrap_or(path);
    rel.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tool_tests;
