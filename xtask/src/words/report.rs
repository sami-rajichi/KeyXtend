//! The lines `words list` and `words build` print for a word list: the counts, the coverage line and one line per
//! rule that did something.

use super::cands::Tally;
use super::list::WordList;
use crate::text::plural;

/// The noun the report counts.
const WORD: &str = "word";

/// The lines printed for `iso`: the counts, the coverage line, one line per rule that did something, and a note
/// when no text was given to a checked list.
pub(crate) fn report(iso: &str, list: &WordList) -> Vec<String> {
    let caps = list.words.iter().filter(|w| w.capital).count();
    let low = list.words.iter().map(|w| w.freq).min().unwrap_or_default();
    let high = list.words.iter().map(|w| w.freq).max().unwrap_or_default();
    let kept = list.words.len();
    let mut lines = vec![format!(
        "{iso}: {kept} {} kept ({caps} capitalised, frequency {low}..={high}) of {} in wordfreq",
        plural(kept, WORD),
        list.read
    )];
    if let Some(c) = &list.coverage {
        lines.push(format!(
            "coverage {iso}: the dictionary rejected {} of the {} most frequent words; first {}: {}",
            c.rejected,
            c.top,
            c.first.len(),
            c.first.join(", ")
        ));
    }
    lines.extend(rule_lines(iso, list));
    if list.coverage.is_some() && !list.text {
        lines.push(format!(
            "text {iso}: none, so twins, pieces and stems are skipped"
        ));
    }
    lines
}

/// One line per rule that changed or dropped a word: how many, then the first ones.
fn rule_lines(iso: &str, list: &WordList) -> Vec<String> {
    let (f, t) = (&list.facts, &list.text_facts);
    let rules = [
        ("script", &f.script),
        ("elisions", &f.elisions),
        ("joins", &f.joins),
        ("merged", &f.merged),
        ("prefixes", &f.prefixes),
        ("twins", &t.twins),
        ("pieces", &t.pieces),
        ("stems", &t.stems),
        ("unseen", &t.unseen),
    ];
    let line = |(name, tally): &(&str, &Tally)| {
        let (count, first) = (tally.count, &tally.first);
        let noun = plural(count, WORD);
        let shown = first.join(", ");
        format!(
            "{name} {iso}: {count} {noun}; first {}: {shown}",
            first.len()
        )
    };
    rules
        .iter()
        .filter(|(_, t)| t.count > 0)
        .map(line)
        .collect()
}
