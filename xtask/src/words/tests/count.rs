//! The text counter: tokens find their candidates, chains count within a sentence only, the held-back share is
//! stable and never got, and lossy counting keeps the frequent chains under the cap.
#![allow(
    clippy::float_cmp,
    reason = "The chain counts here are sums of whole numbers and halves, exact in an f32."
)]

use kx_mod_predict::Casing;

use super::super::count::{
    ChainKey, Chains, Counted, Counter, KEY_LEN, NONE, Repeats, key as key_of,
};
use super::super::fixture::text::{cand, tatoeba, wiki};
use super::super::fixture::{TEXT, manifest_with};
use super::super::manifest::TextRules;
use super::super::vocab::Vocab;
use super::super::{tatoeba as tsv, wiki as dump};

/// The fixture manifest's text rules: chains of 2 and 3, Tatoeba weighing 4, sentences ending at `.` and `?`.
fn rules() -> TextRules {
    assert!(TEXT.contains("tatoeba_weight = 4"));
    manifest_with("").unwrap().text
}

/// The chain key of candidate places `ids`.
fn key(ids: &[u32]) -> ChainKey {
    let mut key = [NONE; KEY_LEN];
    key.iter_mut().zip(ids).for_each(|(slot, id)| *slot = *id);
    key
}

/// The count of chain `ids` in `got`.
fn chain(got: &Counted, ids: &[u32]) -> f32 {
    got.chains.table.get(&key(ids)).copied().unwrap_or_default()
}

/// The words the counter tests know, at places 0 to 4: the, cat, sat, down, I.
fn vocab() -> Vocab {
    let words = [["the"], ["cat"], ["sat"], ["down"], ["I"]];
    let cands: Vec<_> = (0..)
        .zip(words)
        .map(|(i, w)| cand(&w, 300 + i, 0))
        .collect();
    Vocab::new(&cands, Casing::root())
}

#[test]
fn a_token_finds_its_candidate_by_spelling_lower_case_or_fold_key() {
    let cands = [
        cand(&["l", "l'"], 200, 0),
        cand(&["London"], 400, 1),
        cand(&["أنا"], 300, 2),
        cand(&["a"], 210, 3),
        cand(&["à"], 220, 4),
    ];
    let vocab = Vocab::new(&cands, Casing::root());
    let found = |t: &str| vocab.find(t).map(|h| (h.cand, h.form.is_some()));
    assert_eq!(found("l'"), Some((0, true)));
    assert_eq!(found("L'"), Some((0, false)));
    assert_eq!(found("LONDON"), Some((1, false)));
    assert_eq!(
        found("انا"),
        Some((2, false)),
        "انا finds أنا by its fold key"
    );
    assert_eq!(
        (found("A"), found("À")),
        (Some((3, false)), Some((4, false)))
    );
    assert_eq!(found("zzz"), None);
}

#[test]
fn a_turkish_token_finds_its_candidate_by_turkish_casing() {
    let cands = [cand(&["ılık"], 200, 0), cand(&["ilik"], 210, 1)];
    let tr = Vocab::new(&cands, Casing::of("tr"));
    let found = |v: &Vocab, t: &str| v.find(t).map(|h| h.cand);
    assert_eq!((found(&tr, "ILIK"), found(&tr, "İLİK")), (Some(0), Some(1)));
    let en = Vocab::new(&cands, Casing::root());
    assert_eq!(
        found(&en, "ILIK"),
        Some(1),
        "Unicode's default lowers I to i"
    );
}

#[test]
fn turkish_text_counts_dotless_and_dotted_words_apart() {
    let cands = [cand(&["ılık"], 200, 0), cand(&["ilik"], 210, 1)];
    let (vocab, rules) = (Vocab::new(&cands, Casing::of("tr")), rules());
    let mut counter = Counter::new(&vocab, cands.len(), &rules, '\'');
    counter.add("ılık ılık ilik ılık", 1, false);
    let text = counter.finish().text;
    assert_eq!((text.alone("ılık"), text.alone("ilik")), (3, 1));
    assert_eq!((text.seen("ılık"), text.seen("ilik")), (2, 1));
}

