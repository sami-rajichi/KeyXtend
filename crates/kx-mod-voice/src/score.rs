//! The scorer (design "Compare models", ADR-0044 decision 7): words wrong of Compare's set text, and the words a card's
//! text differs by, after `score.toml`'s rules per language. Words sit in wiped buffers and never print.

mod rules;

pub use rules::{Rules, ScoreError, rules};

/// Words wrong of a set text.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Wrong {
    /// Words put in, left out or changed.
    pub words: u16,
    /// The set text's words.
    pub total: u16,
}

/// Words wrong in `heard` against the set text `set`, in `lang`; `None` for a language without rules, where Compare
/// shows no count.
#[must_use]
pub fn wrong(set: &str, heard: &str, lang: &str) -> Option<Wrong> {
    let rules = rules().ok().filter(|r| r.has(lang))?;
    let (set, heard) = (rules.tokens(set, lang), rules.tokens(heard, lang));
    Some(Wrong {
        words: count(edits(&set, &heard, |a, b| rules.same(a, b, lang))),
        total: count(set.len()),
    })
}

/// The words `a` and `b` differ by in `lang`, by the base rules when the language has none of its own.
#[must_use]
pub fn apart(a: &str, b: &str, lang: &str) -> u16 {
    let Ok(rules) = rules() else {
        return count(edits(&plain(a), &plain(b), |x, y| x == y));
    };
    let (a, b) = (rules.tokens(a, lang), rules.tokens(b, lang));
    count(edits(&a, &b, |x, y| rules.same(x, y, lang)))
}

/// `text` cut at whitespace, for a broken rules table, which a test rules out.
fn plain(text: &str) -> Vec<&str> {
    text.split_whitespace().collect()
}

/// `n` as a count a view carries, held at its top.
fn count(n: usize) -> u16 {
    u16::try_from(n).unwrap_or(u16::MAX)
}

/// The fewest words put in, left out or changed to turn `a` into `b`, where `same` says two words match.
fn edits<T>(a: &[T], b: &[T], same: impl Fn(&T, &T) -> bool) -> usize {
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, x) in a.iter().enumerate() {
        let mut diagonal = i;
        let mut left = i + 1;
        for (cell, y) in row.iter_mut().skip(1).zip(b) {
            let up = *cell;
            let next = (diagonal + usize::from(!same(x, y)))
                .min(up + 1)
                .min(left + 1);
            diagonal = up;
            left = next;
            *cell = next;
        }
        if let Some(first) = row.first_mut() {
            *first = i + 1;
        }
    }
    row.last().copied().unwrap_or_default()
}

#[cfg(test)]
mod tests;
