//! The `[predict]` settings: the agreed defaults, every range's ends, the checked lists, and the ranges tied to the
//! engine's own limits so no limit is typed twice.

use super::*;
use crate::learn::{MAX_CHAINS as LEARN_CHAINS, MAX_LEARNED, MAX_MIN_USES};
use crate::pack::layout::MAX_CHAIN;
use crate::score::{BACKOFFS, WEIGHTS};
use kx_module_api::parse_as;

/// The defaults as a table.
fn defaults() -> toml::Table {
    SPEC.defaults.parse().unwrap()
}

/// The defaults with `key` set to `value`.
fn with(key: &str, value: impl Into<toml::Value>) -> toml::Table {
    let mut t = defaults();
    t.insert(key.to_owned(), value.into());
    t
}

/// True when the table is refused by a rule that names `key`.
fn refused(t: &toml::Table, key: &str) -> bool {
    matches!(validate_as::<PredictSettings>(t), Err(SettingsError::Invalid(why)) if why.contains(key))
}

/// True when the table is taken.
fn taken(t: &toml::Table) -> bool {
    validate_as::<PredictSettings>(t).is_ok()
}

/// The ends of `range`, widened for the table.
fn ends<T: Copy + TryInto<i64>>(range: &RangeInclusive<T>) -> (i64, i64) {
    let wide = |n: T| {
        n.try_into()
            .unwrap_or_else(|_| panic!("a range end fits an i64"))
    };
    (wide(*range.start()), wide(*range.end()))
}

/// Each whole-number key and its range's ends.
fn bounds() -> Vec<(&'static str, (i64, i64))> {
    vec![
        ("chips", ends(&CHIPS)),
        ("context_words", ends(&CONTEXT_WORDS)),
        ("min_uses", ends(&MIN_USES)),
        ("max_words", ends(&MAX_WORDS)),
        ("max_chains", ends(&MAX_CHAINS)),
        ("forget_days", ends(&FORGET_DAYS)),
        ("learn_ramp", ends(&LEARN_RAMP)),
        ("max_word", ends(&MAX_WORD)),
        ("max_emails", ends(&MAX_EMAILS)),
        ("min_start", ends(&MIN_START)),
        ("read_ms", ends(&READ_MS)),
        ("read_chars", ends(&READ_CHARS)),
        ("save_s", ends(&SAVE_S)),
    ]
}

#[test]
fn the_defaults_validate_and_hold_the_agreed_values() {
    assert!(SPEC.is_consistent());
    let s = PredictSettings::from_defaults().unwrap();
    assert!(s.on && s.learn);
    assert_eq!((s.chips, s.context_words, s.min_uses), (5, 2, 2));
    assert_eq!(
        (s.max_words, s.max_chains, s.forget_days),
        (10_000, 30_000, 300)
    );
    assert_eq!((s.learn_weight, s.learn_ramp, s.backoff), (0.6, 2000, 0.4));
    assert_eq!((s.max_word, s.max_emails, s.min_start), (48, 50, 2));
    let domains = [
        "gmail.com",
        "outlook.com",
        "hotmail.com",
        "yahoo.com",
        "icloud.com",
    ];
    assert_eq!(s.domains, domains);
    assert_eq!((s.read_ms, s.read_chars, s.save_s), (150, 1024, 30));
}

#[test]
fn each_range_takes_its_ends_and_refuses_a_value_just_outside() {
    for (key, (lo, hi)) in bounds() {
        assert!(taken(&with(key, lo)), "{key} = {lo}");
        assert!(taken(&with(key, hi)), "{key} = {hi}");
        assert!(refused(&with(key, lo - 1), key), "{key} = {}", lo - 1);
        assert!(refused(&with(key, hi + 1), key), "{key} = {}", hi + 1);
    }
}

#[test]
fn the_shares_take_zero_to_one_and_refuse_past_or_nan() {
    for key in ["learn_weight", "backoff"] {
        assert!(taken(&with(key, 0.0)) && taken(&with(key, 1.0)), "{key}");
        for bad in [-0.01, 1.01, f64::NAN] {
            assert!(refused(&with(key, bad), key), "{key} = {bad}");
        }
    }
}

#[test]
fn a_bad_domain_or_too_many_are_refused_and_an_empty_list_taken() {
    let list = |items: &[&str]| {
        toml::Value::from(items.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>())
    };
    assert!(refused(
        &with("domains", list(&["gmail.com", "gmail"])),
        "domains"
    ));
    assert!(refused(&with("domains", list(&["a@b.com"])), "domains"));
    let many = vec!["x.com"; usize::from(*DOMAINS.end()) + 1];
    assert!(refused(&with("domains", list(&many)), "domains"));
    assert!(taken(&with("domains", list(&[]))));
}

#[test]
fn an_unknown_key_does_not_parse() {
    let t = with("chip", 5);
    assert!(matches!(
        validate_as::<PredictSettings>(&t),
        Err(SettingsError::Parse(_))
    ));
}

#[test]
fn the_ranges_are_tied_to_the_engines_limits() {
    assert_eq!(
        *CONTEXT_WORDS.end(),
        MAX_CHAIN - 1,
        "the pack's longest chain"
    );
    assert!(*MIN_USES.end() <= MAX_MIN_USES, "the learner's headroom");
    assert_eq!(*MAX_WORDS.end(), MAX_LEARNED);
    assert!(*MAX_CHAINS.end() <= LEARN_CHAINS);
    assert_eq!((LEARN_WEIGHT, BACKOFF), (WEIGHTS, BACKOFFS));
}

#[test]
fn every_value_in_range_fills_configs_the_engine_takes() {
    let edge = |key: &str, at: f64| {
        let s: PredictSettings = parse_as(&with(key, at)).unwrap();
        (s.score_config().check(), s.learn_config().check())
    };
    for key in ["learn_weight", "backoff"] {
        assert_eq!(edge(key, 0.0), (Ok(()), Ok(())), "{key}");
        assert_eq!(edge(key, 1.0), (Ok(()), Ok(())), "{key}");
    }
    let most: PredictSettings = parse_as(&with("min_uses", i64::from(*MIN_USES.end()))).unwrap();
    assert_eq!(most.learn_config().check(), Ok(()));
}

#[test]
fn one_min_uses_fills_both_configs_and_the_tests_share_the_defaults() {
    let s = PredictSettings::from_defaults().unwrap();
    assert_eq!(
        s.score_config(),
        crate::score::CFG,
        "the scorer's test copy"
    );
    assert_eq!(
        s.learn_config(),
        crate::learn::tests::CFG,
        "the learner's test copy"
    );
    let three: PredictSettings = parse_as(&with("min_uses", 3)).unwrap();
    assert_eq!(
        (three.score_config().min_uses, three.learn_config().min_uses),
        (3, 3)
    );
}

#[test]
fn the_platform_and_tracker_configs_come_from_their_keys() {
    let s = PredictSettings::from_defaults().unwrap();
    let field = s.field_config();
    assert_eq!((field.read_ms, field.read_chars), (s.read_ms, s.read_chars));
    let track = s.track_config();
    assert_eq!(
        (track.context_words, track.max_word),
        (usize::from(s.context_words), s.max_word)
    );
    let email = s.email_config();
    assert_eq!(
        (email.max_emails, email.min_start),
        (s.max_emails, s.min_start)
    );
    assert_eq!(email.domains, s.domains);
}

#[test]
fn the_menus_keys_are_switches_of_the_section() {
    for key in [LEARN_KEY, ON_KEY] {
        assert!(
            defaults().get(key).is_some_and(toml::Value::is_bool),
            "{key}"
        );
    }
}
