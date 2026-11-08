//! The `[selection]` settings: the agreed defaults, every range's ends, the look and target rules, the site and its
//! limit, and the configs the parts take.

use super::*;
use crate::link::TAG_CHARS;
use std::time::Duration;

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
    matches!(validate_as::<SelSettings>(t), Err(SettingsError::Invalid(why)) if why.contains(key))
}

/// True when the table is taken.
fn taken(t: &toml::Table) -> bool {
    validate_as::<SelSettings>(t).is_ok()
}

/// The ends of `range`, widened for the table.
fn ends<T: Copy + Into<i64>>(range: &RangeInclusive<T>) -> (i64, i64) {
    ((*range.start()).into(), (*range.end()).into())
}

#[test]
fn the_defaults_validate_and_hold_the_agreed_values() {
    assert!(SPEC.is_consistent());
    let s = SelSettings::from_defaults().unwrap();
    assert!(s.on && s.copy && s.cut && s.paste && s.translate && s.caret_paste);
    assert!(s.icons && s.words && s.minimised && !s.translate_ok);
    assert_eq!(s.caret_s, 5);
    assert_eq!(
        (s.translate_to.as_str(), s.translate_site.as_str()),
        ("", "google")
    );
    assert_eq!(s.translate_chars, 1000);
    assert_eq!(
        (s.settle_ms, s.read_ms, s.looks_per_s, s.consent_s),
        (120, 250, 10, 30)
    );
}

#[test]
fn each_range_takes_its_ends_and_refuses_past_them() {
    let bounds = [
        ("caret_s", ends(&CARET_S)),
        ("translate_chars", ends(&TRANSLATE_CHARS)),
        ("settle_ms", ends(&SETTLE_MS)),
        ("read_ms", ends(&READ_MS)),
        ("looks_per_s", ends(&LOOKS_PER_S)),
        ("consent_s", ends(&CONSENT_S)),
    ];
    for (key, (lo, hi)) in bounds {
        assert!(taken(&with(key, lo)) && taken(&with(key, hi)), "{key}");
        assert!(refused(&with(key, lo - 1), key), "{key} = {}", lo - 1);
        assert!(refused(&with(key, hi + 1), key), "{key} = {}", hi + 1);
    }
}

#[test]
fn the_plans_ranges_hold() {
    assert_eq!((CARET_S, CONSENT_S), (2..=30, 5..=120));
    assert_eq!(TRANSLATE_CHARS, 50..=5000);
    assert_eq!(
        (SETTLE_MS, READ_MS, LOOKS_PER_S),
        (30..=1000, 50..=2000, 1..=30)
    );
}

#[test]
fn icons_and_words_are_never_both_off() {
    let mut both = with("icons", false);
    both.insert("words".to_owned(), false.into());
    assert!(refused(&both, "icons") && refused(&both, "words"));
    assert!(taken(&with("icons", false)) && taken(&with("words", false)));
}

#[test]
fn translate_to_is_empty_or_a_short_language_tag() {
    for good in ["", "fr", "ar", "zh-TW", "pt-BR"] {
        assert!(taken(&with("translate_to", good)), "{good}");
    }
    let long = "a".repeat(TAG_CHARS + 1);
    for bad in [
        "fr_FR",
        "fr1",
        "f r",
        "-",
        "fr-",
        "é",
        "ar\u{200f}",
        long.as_str(),
    ] {
        assert!(refused(&with("translate_to", bad), "translate_to"), "{bad}");
    }
}

#[test]
fn translate_site_names_a_site_and_the_default_is_the_first() {
    assert!(taken(&with("translate_site", "google")));
    for bad in ["bing", "", "Google"] {
        assert!(
            refused(&with("translate_site", bad), "translate_site"),
            "{bad}"
        );
    }
    let first = sites().unwrap().first().unwrap();
    assert_eq!(
        SelSettings::from_defaults().unwrap().translate_site,
        first.id
    );
}

#[test]
fn translate_chars_stays_within_the_sites_limit() {
    let table = Sites::parse(
        "[[sites]]\nid = \"google\"\nname = \"Small\"\n\
         pattern = \"https://small.example/?tl={to}&text={text}\"\n\
         limit = 1000\nwhole = []\nfallback = \"en\"\nlink_max = 4000\n",
    )
    .unwrap();
    let mut s = SelSettings::from_defaults().unwrap();
    assert_eq!(s.check_site(&table), Ok(()));
    s.translate_chars = 1001;
    assert!(
        matches!(s.check_site(&table), Err(SettingsError::Invalid(why)) if why.contains("translate_chars"))
    );
    s.translate_site = "other".to_owned();
    assert!(
        matches!(s.check_site(&table), Err(SettingsError::Invalid(why)) if why.contains("translate_site"))
    );
}

#[test]
fn an_unknown_key_is_refused() {
    let t = with("show", true);
    assert!(matches!(
        validate_as::<SelSettings>(&t),
        Err(SettingsError::Parse(_))
    ));
}

#[test]
fn the_parts_get_the_settings_in_their_units() {
    let s = SelSettings::from_defaults().unwrap();
    let watch = s.sel_config();
    assert_eq!(
        (watch.settle_ms, watch.read_ms, watch.looks_per_s),
        (120, 250, 10)
    );
    let rules = s.rules(true);
    assert!(
        rules.on && rules.copy && rules.cut && rules.paste && rules.translate && rules.caret_paste
    );
    assert!(
        !s.rules(false).translate,
        "a bad site table turns Translate off"
    );
    let strip = s.strip_config(true);
    assert_eq!((strip.rules, strip.minimised), (rules, true));
    assert_eq!(strip.caret, Duration::from_secs(5));
    assert!(!s.strip_config(false).rules.translate);
    assert_eq!(
        (s.caret(), s.consent()),
        (Duration::from_secs(5), Duration::from_secs(30))
    );
}

#[test]
fn each_switch_reaches_its_rule() {
    let base = SelSettings::from_defaults().unwrap();
    let off = |edit: fn(&mut SelSettings)| {
        let mut s = base.clone();
        edit(&mut s);
        s.rules(true)
    };
    assert!(!off(|s| s.on = false).on);
    assert!(!off(|s| s.copy = false).copy);
    assert!(!off(|s| s.cut = false).cut);
    assert!(!off(|s| s.paste = false).paste);
    assert!(!off(|s| s.translate = false).translate);
    assert!(!off(|s| s.caret_paste = false).caret_paste);
    let mut s = base;
    s.minimised = false;
    assert!(!s.strip_config(true).minimised);
}
