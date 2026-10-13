//! F3: in the bridge crate, `unsafe` and its allow live only in the bridge folder, each use under a safety comment.

use std::fs;
use std::path::Path;

use super::files::{relative_place, walk_dir};
use super::{F3, Severity, Violation};
use crate::config::TidyConfig;
use crate::workspace::Workspace;

/// The only source files F3 reads.
const RUST_EXT: &str = "rs";
/// The keyword F3 looks for.
const UNSAFE: &str = "unsafe";
/// The lint that lets `unsafe` in when a lint list lowers it.
const LINT: &str = "unsafe_code";
/// Lint levels that let `unsafe_code` compile.
const ALLOWS: [&str; 3] = ["allow", "expect", "warn"];
/// Opens a line comment.
const LINE_COMMENT: &str = "//";
/// Opens a doc comment, which is no safety comment.
const DOC_COMMENTS: [&str; 2] = ["///", "//!"];
/// Starts an attribute, which may sit between a safety comment and its `unsafe`.
const ATTRIBUTE: &str = "#[";
/// Prefixes a raw string may carry before its `r`: byte and C strings.
const RAW_PREFIXES: [char; 2] = ['b', 'c'];

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
    findings(&text, in_bridges, &ws.tidy)
        .into_iter()
        .map(|(line, why)| Violation {
            rule: F3,
            place: format!("{place}:{line}"),
            message: why,
            severity: Severity::Error,
        })
        .collect()
}

/// F3's findings in `text`, one per line, as (line from 1, why); `in_bridges` says whether it may hold `unsafe`.
fn findings(text: &str, in_bridges: bool, tidy: &TidyConfig) -> Vec<(usize, String)> {
    let lines: Vec<&str> = text.lines().collect();
    let mut scan = Scan::default();
    let code: Vec<String> = lines.iter().map(|l| scan.code(l)).collect();
    let used = code.iter().enumerate().filter(|(_, c)| keywords(c) > 0);
    if in_bridges {
        let why = format!("unsafe without a {} comment above it", tidy.safety_tag);
        let bare = used.filter(|&(at, _)| !has_safety(&lines[..at], &tidy.safety_tag));
        return bare.map(|(at, _)| (at + 1, why.clone())).collect();
    }
    let mut at: Vec<usize> = used.map(|(at, _)| at).chain(allow_lines(&code)).collect();
    at.sort_unstable();
    at.dedup();
    let why = format!("unsafe code outside {}", tidy.bridge_dir);
    at.into_iter().map(|a| (a + 1, why.clone())).collect()
}

/// True when the comment block right above, past any attributes, holds a line starting with `tag`.
fn has_safety(above: &[&str], tag: &str) -> bool {
    let lines = above.iter().rev().map(|l| l.trim());
    let lines = lines.skip_while(|l| l.starts_with(ATTRIBUTE));
    let is_comment =
        |l: &&str| l.starts_with(LINE_COMMENT) && !DOC_COMMENTS.iter().any(|d| l.starts_with(d));
    lines.take_while(is_comment).any(|l| l.starts_with(tag))
}

/// How many `unsafe` keywords a line of code holds.
fn keywords(code: &str) -> usize {
    let words = code.match_indices(UNSAFE);
    words
        .filter(|&(at, _)| whole(code, at, UNSAFE.len()))
        .count()
}

/// The lines (from 0) where `unsafe_code` stands in an `allow`, `expect` or `warn` list, even one over lines.
fn allow_lines(code: &[String]) -> Vec<usize> {
    let text = code.join("\n");
    text.match_indices(LINT)
        .filter(|&(at, _)| whole(&text, at, LINT.len()) && in_allow(&text[..at]))
        .map(|(at, _)| text[..at].matches('\n').count())
        .collect()
}

/// True when the innermost list still open at the end of `before` belongs to a lint level in `ALLOWS`.
fn in_allow(before: &str) -> bool {
    let mut depth = 0usize;
    for (at, c) in before.char_indices().rev() {
        match c {
            ')' => depth += 1,
            '(' if depth == 0 => return ALLOWS.contains(&last_word(&before[..at])),
            '(' => depth -= 1,
            _ => {}
        }
    }
    false
}

