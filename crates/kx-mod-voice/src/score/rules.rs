//! The compiled-in scoring rules (`score.toml`): each language's casing, the marks it drops, the letters it joins, the
//! characters it maps and the words that count as another. A bad table gives no count; it never panics.

use std::collections::BTreeMap;
use std::sync::LazyLock;

use kx_module_api::Zeroizing;
use kx_module_api::ports::fits_iso;
use kx_module_api::text::{Casing, apostrophe, fold, is_apostrophe};
use serde::Deserialize;
use thiserror::Error;

/// The rules, read and checked at their first use.
static RULES: LazyLock<Result<Rules, ScoreError>> =
    LazyLock::new(|| Rules::parse(include_str!("../../score.toml")));

/// The compiled-in scoring rules, read and checked once.
///
/// # Errors
/// The table's first fault; Compare then shows no count.
pub fn rules() -> Result<&'static Rules, &'static ScoreError> {
    RULES.as_ref()
}

/// Why a rules table was refused.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum ScoreError {
    /// The text is not a rules table.
    #[error("the scoring rules do not parse: {0}")]
    Parse(String),
    /// A language's key is no ISO 639 code.
    #[error("the scoring language {0} is no ISO 639 code")]
    Lang(String),
    /// A language's marks run backwards, or a joined pair is not two characters.
    #[error("a mark span or joined pair of {0} is malformed")]
    Rule(String),
    /// A counted word or one of its forms is not one word, or it has no form.
    #[error("a counted word of {0} is not one word")]
    Words(String),
}

/// The table as written.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Raw {
    /// The rules every language takes.
    #[serde(default)]
    base: RawBase,
    /// Each language's own rules.
    #[serde(default)]
    langs: BTreeMap<String, RawLang>,
}

/// The rules every language takes, as written.
#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawBase {
    /// Characters read as the shared table's apostrophe, beside its own forms.
    #[serde(default)]
    apostrophes: Vec<char>,
}

/// One language's rules, as written.
#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawLang {
    /// The block whose characters the shared table drops are dropped here too.
    marks: Option<Span>,
    /// Two characters and the one letter they are joined into.
    #[serde(default)]
    join: BTreeMap<String, char>,
    /// Characters replaced.
    #[serde(default)]
    map: BTreeMap<char, String>,
    /// Words that count as any of their forms, such as a digit for its number words.
    #[serde(default)]
    words: BTreeMap<String, Vec<String>>,
}

/// A block of characters, both ends included.
#[derive(Copy, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Span {
    /// The first character.
    from: char,
    /// The last character.
    to: char,
}

/// One language's checked rules.
#[derive(Debug, Default)]
struct Lang {
    /// Characters removed.
    drop: Vec<char>,
    /// Two characters and the letter they join into.
    join: BTreeMap<(char, char), char>,
    /// Characters replaced.
    map: BTreeMap<char, String>,
    /// Folded words and the folded forms each counts as.
    words: BTreeMap<String, Vec<String>>,
}

/// The checked rules.
#[derive(Debug)]
pub struct Rules {
    /// Characters read as the apostrophe in every language.
    apostrophes: Vec<char>,
    /// Each language's own rules.
    langs: BTreeMap<String, Lang>,
}

impl Rules {
    /// Reads and checks a rules table; each counted word and form is folded once here.
    ///
    /// # Errors
    /// The first fault found.
    pub fn parse(text: &str) -> Result<Self, ScoreError> {
        let raw: Raw = toml::from_str(text).map_err(|e| ScoreError::Parse(e.to_string()))?;
        if let Some(bad) = raw.langs.keys().find(|l| !fits_iso(l)) {
            return Err(ScoreError::Lang(bad.clone()));
        }
        let mut said = BTreeMap::new();
        let mut rules = Self {
            apostrophes: raw.base.apostrophes,
            langs: BTreeMap::new(),
        };
        for (iso, lang) in raw.langs {
            let own = Lang::new(&iso, lang.marks, lang.join, lang.map)
                .ok_or_else(|| ScoreError::Rule(iso.clone()))?;
            said.insert(iso.clone(), lang.words);
            rules.langs.insert(iso, own);
        }
        for (iso, words) in said {
            let folded = rules
                .counted(&iso, &words)
                .ok_or_else(|| ScoreError::Words(iso.clone()))?;
            if let Some(own) = rules.langs.get_mut(&iso) {
                own.words = folded;
            }
        }
        Ok(rules)
    }