#[test]
fn a_word_that_does_not_stand_alone_ends_the_chains_as_the_tracker_does() {
    let words = [["est"], ["ce"], ["que"], ["site"], ["com"], ["today"]];
    let cands: Vec<_> = (0..)
        .zip(words)
        .map(|(i, w)| cand(&w, 300 + i, 0))
        .collect();
    let (vocab, rules) = (Vocab::new(&cands, Casing::root()), rules());
    let mut counter = Counter::new(&vocab, cands.len(), &rules, '\'');
    counter.add("Est-ce que site.com today", 1, false);
    let got = counter.finish();
    assert_eq!(got.chains.table.len(), 0, "no chain into que or today");
    assert_eq!((got.text.total, got.text.alone("que")), (6, 1));
}

#[test]
fn chains_count_within_a_sentence_and_an_unknown_word_breaks_them() {
    let (vocab, rules) = (vocab(), rules());
    let mut counter = Counter::new(&vocab, 5, &rules, '\'');
    counter.add("The cat sat. Down the cat zzz sat down", 1, false);
    let got = counter.finish();
    let (the, cat, sat, down) = (0, 1, 2, 3);
    assert_eq!(chain(&got, &[the, cat]), 2.0);
    assert_eq!(chain(&got, &[the, cat, sat]), 1.0);
    assert_eq!(
        chain(&got, &[cat, sat]),
        1.0,
        "zzz breaks the second cat sat"
    );
    assert_eq!(
        chain(&got, &[sat, down]),
        1.0,
        "sat. Down spans two sentences"
    );
    assert_eq!(chain(&got, &[down, the, cat]), 1.0);
    assert_eq!(got.chains.table.len(), 6);
    assert_eq!(got.text.total, 9, "zzz is a token too");
}

#[test]
fn counts_are_weighted_and_spellings_count_alone_away_from_sentence_starts() {
    let (vocab, rules) = (vocab(), rules());
    let mut counter = Counter::new(&vocab, 5, &rules, '\'');
    counter.add("I sat. The cat and I, x-I", 4, false);
    let got = counter.finish();
    let text = &got.text;
    assert_eq!(text.total, 32);
    assert_eq!(
        (text.count("I"), text.seen("I"), text.alone("i")),
        (12, 4, 8),
        "the I glued to x counts, but stands alone nowhere"
    );
    assert_eq!(got.seen.first(), Some(&4), "the, at a start, still counts");
    assert_eq!(chain(&got, &[4, 2]), 4.0);
}

#[test]
fn spellings_sharing_a_fold_key_keep_their_own_counts() {
    let cands = [
        cand(&["c'"], 200, 0),
        cand(&["ç'"], 300, 1),
        cand(&["est"], 210, 2),
        cand(&["a"], 220, 3),
    ];
    let (vocab, rules) = (Vocab::new(&cands, Casing::root()), rules());
    let mut counter = Counter::new(&vocab, cands.len(), &rules, '\'');
    counter.add("C'est vrai. c'est ç'a", 1, false);
    let text = counter.finish().text;
    assert_eq!(
        (text.count("c'"), text.count("ç'")),
        (2, 1),
        "not 3 each by their fold key"
    );
}

#[test]
fn a_repeated_sentence_counts_once_for_words_and_chains() {
    let (vocab, rules) = (vocab(), rules());
    let mut counter = Counter::new(&vocab, 5, &rules, '\'');
    assert!(counter.add("The cat sat. The cat, sat? the cat sat.", 1, false));
    assert!(
        !counter.add("The cat sat.", 1, false),
        "a repeat counts nothing"
    );
    let got = counter.finish();
    assert_eq!(chain(&got, &[0, 1]), 2.0, "the lower-case the is no repeat");
    assert_eq!((got.text.total, got.seen.get(1)), (6, Some(&2)));
    let r = &got.repeats;
    assert_eq!((r.skipped, r.read, r.kept()), (2, 4, 2));
}

#[test]
fn the_key_is_the_letters_in_order_with_their_case() {
    assert_eq!(key_of("The cat, sat!"), key_of("Thecatsat"));
    assert_ne!(key_of("the cat sat"), key_of("The cat sat"));
    assert_ne!(key_of("cat sat"), key_of("sat cat"));
    assert_eq!(key_of("قال توم."), key_of("قال، توم"));
    assert_eq!(key_of("12 ?!"), None);
}