/// The name that ends `text`, past any spaces.
fn last_word(text: &str) -> &str {
    let text = text.trim_end();
    let start = text
        .char_indices()
        .rev()
        .take_while(|&(_, c)| part(c))
        .last()
        .map_or(text.len(), |(i, _)| i);
    &text[start..]
}

/// True when `code[at..at + len]` is a whole word, not part of a longer name.
fn whole(code: &str, at: usize, len: usize) -> bool {
    let before = code[..at].chars().next_back().is_none_or(|c| !part(c));
    let after = code[at + len..].chars().next().is_none_or(|c| !part(c));
    before && after
}

/// True for a character a name may hold.
fn part(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// What the scan is inside between tokens.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum Text {
    /// Code.
    #[default]
    Code,
    /// A string with escapes.
    Str,
    /// A raw string, closed by a quote and this many `#`.
    Raw(usize),
}

/// Where the scan is between lines: inside block comments, which nest, or a string.
#[derive(Debug, Default)]
struct Scan {
    depth: usize,
    text: Text,
}

impl Scan {
    /// `line` without comments, with literals blanked; it never panics.
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
        if self.depth > 0 {
            return self.comment(c, next, i);
        }
        match self.text {
            Text::Str => return self.string(c, i),
            Text::Raw(n) => return self.raw(chars, i, n),
            Text::Code => {}
        }
        match (c, next) {
            ('/', Some('/')) => chars.len(),
            ('/', Some('*')) => {
                self.depth = 1;
                i + 2
            }
            ('"', _) => self.open(Text::Str, out, i + 1),
            ('r', _) => match raw_open(chars, i) {
                Some((n, end)) => self.open(Text::Raw(n), out, end),
                None => keep(out, c, i),
            },
            ('\'', _) => char_end(chars, i).map_or_else(|| keep(out, c, i), |end| end + 1),
            _ => keep(out, c, i),
        }
    }

    /// Inside block comments: `/*` goes one deeper and `*/` one out.
    fn comment(&mut self, c: char, next: Option<char>, i: usize) -> usize {
        match (c, next) {
            ('*', Some('/')) => {
                self.depth -= 1;
                i + 2
            }
            ('/', Some('*')) => {
                self.depth += 1;
                i + 2
            }
            _ => i + 1,
        }
    }

    /// Inside a string: a backslash skips the next character and a quote closes it.
    fn string(&mut self, c: char, i: usize) -> usize {
        match c {
            '\\' => i + 2,
            '"' => {
                self.text = Text::Code;
                i + 1
            }
            _ => i + 1,
        }
    }

    /// Inside a raw string: only a quote with its `n` hashes closes it.
    fn raw(&mut self, chars: &[char], i: usize, n: usize) -> usize {
        let closes = chars[i] == '"' && (1..=n).all(|k| chars.get(i + k) == Some(&'#'));
        if !closes {
            return i + 1;
        }
        self.text = Text::Code;
        i + 1 + n
    }

    /// Enters literal `text`, leaving a blank in `out`, and goes on at `next`.
    fn open(&mut self, text: Text, out: &mut String, next: usize) -> usize {
        self.text = text;
        out.push(' ');
        next
    }
}

/// Keeps `c` in `out` and moves on by one.
fn keep(out: &mut String, c: char, i: usize) -> usize {
    out.push(c);
    i + 1
}

/// For a raw string whose `r` is at `i` (`r"`, `r#"`, `br"`, `cr"`): its hash count and where its text starts.
/// `None` when the `r` ends a longer name or no quote follows its hashes.
fn raw_open(chars: &[char], i: usize) -> Option<(usize, usize)> {
    let before = i.checked_sub(1).map(|j| chars[j]);
    let prefixed = before.is_some_and(|b| RAW_PREFIXES.contains(&b))
        && i.checked_sub(2).is_none_or(|j| !part(chars[j]));
    if before.is_some_and(part) && !prefixed {
        return None;
    }
    let hashes = chars[i + 1..].iter().take_while(|&&c| c == '#').count();
    (chars.get(i + 1 + hashes) == Some(&'"')).then_some((hashes, i + 2 + hashes))
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
