//! Path and folder-name rules shared by the dev tools.

use std::path::{Component, Path, PathBuf};

use super::DevError;

/// Characters Windows forbids in a folder name, both separators, and PowerShell wildcards.
const FORBIDDEN: &str = "\\/:*?\"<>|[]`";
/// Windows device names, which no folder may use, with or without an extension.
const DEVICES: [&str; 4] = ["CON", "PRN", "AUX", "NUL"];
/// Device-name prefixes that take one digit (`COM1`, `LPT9`, `COM¹`).
const NUMBERED: [&str; 2] = ["COM", "LPT"];
/// The digits Windows accepts after a numbered device prefix, superscripts included.
const DEVICE_DIGITS: &str = "123456789¹²³";
/// Separator of the relative folders in the settings.
const SETTINGS_SEP: char = '/';

/// A path as text for messages and program arguments.
pub(crate) fn text(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

/// True if `name` is one plain folder or file name that Windows keeps as written.
pub(crate) fn is_plain_name(name: &str) -> bool {
    !name.trim().is_empty()
        && !name.ends_with(['.', ' '])
        && !name
            .chars()
            .any(|c| c.is_control() || FORBIDDEN.contains(c))
        && !is_device(name)
}

/// True if `name`, before its first dot, is a Windows device name.
fn is_device(name: &str) -> bool {
    let stem = name
        .split('.')
        .next()
        .unwrap_or_default()
        .to_ascii_uppercase();
    DEVICES.contains(&stem.as_str())
        || NUMBERED.iter().any(|prefix| {
            stem.strip_prefix(prefix).is_some_and(|rest| {
                let mut chars = rest.chars();
                matches!((chars.next(), chars.next()), (Some(d), None) if DEVICE_DIGITS.contains(d))
            })
        })
}

/// True if `rel` is a relative, `/`-separated path of plain names.
pub(crate) fn is_relative_plain(rel: &str) -> bool {
    rel.split(SETTINGS_SEP).all(is_plain_name)
}

/// `base` joined with each part of the `/`-separated settings path `rel`.
pub(crate) fn join_rel(base: PathBuf, rel: &str) -> PathBuf {
    rel.split(SETTINGS_SEP)
        .fold(base, |acc, part| acc.join(part))
}

/// True if `path` lies strictly inside `dir`, ignoring letter case.
///
/// `dir` must be absolute and name at least one folder, and neither may have `.` or `..` steps.
pub(crate) fn is_under(path: &Path, dir: &Path) -> bool {
    let has_folder = dir.components().any(|c| matches!(c, Component::Normal(_)));
    if !dir.is_absolute() || !has_folder {
        return false;
    }
    match (folder_names(path), folder_names(dir)) {
        (Some(p), Some(d)) => p.len() > d.len() && p.starts_with(&d),
        _ => false,
    }
}

/// True if the two folders are the same or one lies inside the other.
///
/// A path with a `.` or `..` step cannot be compared, so it counts as overlapping.
pub(crate) fn overlaps(a: &Path, b: &Path) -> bool {
    match (folder_names(a), folder_names(b)) {
        (Some(x), Some(y)) => x == y || is_under(a, b) || is_under(b, a),
        _ => true,
    }
}

/// The lower-cased components of `path`, or `None` if it has a `.` or `..` step.
fn folder_names(path: &Path) -> Option<Vec<String>> {
    path.components()
        .map(|c| match c {
            Component::CurDir | Component::ParentDir => None,
            other => Some(other.as_os_str().to_string_lossy().to_lowercase()),
        })
        .collect()
}

/// `path` rebuilt from its parts, which drops a trailing separator.
pub(crate) fn normalize(path: &Path) -> PathBuf {
    path.components().collect()
}

/// Total size in bytes of the files under `dir`; a link or junction is refused, since a copy would follow it.
pub(crate) fn folder_size(dir: &Path) -> Result<u64, DevError> {
    let mut total = 0;
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let meta = std::fs::symlink_metadata(entry.path())?;
        if meta.file_type().is_symlink() {
            return Err(DevError::Link(text(&entry.path())));
        }
        if meta.is_dir() {
            total += folder_size(&entry.path())?;
        } else if meta.is_file() {
            total += meta.len();
        }
    }
    Ok(total)
}

#[cfg(test)]
mod tests;
