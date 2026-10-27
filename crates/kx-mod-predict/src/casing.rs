//! Casing by language (spec §6 "Matching"): Turkish pairs İ with i and I with ı, where Unicode's default pairs I with
//! i. The rows live in `casing.toml`, and one function serves the pack builder and the engine.

use std::cmp::Reverse;
use std::collections::HashMap;
use std::sync::LazyLock;

use serde::Deserialize;

/// The casing rows, compiled in so they are never read from disk.
const TABLE: &str = include_str!("casing.toml");

/// Each language's casing, parsed once; a broken table leaves every language on the default, and a test keeps it
/// sound.
static LANGS: LazyLock<Option<HashMap<String, Casing>>> = LazyLock::new(|| parse(TABLE));

/// Unicode's default casing, for every language without rows.
static ROOT: Casing = Casing {
    lower: Vec::new(),
    title: Vec::new(),
    upper: Vec::new(),
    keep: Vec::new(),
};

/// One language's section of the table.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Section {
    /// Its `SpecialCasing.txt` rows.
    rows: Vec<Row>,
    /// The characters `fold.toml` folds that the language keeps apart.
    #[serde(default)]
    keep: Vec<char>,
}

/// One `SpecialCasing.txt` row: a text and its lower, title and upper forms.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Row {
    /// The text the row cases.
    text: String,
    /// Its lower form.
    lower: String,
    /// Its title form, for a word's first letter.
    title: String,
    /// Its upper form.
    upper: String,
}

/// How one language cases text.
#[derive(Debug, Default)]
pub struct Casing {
    /// Texts and their lower forms, longest first.
    lower: Vec<(String, String)>,
    /// Texts and their title forms, longest first.
    title: Vec<(String, String)>,
    /// Texts and their upper forms, longest first.
    upper: Vec<(String, String)>,
    /// The characters folding leaves as they are.
    keep: Vec<char>,
}

impl Casing {
    /// The casing of language `iso`; Unicode's default when the table has no rows for it.
    #[must_use]
    pub fn of(iso: &str) -> &'static Self {
        LANGS.as_ref().and_then(|l| l.get(iso)).unwrap_or(&ROOT)
    }

    /// Unicode's default casing, for text of no language, such as an email.
    #[must_use]
    pub fn root() -> &'static Self {
        &ROOT
    }

    /// `text` in lower case.
    #[must_use]
    pub fn lower(&self, text: &str) -> String {
        map(text, &self.lower, str::to_lowercase)
    }

    /// `text` in upper case.
    #[must_use]
    pub fn upper(&self, text: &str) -> String {
        map(text, &self.upper, str::to_uppercase)
    }

    /// `word` with its first letter in capitals and the rest as it is.
    #[must_use]
    pub fn capital(&self, word: &str) -> String {
        let row = self
            .title
            .iter()
            .find(|(from, _)| word.starts_with(from.as_str()));
        if let Some((from, to)) = row {
            return format!("{to}{}", word.get(from.len()..).unwrap_or_default());
        }
        first_upper(word)
    }

    /// True when folding leaves `c` as it is in this language.
    #[must_use]
    pub fn keeps(&self, c: char) -> bool {
        self.keep.contains(&c)
    }
}

/// `word` with its first letter in Unicode's default capital and the rest as it is.
fn first_upper(word: &str) -> String {
    let mut chars = word.chars();
    let first = chars.next().into_iter().flat_map(char::to_uppercase);
    first.chain(chars).collect()
}

/// `text` with each of `rows`' texts replaced, longest first, and the runs between them cased by `default`.
fn map(text: &str, rows: &[(String, String)], default: fn(&str) -> String) -> String {
    if rows.is_empty() {
        return default(text);
    }
    let mut out = String::with_capacity(text.len());
    let (mut from, mut at) = (0, 0);
    while let Some(tail) = text.get(at..).filter(|t| !t.is_empty()) {
        if let Some((key, to)) = rows.iter().find(|(key, _)| tail.starts_with(key.as_str())) {
            out.push_str(&default(text.get(from..at).unwrap_or_default()));
            out.push_str(to);
            at += key.len();
            from = at;
        } else {
            at += tail.chars().next().map_or(1, char::len_utf8);
        }
    }
    out.push_str(&default(text.get(from..).unwrap_or_default()));
    out
}

/// The table in `text`; `None` when it does not parse or a row's text is empty.
fn parse(text: &str) -> Option<HashMap<String, Casing>> {
    let sections: HashMap<String, Section> = toml::from_str(text).ok()?;
    let each = |(iso, section): (String, Section)| Some((iso, casing(section)?));
    sections.into_iter().map(each).collect()
}

/// The casing of one section, each form's rows longest first; `None` when a row's text is empty. A row enters a
/// form's table only where that form differs from Unicode's default, so the default cases whole runs in context
/// (Greek final sigma).
fn casing(section: Section) -> Option<Casing> {
    if section.rows.iter().any(|r| r.text.is_empty()) {
        return None;
    }
    let pairs = |form: fn(&Row) -> &String, default: fn(&str) -> String| {
        let mut out: Vec<(String, String)> = (section.rows.iter())
            .filter(|r| *form(r) != default(&r.text))
            .map(|r| (r.text.clone(), form(r).clone()))
            .collect();
        out.sort_by_key(|(from, _)| Reverse(from.len()));
        out
    };
    Some(Casing {
        lower: pairs(|r| &r.lower, str::to_lowercase),
        title: pairs(|r| &r.title, first_upper),
        upper: pairs(|r| &r.upper, str::to_uppercase),
        keep: section.keep,
    })
}

#[cfg(test)]
mod tests;