    /// True when `lang` has rules of its own, so Compare counts its words.
    #[must_use]
    pub fn has(&self, lang: &str) -> bool {
        self.langs.contains_key(lang)
    }

    /// True when words `a` and `b` read alike in `lang`: the same, or a counted word and one of its forms.
    pub(super) fn same(&self, a: &str, b: &str, lang: &str) -> bool {
        let words = self.langs.get(lang).map(|l| &l.words);
        let counts = |from: &str, to: &str| {
            let forms = words.and_then(|w| w.get(from));
            forms.is_some_and(|f| f.iter().any(|form| form == to))
        };
        a == b || counts(a, b) || counts(b, a)
    }

    /// `text` lower-cased by `lang`, its marks dropped, letters joined and characters mapped, then cut into words.
    pub(super) fn tokens(&self, text: &str, lang: &str) -> Vec<Zeroizing<String>> {
        let own = self.langs.get(lang);
        let cased = Zeroizing::new(Casing::of(lang).lower(text));
        let mut flat = Zeroizing::new(String::with_capacity(cased.len()));
        for c in cased.chars() {
            self.push(c, own, &mut flat);
        }
        let apos = apostrophe();
        let inside = |c: char| c.is_alphanumeric() || Some(c) == apos;
        let trim = |w: &'_ str| w.trim_matches(|c| Some(c) == apos).to_owned();
        let words = flat
            .split(|c| !inside(c))
            .map(trim)
            .filter(|w| !w.is_empty());
        words.map(Zeroizing::new).collect()
    }

    /// `words` folded in `lang`, each word and form one word and each word with a form; `None` when one is not.
    fn counted(
        &self,
        lang: &str,
        words: &BTreeMap<String, Vec<String>>,
    ) -> Option<BTreeMap<String, Vec<String>>> {
        let one = |text: &str| {
            let mut tokens = self.tokens(text, lang);
            (tokens.len() == 1)
                .then(|| tokens.pop())
                .flatten()
                .map(|t| t.as_str().to_owned())
        };
        let each = |(from, forms): (&String, &Vec<String>)| {
            let folded: Vec<String> = forms.iter().map(|f| one(f)).collect::<Option<_>>()?;
            Some((one(from)?, folded)).filter(|(_, f)| !f.is_empty())
        };
        words.iter().map(each).collect()
    }

    /// Writes `c` after `out`: nothing, joined with the last letter, as the apostrophe, its replacement, or itself.
    fn push(&self, c: char, own: Option<&Lang>, out: &mut String) {
        if own.is_some_and(|l| l.drop.contains(&c)) {
            return;
        }
        let last = out.chars().next_back();
        if let Some(&joined) = own.zip(last).and_then(|(l, last)| l.join.get(&(last, c))) {
            out.pop();
            out.push(joined);
            return;
        }
        let apos = apostrophe().filter(|_| is_apostrophe(c) || self.apostrophes.contains(&c));
        match (apos, own.and_then(|l| l.map.get(&c))) {
            (Some(apos), _) => out.push(apos),
            (None, Some(to)) => out.push_str(to),
            (None, None) => out.push(c),
        }
    }
}

impl Lang {
    /// Language `iso`'s rules: the shared table's drops within `marks`, the joined pairs and the map; `None` for a span
    /// that runs backwards or a pair that is not two characters.
    fn new(
        iso: &str,
        marks: Option<Span>,
        join: BTreeMap<String, char>,
        map: BTreeMap<char, String>,
    ) -> Option<Self> {
        let drop = marks.map_or(Some(Vec::new()), |span| shared_drops(iso, span))?;
        let pairs = join.into_iter().map(|(two, to)| Some((pair(&two)?, to)));
        Some(Self {
            drop,
            join: pairs.collect::<Option<_>>()?,
            map,
            words: BTreeMap::new(),
        })
    }
}

/// The characters of `span` that the shared fold table drops in language `iso`; `None` when it runs backwards.
fn shared_drops(iso: &str, span: Span) -> Option<Vec<char>> {
    let casing = Casing::of(iso);
    let mut buf = [0; char::MAX_LEN_UTF8];
    let dropped = |c: &char| fold(c.encode_utf8(&mut buf), casing).is_empty();
    (span.from <= span.to).then(|| (span.from..=span.to).filter(dropped).collect())
}

/// The two characters of `two`; `None` for any other count.
fn pair(two: &str) -> Option<(char, char)> {
    let mut chars = two.chars();
    match (chars.next(), chars.next(), chars.next()) {
        (Some(a), Some(b), None) => Some((a, b)),
        _ => None,
    }
}
