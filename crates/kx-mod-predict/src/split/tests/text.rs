//! The tokenizer: sentences, words, elisions, whole stems, and what parts each word from the next.

use super::super::{Splitter, Token};

/// The tests' own sentence ends.
const ENDS: [char; 7] = ['.', '!', '?', '\u{061F}', '\u{2026}', '\n', '\r'];

/// One word as a test reads it: its text, then first, alone and joined.
type Seen = (String, bool, bool, bool);

/// The words of each sentence of `text`, where the list knows the words `known`.
fn words(text: &str, known: &[&str]) -> Vec<Vec<Seen>> {
    let split = Splitter {
        ends: &ENDS,
        apostrophe: '\'',
    };
    let is_known = |t: &str| known.contains(&t.to_lowercase().as_str());
    let text = split.normalise(text);
    let mut out = Vec::new();
    for sentence in split.sentences(&text) {
        let mut seen = Vec::new();
        split.words(sentence, &is_known, &mut |t: Token<'_>| {
            seen.push((t.text.to_string(), t.first, t.alone, t.joined));
        });
        if !seen.is_empty() {
            out.push(seen);
        }
    }
    out
}

/// The texts of `seen`.
fn texts(seen: &[Seen]) -> Vec<&str> {
    seen.iter().map(|s| s.0.as_str()).collect()
}

/// One expected word.
fn w(text: &str, first: bool, alone: bool, joined: bool) -> Seen {
    (text.to_string(), first, alone, joined)
}

#[test]
fn a_known_elision_comes_off_and_the_chain_runs_through_it() {
    let known = ["l'", "homme", "est", "là", "il", "part"];
    let got = words("L'homme est là. Il part !", &known);
    let first = vec![
        w("L'", true, true, false),
        w("homme", false, true, true),
        w("est", false, true, true),
        w("là", false, true, true),
    ];
    assert_eq!(
        got,
        [
            first,
            vec![w("Il", true, true, false), w("part", false, true, true)]
        ]
    );
}

#[test]
fn a_whole_word_the_list_knows_stays_whole_before_any_elision() {
    let known = ["qu'", "aujourd'hui", "don't", "l'", "jeu", "l'jeu"];
    let got = words("qu'aujourd'hui don't l'jeu", &known);
    let all: Vec<&str> = got.iter().flat_map(|s| texts(s)).collect();
    assert_eq!(all, ["qu'", "aujourd'hui", "don't", "l'jeu"]);
}

#[test]
fn quelqu_un_stays_whole_though_its_elision_is_known() {
    let known = ["quelqu'", "quelqu'un", "un", "jusqu'", "à"];
    let got = words("Quelqu'un jusqu'à", &known);
    let expected = vec![
        w("Quelqu'un", true, true, false),
        w("jusqu'", false, true, true),
        w("à", false, true, true),
    ];
    assert_eq!(got, [expected]);
}

#[test]
fn an_unknown_apostrophe_word_splits_into_glued_pieces() {
    let got = words("It's Muiriel's.", &["it's", "muiriel"]);
    let expected = vec![
        w("It's", true, true, false),
        w("Muiriel", false, false, true),
        w("s", false, false, false),
    ];
    assert_eq!(got, [expected]);
}

#[test]
fn hyphens_and_commas_break_the_chain_and_only_spaces_keep_alone() {
    let got = words("peut-être, oui « non »", &[]);
    let expected = vec![
        w("peut", true, false, false),
        w("être", false, false, false),
        w("oui", false, true, false),
        w("non", false, true, false),
    ];
    assert_eq!(got, [expected]);
}

#[test]
fn a_trailing_apostrophe_is_a_closing_quote() {
    let got = words("the boys' toys", &["the", "boys", "toys"]);
    assert_eq!(texts(got.first().unwrap()), ["the", "boys", "toys"]);
    assert!(
        got.first().unwrap().iter().all(|s| s.2),
        "every word stands alone"
    );
}

#[test]
fn the_typographic_apostrophe_reads_as_the_plain_one() {
    let got = words("l\u{2019}homme", &["l'", "homme"]);
    assert_eq!(texts(got.first().unwrap()), ["l'", "homme"]);
}

#[test]
fn arabic_sentences_end_at_the_arabic_question_mark_and_its_comma_breaks_chains() {
    let got = words("ذهبت إلى المدرسة، ثم عدت؟ نعم", &[]);
    let first = got.first().unwrap();
    assert_eq!(texts(first), ["ذهبت", "إلى", "المدرسة", "ثم", "عدت"]);
    let joined: Vec<bool> = first.iter().map(|s| s.3).collect();
    assert_eq!(joined, [false, true, true, false, true]);
    assert_eq!(texts(got.get(1).unwrap()), ["نعم"]);
}

#[test]
fn a_persian_joiner_stays_inside_its_word_and_goes_at_its_edge() {
    let got = words("کتاب\u{200C}ها را خواندم\u{200C} ", &[]);
    let first = got.first().unwrap();
    assert_eq!(texts(first), ["کتاب\u{200C}ها", "را", "خواندم"]);
    assert!(first.iter().all(|s| s.2), "each stands alone");
}

#[test]
fn harakat_and_digits_stay_inside_words() {
    let got = words("كتابٌ in 1999 e\u{0301}cole", &[]);
    assert_eq!(
        texts(got.first().unwrap()),
        ["كتابٌ", "in", "1999", "e\u{0301}cole"]
    );
}

#[test]
fn line_ends_end_sentences() {
    let got = words("one two\nthree\r\nfour", &[]);
    let sentences: Vec<Vec<&str>> = got.iter().map(|s| texts(s)).collect();
    assert_eq!(sentences, [vec!["one", "two"], vec!["three"], vec!["four"]]);
}

#[test]
fn a_mark_inside_a_word_ends_no_sentence_and_glues_its_neighbours() {
    let got = words("The U.S. Army won 3.14 points.Next", &[]);
    let sentences: Vec<Vec<&str>> = got.iter().map(|s| texts(s)).collect();
    let all = vec!["Army", "won", "3", "14", "points", "Next"];
    assert_eq!(sentences, [vec!["The", "U", "S"], all]);
    let alone: Vec<bool> = got.first().unwrap().iter().map(|s| s.2).collect();
    assert_eq!(
        alone,
        [true, false, false],
        "U and S are glued by the inner dot"
    );
}
