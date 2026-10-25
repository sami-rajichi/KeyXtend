//! Loading with other settings: tables and caps the settings drop, a trimmed blocklist, renumbered ids, and counts
//! past the top.

use super::super::clean::{COUNT_TOP, peak};
use super::bytes::{C, ONE, W, encode};
use super::*;

#[test]
fn tables_and_caps_the_settings_drop_are_left_out_on_load() {
    let words: Vec<String> = (0..13).map(word).collect();
    let listed: Vec<W<'_>> = (0..13)
        .map(|n| {
            (
                u16::try_from(n).unwrap(),
                words[n].as_str(),
                1,
                DAY + u32::try_from(n).unwrap(),
            )
        })
        .collect();
    let longer: &[C<'_>] = &[(&[3, 4, 5], 1, DAY)];
    let bytes = encode("en", &listed, &[&[(&[3, 4], 1, DAY)], longer], &[]);
    let cfg = LearnConfig {
        max_words: 10,
        ..ONE
    };
    let l = Learner::from_bytes(&bytes, cfg).unwrap();
    assert_eq!((l.words.len(), l.chains.len()), (10, 1));
    let view = l.view(None);
    assert_eq!((view.count(&words[2]), view.count(&words[3])), (0, 1));
    assert_eq!(view.chain(&[&words[3]], &words[4]), 1);
}

#[test]
fn a_lowered_max_words_trims_the_blocklist_least_recently_used_first() {
    let blocked = [
        ("a", 0, DAY + 2),
        ("b", 1, DAY),
        ("c", 0, DAY),
        ("d", 0, DAY + 1),
    ];
    let cfg = LearnConfig {
        max_words: 2,
        ..CFG
    };
    let l = Learner::from_bytes(&encode("en", &[], &[], &blocked), cfg).unwrap();
    let kept: Vec<&str> = l.blocked.iter().map(|(text, _)| text).collect();
    assert_eq!(kept, ["a", "d"]);
    let none = LearnConfig {
        max_words: 0,
        ..CFG
    };
    let l = Learner::from_bytes(&encode("en", &[], &[], &blocked), none).unwrap();
    assert_eq!(l.blocked.len(), 1, "the word just forgotten always fits");
}

#[test]
fn ids_past_the_peak_are_renumbered_on_load() {
    let (a, far) = ((0, "a", 1, DAY), u16::MAX);
    let spread = encode(
        "en",
        &[a, (far, "b", 1, DAY)],
        &[&[(&[far, 0], 1, DAY)]],
        &[],
    );
    let packed = encode("en", &[a, (1, "b", 1, DAY)], &[&[(&[1, 0], 1, DAY)]], &[]);
    let back = Learner::from_bytes(&spread, ONE);
    assert_eq!(back, Learner::from_bytes(&packed, ONE));
    assert_eq!(back.unwrap().view(None).chain(&["b"], "a"), 1);
}

#[test]
fn the_word_table_refuses_an_id_past_its_peak() {
    let at = |id: usize| {
        let word = Word {
            text: "a".into(),
            count: 1,
            day: DAY,
        };
        Words::load(
            CFG.max_words,
            Casing::of("en"),
            vec![(Id::try_from(id).unwrap(), word)],
        )
    };
    assert!(at(peak(CFG.max_words)).is_some());
    assert!(at(peak(CFG.max_words) + 1).is_none());
}

#[test]
fn more_words_than_a_lowered_cap_holds_load_and_clean() {
    let words: Vec<W<'_>> = vec![(0, "a", 1, DAY), (1, "b", 1, DAY + 1), (2, "c", 1, DAY + 2)];
    let one = LearnConfig {
        max_words: 1,
        ..ONE
    };
    let l = Learner::from_bytes(&encode("en", &words, &[], &[]), one).unwrap();
    assert_eq!((l.words.len(), l.view(None).count("c")), (1, 1));
}

#[test]
fn a_chain_past_the_top_halves_every_table_on_its_next_use() {
    let (a, b) = ((0, "a", 4, DAY), (1, "b", 1, DAY));
    let bytes = encode("en", &[a, b], &[&[(&[0, 1], COUNT_TOP, DAY)]], &[]);
    let mut l = Learner::from_bytes(&bytes, ONE).unwrap();
    l.learn(None, "b", &["a"], DAY);
    let view = l.view(None);
    let half = u32::from(COUNT_TOP.div_ceil(2));
    assert_eq!((view.chain(&["a"], "b"), view.count("a")), (half + 1, 2));
}
