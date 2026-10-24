//! The learned part: the two-use gate, forgotten seed words, the growing weight and learned chains.

use super::*;

#[test]
fn a_learned_word_not_in_the_seed_appears_only_from_its_second_use() {
    let en = english();
    let once = Fake::default().uses(10).word("thx", 1);
    assert!(!run(&en, &once, &ask(&[], "th"), PLAIN).contains(&"thx".to_owned()));
    let twice = Fake::default().uses(10).word("thx", 2);
    assert!(run(&en, &twice, &ask(&[], "th"), PLAIN).contains(&"thx".to_owned()));
}

#[test]
fn a_blocked_seed_word_never_appears() {
    let en = english();
    let blocked = Fake::default().uses(10).word("the", 9).block("the");
    let typed = run(&en, &blocked, &ask(&[], "th"), PLAIN);
    let next = run(&en, &blocked, &ask(&["in"], ""), PLAIN);
    assert!(!typed.contains(&"the".to_owned()) && !next.contains(&"the".to_owned()));
    assert_eq!(typed.first().map(String::as_str), Some("they"));
}

#[test]
fn the_learned_weight_grows_with_uses() {
    assert!(CFG.weight(0).abs() < f64::EPSILON);
    assert!((CFG.weight(2000) - 0.3).abs() < 1e-9);
    let steps = [0, 10, 100, 2000, 20_000, u64::MAX];
    let weights: Vec<f64> = steps.iter().map(|&u| CFG.weight(u)).collect();
    assert!(weights.windows(2).all(|w| w[0] < w[1]), "{weights:?}");
    assert!(weights.iter().all(|&w| w <= CFG.learn_weight));
    let no_ramp = ScoreConfig {
        learn_ramp: 0,
        ..CFG
    };
    assert!(no_ramp.weight(0).abs() < f64::EPSILON);
    let en = english();
    let early = Fake::default().uses(100).word("thx", 50);
    let late = Fake::default().uses(10_000).word("thx", 5_000);
    assert_eq!(
        run(&en, &early, &ask(&[], "th"), PLAIN)
            .first()
            .map(String::as_str),
        Some("the")
    );
    assert_eq!(
        run(&en, &late, &ask(&[], "th"), PLAIN)
            .first()
            .map(String::as_str),
        Some("thx")
    );
}

#[test]
fn learned_chains_guess_the_next_word() {
    let en = english();
    let learned = Fake::default()
        .uses(100)
        .word("in", 20)
        .word("Sami", 10)
        .chain(&["in"], "Sami", 10);
    let next = run(&en, &learned, &ask(&["in"], ""), PLAIN);
    assert!(next.contains(&"Sami".to_owned()), "{next:?}");
}

#[test]
fn without_a_pack_the_learned_words_still_guess() {
    let learned = Fake::default()
        .uses(10)
        .word("sami", 3)
        .word("salut", 2)
        .word("sa", 1);
    let found = texts(&guesses(
        None,
        &learned,
        &ask(&[], "sa"),
        PLAIN,
        LIMIT,
        &CFG,
    ));
    assert_eq!(found, ["sami", "salut"]);
}

#[test]
fn learned_uses_lift_a_seed_word() {
    let en = english();
    let plain = run(&en, &NoLearned, &ask(&[], "th"), PLAIN);
    assert_eq!(plain.last().map(String::as_str), Some("theory"));
    let learned = Fake::default().uses(100).word("theory", 50);
    let lifted = run(&en, &learned, &ask(&[], "th"), PLAIN);
    assert_eq!(
        lifted.get(1).map(String::as_str),
        Some("theory"),
        "{lifted:?}"
    );
}
