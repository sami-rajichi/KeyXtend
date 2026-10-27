//! The settings: a learned weight or a backoff outside 0 to 1 is refused, as the early stop cannot bound it.

use super::*;

#[test]
fn a_weight_or_a_backoff_outside_0_to_1_is_refused() {
    for learn_weight in [-0.1, 1.1, f64::NAN] {
        let cfg = ScoreConfig {
            learn_weight,
            ..CFG
        };
        assert_eq!(
            cfg.check(),
            Err(ScoreConfigError::BadWeight),
            "{learn_weight}"
        );
    }
    for backoff in [-0.1, 1.5, 1e78, f64::NAN, f64::INFINITY] {
        let cfg = ScoreConfig { backoff, ..CFG };
        assert_eq!(cfg.check(), Err(ScoreConfigError::BadBackoff), "{backoff}");
    }
    for (backoff, learn_weight) in [(0.0, 0.0), (0.4, 0.6), (1.0, 1.0)] {
        let cfg = ScoreConfig {
            backoff,
            learn_weight,
            ..CFG
        };
        assert_eq!(cfg.check(), Ok(()), "{backoff}");
    }
    assert_eq!(CFG.check(), Ok(()));
}

#[test]
fn a_refused_setting_gives_no_guess() {
    let bytes = english();
    let pack = Pack::parse(&bytes).unwrap();
    let cfg = ScoreConfig {
        learn_weight: 2.0,
        ..CFG
    };
    let found = guesses(
        "en",
        Some(&pack),
        &NoLearned,
        &ask(&[], "th"),
        TypingMode::default(),
        LIMIT,
        &cfg,
    );
    assert!(found.is_empty());
}

#[test]
fn each_refusal_names_the_ends_of_its_range() {
    for (error, range) in [
        (ScoreConfigError::BadWeight, WEIGHTS),
        (ScoreConfigError::BadBackoff, BACKOFFS),
    ] {
        let ends = format!("{} to {}", range.start(), range.end());
        assert!(error.to_string().ends_with(&ends), "{error}");
    }
}
