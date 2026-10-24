//! The lines `words list` and `words build` print for a word list: the counts, the coverage line and one line per
//! rule that did something.

use super::cands::Tally;
use super::list::WordList;
use crate::text::plural;

/// The noun the report counts.
const WORD: &str = "word";
/// A share as a percentage.
const PERCENT: f64 = 100.0;

/// The lines printed for `iso`: the counts, the coverage line, one line per rule that did something, and a note
/// when no text was given to a checked list.
pub(crate) fn report(iso: &str, list: &WordList) -> Vec<String> {
    let caps = list.words.iter().filter(|w| w.capital).count();
    let low = list.words.iter().map(|w| w.freq).min().unwrap_or_default();
    let high = list.words.iter().map(|w| w.freq).max().unwrap_or_default();
    let (kept, read) = (list.words.len(), list.read);
    let covers = list.covers * PERCENT;
    let mut lines = vec![format!(
        "{iso}: {kept} {} kept ({caps} capitalised, frequency {low}..={high}) of {read} in wordfreq, covering \
         {covers:.1}% of its frequency mass",
        plural(kept, WORD)
    )];
    let (cut, budget) = list.cut;
    if cut > 0 {
        lines.push(format!(
            "budget {iso}: the {cut} rarest {} cut to fit {budget} bytes",
            plural(cut, WORD)
        ));
    }
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
    rules
        .iter()
        .filter(|(_, t)| t.count > 0)
        .map(|(name, tally)| tally_line(name, iso, tally))
        .collect()
}

/// The line of one rule's `tally`: how many words, then the first ones.
pub(crate) fn tally_line(name: &str, iso: &str, tally: &Tally) -> String {
    let (count, first) = (tally.count, &tally.first);
    let noun = plural(count, WORD);
    if first.is_empty() {
        return format!("{name} {iso}: {count} {noun}");
    }
    let shown = first.join(", ");
    format!(
        "{name} {iso}: {count} {noun}; first {}: {shown}",
        first.len()
    )
}
