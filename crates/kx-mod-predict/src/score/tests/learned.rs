//! The learned part: the two-use gate, forgotten seed words, the growing weight and learned chains.

use super::*;

#[test]
fn a_learned_word_not_in_the_seed_appears_only_from_its_second_use() {
    let en = english();
    let once = Fake::default().uses(10).word("thx", 1);
    assert!(!run(&en, &once, &ask(&[], "th"), TypingMode::default()).contains(&"thx".to_owned()));
    let twice = Fake::default().uses(10).word("thx", 2);
    assert!(run(&en, &twice, &ask(&[], "th"), TypingMode::default()).contains(&"thx".to_owned()));
}

#[test]
fn a_blocked_seed_word_never_appears() {
    let en = english();
    let blocked = Fake::default().uses(10).word("the", 9).block("the");
    let typed = run(&en, &blocked, &ask(&[], "th"), TypingMode::default());
    let next = run(&en, &blocked, &ask(&["in"], ""), TypingMode::default());
    assert!(!typed.contains(&"the".to_owned()) && !next.contains(&"the".to_owned()));
    assert_eq!(typed.first().map(String::as_str), Some("they"));
}

#[test]
fn the_learned_weight_grows_with_uses() {
    assert!(CFG.weight(0).abs() < f64::EPSILON);
    let half = CFG.weight(CFG.learn_ramp.into());
    assert!(
        (half - CFG.learn_weight / 2.0).abs() < 1e-9,
        "half its most at the ramp"
    );
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
        run(&en, &early, &ask(&[], "th"), TypingMode::default())
            .first()
            .map(String::as_str),
        Some("the")
    );
    assert_eq!(
        run(&en, &late, &ask(&[], "th"), TypingMode::default())
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
    let next = run(&en, &learned, &ask(&["in"], ""), TypingMode::default());
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
        "en",
        None,
        &learned,
        &ask(&[], "sa"),
        TypingMode::default(),
        LIMIT,
        &CFG,
    ));
    assert_eq!(found, ["sami", "salut"]);
}

#[test]
fn learned_uses_lift_a_seed_word() {
    let en = english();
    let plain = run(&en, &NoLearned, &ask(&[], "th"), TypingMode::default());
    assert_eq!(plain.last().map(String::as_str), Some("theory"));
    let learned = Fake::default().uses(100).word("theory", 50);
    let lifted = run(&en, &learned, &ask(&[], "th"), TypingMode::default());
    assert_eq!(
        lifted.get(1).map(String::as_str),
        Some("theory"),
        "{lifted:?}"
    );
}

#[test]
fn after_an_elision_a_seed_follower_keeps_its_learned_part() {
    let fr = french();
    let learned = Fake::default()
        .uses(10_000)
        .word("école", 10_000)
        .word("l'", 4)
        .word("écoute", 1)
        .chain(&["l'"], "écoute", 1);
    let reading = Reading {
        context: Redacted::new(vec!["l'".to_owned()]),
        lead: Redacted::new("l'".to_owned()),
        prefix: Redacted::new("é".to_owned()),
    };
    let query = Query {
        word: Redacted::new("l'é".to_owned()),
        readings: vec![reading],
    };
    let found = run(&fr, &learned, &query, TypingMode::default());
    assert_eq!(
        found.get(..2),
        Some(&["l'école".to_owned(), "l'écoute".to_owned()][..])
    );
}
