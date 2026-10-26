//! Caps and their clean-up, least recently used first; the purge after `forget_days`; halving near the count limit,
//! a merge's included.

use super::super::clean::{COUNT_TOP, HEADROOM};
use super::seed::french;
use super::*;

/// Caps of 10 words and 10 chains of each length.
const SMALL: LearnConfig = LearnConfig {
    max_words: 10,
    max_chains: 10,
    ..CFG
};

/// Learns words `from..to`, word `n` after the words before it on day `DAY + n`; gives the words up to `to`.
fn chained(l: &mut Learner, from: usize, to: usize) -> Vec<String> {
    let words: Vec<String> = (0..to).map(word).collect();
    for n in from..to {
        let before: Vec<&str> = words[n.saturating_sub(l.cfg.context_words)..n]
            .iter()
            .map(String::as_str)
            .collect();
        l.learn(None, &words[n], &before, DAY + u32::try_from(n).unwrap());
    }
    words
}

#[test]
fn the_thirteenth_word_cleans_back_to_ten_dropping_the_least_recently_used() {
    let mut l = Learner::new("en", SMALL).unwrap();
    let words = chained(&mut l, 0, 12);
    assert_eq!(l.words.len(), 12);
    assert_eq!(l.view(None).count(&words[0]), 1);
    let words = chained(&mut l, 12, 13);
    assert_eq!(l.words.len(), 10);
    let view = l.view(None);
    let counts: Vec<u32> = words.iter().map(|w| view.count(w)).collect();
    assert_eq!(counts, [0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
    assert_eq!(view.chain(&[&words[2]], &words[3]), 0);
    assert_eq!(view.chain(&[&words[2], &words[3]], &words[4]), 0);
    assert_eq!(view.chain(&[&words[3]], &words[4]), 1);
    assert_eq!(view.chain(&[&words[3], &words[4]], &words[5]), 1);
    assert_eq!(lengths(&l), [9, 8], "no chain names a dropped word");
}

#[test]
fn the_least_recently_used_goes_first_even_with_more_uses() {
    let mut l = Learner::new("en", SMALL).unwrap();
    let words: Vec<String> = (0..13).map(word).collect();
    for _ in 0..3 {
        l.learn(None, &words[0], &[], DAY);
    }
    for w in words.iter().skip(1) {
        l.learn(None, w, &[], DAY + 1);
    }
    let view = l.view(None);
    let counts: Vec<u32> = words.iter().map(|w| view.count(w)).collect();
    assert_eq!(counts, [0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
}

#[test]
fn among_words_last_used_the_same_day_the_lower_count_goes_first() {
    let mut l = Learner::new("en", SMALL).unwrap();
    let words: Vec<String> = (0..13).map(word).collect();
    for (n, w) in words.iter().take(12).enumerate() {
        let uses = if n >= 9 { 1 } else { 2 };
        for _ in 0..uses {
            l.learn(None, w, &[], DAY);
        }
    }
    l.learn(None, &words[12], &[], DAY + 1);
    let view = l.view(None);
    let counts: Vec<u32> = words.iter().map(|w| view.count(w)).collect();
    assert_eq!(counts, [2, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 0, 1]);
}

#[test]
fn a_chain_table_past_its_peak_cleans_back_to_its_cap() {
    let cfg = LearnConfig {
        max_chains: 10,
        context_words: 1,
        ..CFG
    };
    let mut l = Learner::new("en", cfg).unwrap();
    chained(&mut l, 0, 13);
    assert_eq!(l.chains[0].len(), 12);
    let words = chained(&mut l, 13, 14);
    assert_eq!(l.chains[0].len(), 10);
    assert_eq!(l.words.len(), 14);
    let view = l.view(None);
    let kept: Vec<u32> = words
        .windows(2)
        .map(|p| view.chain(&[&p[0]], &p[1]))
        .collect();
    assert_eq!(kept, [0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
    assert!(words.iter().all(|w| view.count(w) > 0), "{words:?}");
}

#[test]
fn purge_drops_entries_unused_for_more_than_forget_days() {
    let mut l = learner("en");
    teach(&mut l, None, "old words", DAY);
    l.learn(None, "new", &["old"], DAY + 1);
    l.learn(None, "words", &[], DAY + 1);
    l.purge(DAY + CFG.forget_days);
    assert_eq!(l.view(None).count("old"), 1);
    assert_eq!(l.view(None).chain(&["old"], "words"), 1);
    l.purge(DAY + CFG.forget_days + 1);
    let view = l.view(None);
    assert_eq!((view.count("old"), view.count("words")), (0, 2));
    assert_eq!(view.chain(&["old"], "words"), 0);
    assert_eq!(
        view.chain(&["old"], "new"),
        0,
        "a chain naming a dropped word goes"
    );
    assert_eq!(view.count("new"), 1);
    assert_eq!(view.uses(), 3);
    assert_eq!(lengths(&l), [0, 0], "no chain names a dropped word");
}

#[test]
fn counts_near_the_limit_halve_and_keep_their_order() {
    let mut l = learner("en");
    teach(&mut l, None, "c b b a a a", DAY);
    for _ in 0..COUNT_TOP {
        l.learn(None, "top", &["a"], DAY);
    }
    let counts = |l: &Learner| ["top", "a", "b", "c"].map(|w| l.view(None).count(w));
    let top = u32::from(COUNT_TOP);
    assert_eq!(counts(&l), [top, 3, 2, 1]);
    l.learn(None, "top", &["a"], DAY);
    let half = u32::from(COUNT_TOP.div_ceil(2));
    assert_eq!(
        counts(&l),
        [half + 1, 2, 2, 1],
        "no count falls below min_uses"
    );
    let view = l.view(None);
    assert_eq!(view.chain(&["a"], "top"), half + 1);
    assert_eq!(view.chain(&["a"], "a"), 2);
    assert_eq!(view.chain(&["b", "a"], "a"), 1);
    assert_eq!(view.uses(), u64::from(half + 1 + 2 + 2 + 1));
}

/// Learns `top` until its count passes the top once, so every table halves.
fn halve(l: &mut Learner) {
    for _ in 0..=COUNT_TOP {
        l.learn(None, "top", &[], DAY);
    }
}

#[test]
fn a_word_at_min_uses_stays_offered_after_a_halving() {
    let mut l = learner("en");
    teach(&mut l, None, "zork zork odd", DAY);
    halve(&mut l);
    let view = l.view(None);
    assert_eq!((view.count("zork"), view.count("odd")), (CFG.min_uses, 1));
    assert_eq!(guessed(&l, None, &[], "zo"), ["zork"]);
}

#[test]
fn a_halving_halves_every_table_so_chain_shares_survive() {
    let mut l = learner("en");
    for _ in 0..4 {
        teach(&mut l, None, "my big park", DAY);
        l.learn(None, "my", &[], DAY);
    }
    let shares = |l: &Learner| {
        let view = l.view(None);
        let chains = (
            view.chain(&["my"], "big"),
            view.chain(&["my", "big"], "park"),
        );
        (view.count("my"), chains)
    };
    assert_eq!(shares(&l), (8, (4, 4)));
    halve(&mut l);
    assert_eq!(shares(&l), (4, (2, 2)));
}

#[test]
fn a_merge_halves_every_table_before_a_word_count_would_pass_the_top() {
    let bytes = french();
    let fr = Pack::parse(&bytes).unwrap();
    let mut l = learner("fr");
    let many = COUNT_TOP / 2 + HEADROOM;
    for _ in 0..many {
        l.learn(None, "école", &[], DAY);
        l.learn(None, "ecole", &[], DAY);
    }
    l.reconcile(Some(&fr));
    let half = u32::from(many.div_ceil(2));
    assert_eq!(l.view(Some(&fr)).count("école"), 2 * half);
}

#[test]
fn a_merge_halves_every_table_before_a_met_chain_would_pass_the_top() {
    let words = [
        (0, "l'", 10, DAY),
        (1, "école", 10, DAY),
        (2, "ecole", 10, DAY),
    ];
    let high = COUNT_TOP - HEADROOM;
    let chains: &[super::bytes::C<'_>] = &[(&[0, 1], high, DAY), (&[0, 2], high, DAY)];
    let stored = super::bytes::encode("fr", &words, &[chains], &[]);
    let mut l = Learner::from_bytes(&stored, CFG).unwrap();
    let bytes = french();
    let fr = Pack::parse(&bytes).unwrap();
    l.reconcile(Some(&fr));
    let half = u32::from(high.div_ceil(2));
    assert_eq!(l.view(Some(&fr)).chain(&["l'"], "école"), 2 * half);
}
