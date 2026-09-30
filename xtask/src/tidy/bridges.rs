//! F3: in the bridge crate, `unsafe` and its allow live only in the bridge folder, each use under a safety comment.

use std::fs;
use std::path::Path;

use super::files::{relative_place, walk_dir};
use super::{F3, Severity, Violation};
use crate::workspace::Workspace;

/// The only source files F3 reads.
const RUST_EXT: &str = "rs";
/// The keyword F3 looks for.
const UNSAFE: &str = "unsafe";
/// Attributes that let `unsafe` in, up to the lint's name.
const ALLOWS: [&str; 2] = ["allow(unsafe_code", "expect(unsafe_code"];
/// Opens a line comment.
const LINE_COMMENT: &str = "//";
/// Opens a doc comment, which is no safety comment.
const DOC_COMMENTS: [&str; 2] = ["///", "//!"];
/// Starts an attribute, which may sit between a safety comment and its `unsafe`.
const ATTRIBUTE: &str = "#[";

/// F3: every `unsafe` or allow of it in the bridge crate sits in `bridge_dir`, and each `unsafe` there follows
/// a `safety_tag` comment. A workspace without the bridge crate passes; other crates are left to the compiler.
pub(crate) fn f3_bridge_unsafe(ws: &Workspace) -> Vec<Violation> {
    let Some(krate) = ws.find_member(&ws.tidy.bridge_crate) else {
        return Vec::new();
    };
    let Some(dir) = krate.manifest_path.parent() else {
        return Vec::new();
    };
    let (mut files, mut ignored) = (Vec::new(), Vec::new());
    walk_dir(dir, &ws.root_dir, &ws.tidy, &mut files, &mut ignored);
    let bridges = dir.join(&ws.tidy.bridge_dir);
    files
        .iter()
        .filter(|p| p.extension().is_some_and(|e| e == RUST_EXT))
        .flat_map(|p| check_file(p, p.starts_with(&bridges), ws))
        .collect()
}

/// F3 violations in one file; `in_bridges` says whether it may hold `unsafe`. An unreadable file is F1's to report.
fn check_file(path: &Path, in_bridges: bool, ws: &Workspace) -> Vec<Violation> {
    let Ok(text) = fs::read_to_string(path) else {
        return Vec::new();
    };
    let place = relative_place(path, &ws.root_dir);
    let lines: Vec<&str> = text.lines().collect();
    let mut scan = Scan::default();
    let mut found = Vec::new();
    for (at, line) in lines.iter().enumerate() {
        let (keywords, allows) = uses(&scan.code(line));
        let why = if !in_bridges && keywords + allows > 0 {
            format!("unsafe code outside {}", ws.tidy.bridge_dir)
        } else if in_bridges && keywords > 0 && !has_safety(&lines[..at], &ws.tidy.safety_tag) {
            format!("unsafe without a {} comment above it", ws.tidy.safety_tag)
        } else {
            continue;
        };
        found.push(Violation {
            rule: F3,
            place: format!("{place}:{}", at + 1),
            message: why,
            severity: Severity::Error,
        });
    }
    found
}

/// True when the comment block right above, past any attributes, holds a line starting with `tag`.
fn has_safety(above: &[&str], tag: &str) -> bool {
    let lines = above.iter().rev().map(|l| l.trim());
    let lines = lines.skip_while(|l| l.starts_with(ATTRIBUTE));
    let is_comment =
        |l: &&str| l.starts_with(LINE_COMMENT) && !DOC_COMMENTS.iter().any(|d| l.starts_with(d));
    lines.take_while(is_comment).any(|l| l.starts_with(tag))
}

/// How many `unsafe` keywords and allows of `unsafe_code` a line of code holds.
fn uses(code: &str) -> (usize, usize) {
    let words = code
        .match_indices(UNSAFE)
        .filter(|&(at, _)| whole(code, at, UNSAFE.len()));
    let allows = ALLOWS.iter().map(|a| code.matches(a).count()).sum();
    (words.count(), allows)
}

/// True when `code[at..at + len]` is a whole word, not part of a longer name.
fn whole(code: &str, at: usize, len: usize) -> bool {
    let part = |c: char| c.is_alphanumeric() || c == '_';
    let before = code[..at].chars().next_back().is_none_or(|c| !part(c));
    let after = code[at + len..].chars().next().is_none_or(|c| !part(c));
    before && after
}

/// Where the scan is between lines: inside a block comment or a string.
#[derive(Debug, Default)]
struct Scan {
    in_block: bool,
    in_str: bool,
}

impl Scan {
    /// `line` without comments, with literals blanked; nested block comments and raw strings with quotes inside
    /// are not followed, but it never panics.
    fn code(&mut self, line: &str) -> String {
        let chars: Vec<char> = line.chars().collect();
        let mut out = String::new();
        let mut i = 0;
        while i < chars.len() {
            i = self.step(&chars, i, &mut out);
        }
        out
    }

    /// Reads the token at `i`, keeps it in `out` when it is code, and returns where the next one starts.
    fn step(&mut self, chars: &[char], i: usize, out: &mut String) -> usize {
        let (c, next) = (chars[i], chars.get(i + 1).copied());
        if self.in_block {
            self.in_block = !(c == '*' && next == Some('/'));
            return if self.in_block { i + 1 } else { i + 2 };
        }
        if self.in_str {
            self.in_str = c != '"';
            return if c == '\\' { i + 2 } else { i + 1 };
        }
        match (c, next) {
            ('/', Some('/')) => chars.len(),
            ('/', Some('*')) => {
                self.in_block = true;
                i + 2
            }
            ('"', _) => {
                self.in_str = true;
                out.push(' ');
                i + 1
            }
            ('\'', _) => char_end(chars, i).map_or_else(|| keep(out, c, i), |end| end + 1),
            _ => keep(out, c, i),
        }
    }
}

/// Keeps `c` in `out` and moves on by one.
fn keep(out: &mut String, c: char, i: usize) -> usize {
    out.push(c);
    i + 1
}

/// The closing quote of a char literal opening at `i`, such as `'a'` or `'\''`; `None` for a lifetime.
fn char_end(chars: &[char], i: usize) -> Option<usize> {
    let end = if chars.get(i + 1) == Some(&'\\') {
        (i + 3..chars.len()).find(|&j| chars[j] == '\'')?
    } else {
        i + 2
    };
    (chars.get(end) == Some(&'\'')).then_some(end)
}

#[cfg(test)]
mod tests;