#[test]
fn past_the_cap_a_new_sentence_counts_but_is_not_remembered() {
    let mut repeats = Repeats::new(1);
    let seen: Vec<bool> = ["a", "a", "b", "b", "1"]
        .iter()
        .map(|s| repeats.first(s))
        .collect();
    assert_eq!(seen, [true, false, true, true, true]);
    assert_eq!((repeats.kept(), repeats.skipped), (1, 1));
}

#[test]
fn scaled_text_scales_the_chains_its_words_end_and_other_text_does_not() {
    let (vocab, rules) = (vocab(), rules());
    let mut counter = Counter::new(&vocab, 5, &rules, '\'');
    counter.scale(vec![1.0, 0.5, 1.0, 1.0, 1.0]);
    counter.add("The cat sat.", 2, true);
    counter.add("The cat sat down.", 2, false);
    let got = counter.finish();
    let (the, cat, sat) = (0, 1, 2);
    assert_eq!(chain(&got, &[the, cat]), 3.0, "2 × 0.5 + 2");
    assert_eq!(
        chain(&got, &[cat, sat]),
        4.0,
        "chains ending in sat keep their weight"
    );
    assert_eq!(chain(&got, &[the, cat, sat]), 4.0);
    assert_eq!(got.seen.get(1), Some(&4), "sightings are never scaled");
}

#[test]
fn lossy_counting_keeps_the_frequent_chains_under_the_cap() {
    let mut chains = Chains::new(10);
    let frequent = key(&[1, 2]);
    for i in 0..200 {
        chains.add(key(&[100 + i, 7]), 1.0);
        chains.add(frequent, 1.0);
        assert!(chains.table.len() <= 10);
    }
    assert!(chains.prunes > 0 && chains.floor > 0.0);
    assert_eq!(
        chains.table.get(&frequent),
        Some(&200.0),
        "never dropped, so never undercounted"
    );
}

#[test]
fn tatoeba_gives_its_entries_up_to_its_cap_and_refuses_a_short_line() {
    let file = tatoeba(&[
        ("1", "Hi.", "ann"),
        ("2", "Go!", "bob"),
        ("3", "Yes?", "\\N"),
    ]);
    let mut seen = Vec::new();
    let read = tsv::read(file.as_slice(), 2, &mut |e| {
        seen.push((e.id.to_string(), e.text.to_string(), e.user.to_string()));
    });
    assert_eq!(read, Ok(2));
    assert_eq!(
        seen,
        [
            ("1".into(), "Hi.".into(), "ann".into()),
            ("2".into(), "Go!".into(), "bob".into())
        ]
    );
    let short = super::super::fixture::text::bz2(b"1\txx\tHi.\n");
    assert!(tsv::read(short.as_slice(), 9, &mut |_| {}).is_err());
}

#[test]
fn the_held_back_share_is_stable_and_near_its_per_mille() {
    let ids: Vec<String> = (1..=10_000).map(|i| i.to_string()).collect();
    let held: Vec<&String> = ids.iter().filter(|id| tsv::held(id, 20)).collect();
    let again: Vec<&String> = ids.iter().filter(|id| tsv::held(id, 20)).collect();
    assert_eq!(held, again);
    assert!((150..=250).contains(&held.len()), "{} of 10000", held.len());
    assert!(ids.iter().all(|id| !tsv::held(id, 0)));
}

#[test]
fn wikipedia_gives_each_text_up_to_its_cap_and_needs_text_in_its_first_document() {
    let file = wiki(&[Some("One doc."), None, Some("Two docs.")]);
    let mut seen = Vec::new();
    assert_eq!(
        dump::read(file.as_slice(), 100, &mut |t| seen.push(t.to_string())),
        Ok(17)
    );
    assert_eq!(seen, ["One doc.", "Two docs."]);
    let mut first = Vec::new();
    assert_eq!(
        dump::read(file.as_slice(), 3, &mut |t| first.push(t.to_string())),
        Ok(8)
    );
    assert_eq!(first, ["One doc."]);
    let bad = wiki(&[None, Some("text")]);
    let err = dump::read(bad.as_slice(), 100, &mut |_| {}).unwrap_err();
    assert!(err.contains("first document has no text"), "{err}");
}
