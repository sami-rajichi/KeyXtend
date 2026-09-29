//! Word error rate of a transcript against the sentence read aloud, after the usual EN, FR and AR normalising.
#![cfg(windows)]

use spike_core::legend;

use crate::stats;

/// Tatweel only stretches a word.
const TATWEEL: char = '\u{0640}';
/// Arabic letter forms that transcripts write either way: hamza on alef, madda, alef maqsura, ta marbuta.
const FOLD: [(char, char); 5] = [
    ('\u{0623}', '\u{0627}'),
    ('\u{0625}', '\u{0627}'),
    ('\u{0622}', '\u{0627}'),
    ('\u{0649}', '\u{064A}'),
    ('\u{0629}', '\u{0647}'),
];

/// Arabic marks and tatweel: not letters, so dropped before words are compared.
fn is_mark(c: char) -> bool {
    c == TATWEEL || legend::is_mark(c)
}

fn fold(c: char) -> char {
    FOLD.iter()
        .find(|(from, _)| *from == c)
        .map_or(c, |&(_, to)| to)
}

/// The words of `s`: lower case, no punctuation (so French elisions split), Arabic marks dropped and letter forms folded.
pub fn words(s: &str) -> Vec<String> {
    let clean: String = s
        .chars()
        .filter(|&c| !is_mark(c))
        .map(|c| if c.is_alphanumeric() { fold(c) } else { ' ' })
        .collect();
    clean
        .to_lowercase()
        .split_whitespace()
        .map(str::to_string)
        .collect()
}

/// Word error rate of `heard` against `reference`: word edits over reference words; `None` for an empty reference.
pub fn wer(reference: &str, heard: &str) -> Option<f64> {
    let (r, h) = (words(reference), words(heard));
    (!r.is_empty()).then(|| stats::ratio(distance(&r, &h), r.len()))
}

/// Fewest word insertions, deletions and substitutions that turn `a` into `b`.
fn distance(a: &[String], b: &[String]) -> usize {
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, x) in a.iter().enumerate() {
        let mut cur = vec![i + 1];
        for (j, y) in b.iter().enumerate() {
            let swap = prev[j] + usize::from(x != y);
            cur.push(swap.min(prev[j + 1] + 1).min(cur[j] + 1));
        }
        prev = cur;
    }
    prev[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_words_score_zero_whatever_the_case_and_punctuation() {
        assert_eq!(
            wer("Please open the file.", "please, open the FILE"),
            Some(0.0)
        );
    }

    #[test]
    fn one_wrong_word_in_four_is_a_quarter() {
        assert_eq!(wer("open the file now", "open a file now"), Some(0.25));
        assert_eq!(
            wer("open the file now", "open file now"),
            Some(0.25),
            "a missing word"
        );
        assert_eq!(wer("open the file now", ""), Some(1.0));
    }

    #[test]
    fn french_elisions_split_into_words() {
        assert_eq!(
            wer("Merci d'ouvrir le fichier", "merci d’ouvrir le fichier"),
            Some(0.0)
        );
        assert_eq!(words("l'envoyer").len(), 2);
    }

    #[test]
    fn arabic_marks_and_letter_forms_do_not_count_as_errors() {
        assert_eq!(
            wer(
                "من فضلك افتح الملف وأرسله لي غدًا.",
                "من فضلك افتح الملف وارسله لي غدا"
            ),
            Some(0.0)
        );
        assert_eq!(
            wer("إلى المدرسة", "الي المدرسه"),
            Some(0.0),
            "hamza, alef maqsura and ta marbuta"
        );
    }

    #[test]
    fn an_empty_reference_has_no_rate() {
        assert_eq!(wer(" .", "anything"), None);
    }
}
