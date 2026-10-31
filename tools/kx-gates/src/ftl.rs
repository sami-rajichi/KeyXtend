//! kx-ui's texts as the product ships them, in English, French and Arabic, for gates that know a window by its title
//! or read a word or a number back out of an accessible name.
#![cfg(windows)]

/// Starts a comment line.
const COMMENT: char = '#';
/// Starts an attribute, and parts a message from it in a key.
const ATTR: char = '.';
/// Starts a term's name.
const TERM: char = '-';
/// Starts a variable's name.
const VAR: char = '$';
/// Opens and closes a string literal placeable.
const QUOTE: char = '"';
/// Starts an escape inside a string literal, which this reader never reads.
const ESCAPE: char = '\\';

/// Each language's tag, main file and clipboard panel file, which kx-ui loads into one bundle; workspace paths from
/// this file, all of kx-ui's (checked).
const FILES: [(&str, &str, &str); 3] = [
    (
        "en",
        include_str!("../../../crates/kx-ui/i18n/en.ftl"),
        include_str!("../../../crates/kx-ui/i18n/clip/en.ftl"),
    ),
    (
        "fr",
        include_str!("../../../crates/kx-ui/i18n/fr.ftl"),
        include_str!("../../../crates/kx-ui/i18n/clip/fr.ftl"),
    ),
    (
        "ar",
        include_str!("../../../crates/kx-ui/i18n/ar.ftl"),
        include_str!("../../../crates/kx-ui/i18n/clip/ar.ftl"),
    ),
];

/// Each language's tag and its files as one text, as kx-ui's bundle holds them.
fn bundles() -> impl Iterator<Item = (&'static str, String)> {
    FILES
        .iter()
        .map(|(lang, main, clip)| (*lang, format!("{main}\n{clip}")))
}

/// The text of `key` in `ftl`, with each `{ -term }` filled in: `group.name` names a message's attribute, a plain name
/// a message or a term; none when it is missing.
pub fn text(ftl: &str, key: &str) -> Option<String> {
    fill(ftl, written(ftl, key)?)
}

/// The value of `key` in `ftl` as written, on one line; none when it is missing or spans lines.
fn written<'a>(ftl: &'a str, key: &str) -> Option<&'a str> {
    let (msg, attr) = key
        .split_once(ATTR)
        .map_or((key, None), |(m, a)| (m, Some(a)));
    let mut current = "";
    for line in ftl.lines() {
        let entry = line.trim();
        if entry.starts_with(COMMENT) {
            continue;
        }
        let Some((name, value)) = entry.split_once('=') else {
            continue;
        };
        let (name, value) = (name.trim(), value.trim());
        let wanted = if line.starts_with(char::is_whitespace) {
            current == msg && attr.is_some() && name.strip_prefix(ATTR) == attr
        } else {
            current = name;
            attr.is_none() && name == msg
        };
        if wanted {
            return (!value.is_empty()).then_some(value);
        }
    }
    None
}

/// The text of string literal `term`, quotes and all, when it holds no escape; none for anything else.
fn literal(term: &str) -> Option<&str> {
    let inner = term.strip_prefix(QUOTE)?.strip_suffix(QUOTE)?;
    (!inner.contains([QUOTE, ESCAPE])).then_some(inner)
}

/// `value` with each `{ -term }` filled from `ftl` and each `{"literal"}` by its text; none for any other placeable or
/// a term that holds one.
fn fill(ftl: &str, value: &str) -> Option<String> {
    let mut out = String::new();
    let mut rest = value;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let close = open + rest[open..].find('}')?;
        let term = rest[open + 1..close].trim();
        let named = || {
            let found = term.starts_with(TERM).then(|| written(ftl, term));
            found.flatten().filter(|t| !t.contains('{'))
        };
        let filled = literal(term).or_else(named)?;
        out.push_str(filled);
        rest = &rest[close + 1..];
    }
    out.push_str(rest);
    Some(out)
}

/// The text of `key` in `ftl` around its one placeable `{ $var }`: what comes before it and after it, terms filled;
/// none when the key, the placeable or a term is missing.
fn split(ftl: &str, key: &str, var: &str) -> Option<(String, String)> {
    let value = written(ftl, key)?;
    let mut from = 0;
    while let Some(open) = value[from..].find('{').map(|at| from + at) {
        let close = open + value[open..].find('}')?;
        if value[open + 1..close].trim().strip_prefix(VAR) == Some(var) {
            return Some((fill(ftl, &value[..open])?, fill(ftl, &value[close + 1..])?));
        }
        from = close + 1;
    }
    None
}

/// The text of `key` in every shipped language split around `{ $var }`, in file order; an error naming the first
/// language that lacks it.
pub fn around(key: &str, var: &str) -> Result<Vec<(String, String)>, String> {
    bundles()
        .map(|(lang, ftl)| {
            split(&ftl, key, var).ok_or_else(|| format!("kx-ui has no {key} with ${var} in {lang}"))
        })
        .collect()
}

/// The text of `key` in every shipped language, in file order; an error naming the first language that lacks it.
pub fn every(key: &str) -> Result<Vec<String>, String> {
    bundles()
        .map(|(lang, ftl)| text(&ftl, key).ok_or_else(|| format!("kx-ui has no {key} in {lang}")))
        .collect()
}

/// One piece of a text: words as shown, or the place of a variable, by its name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Piece {
    /// Words, terms and literals filled.
    Words(String),
    /// A variable's place.
    Var(String),
}

/// The text of `key` in `ftl` cut at its variables, terms and literals filled; none when it is missing or holds a
/// placeable of another kind.
fn pieces(ftl: &str, key: &str) -> Option<Vec<Piece>> {
    let (mut rest, mut out) = (written(ftl, key)?, Vec::new());
    let words = |w: String| (!w.is_empty()).then_some(Piece::Words(w));
    while let Some(open) = rest.find('{') {
        let close = open + rest[open..].find('}')?;
        out.extend(words(fill(ftl, &rest[..open])?));
        match rest[open + 1..close].trim().strip_prefix(VAR) {
            Some(var) => out.push(Piece::Var(var.to_string())),
            None => out.extend(words(fill(ftl, &rest[open..=close])?)),
        }
        rest = &rest[close + 1..];
    }
    out.extend(words(fill(ftl, rest)?));
    Some(out)
}

/// The text of `key` in every shipped language cut at its variables, in file order; an error naming the first
/// language that lacks it.
pub fn shapes(key: &str) -> Result<Vec<Vec<Piece>>, String> {
    bundles()
        .map(|(lang, ftl)| pieces(&ftl, key).ok_or_else(|| format!("kx-ui has no {key} in {lang}")))
        .collect()
}

#[cfg(test)]
mod tests;
