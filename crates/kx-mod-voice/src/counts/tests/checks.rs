//! A card's check counts for the language of its clip alone (ADR-0052): one in English lets no other language use the
//! card, and a result kept before checks named their language counts for none.

use super::*;

/// The card's check of file 1 under driver `1.0`, on the clip of `lang`.
fn on(lang: Option<&str>, verdict: Verdict) -> Checked {
    Checked {
        lang: lang.map(str::to_owned),
        ..checked("Vulkan0", "1.0", 1, verdict)
    }
}

/// Its verdict in `lang`.
fn seen(checks: &Checks, lang: &str) -> Option<Verdict> {
    checks.verdict(&card("Vulkan0"), "1.0", &print(1), lang)
}

const DIFFERS: Verdict = Verdict::Differs {
    words: 3,
    allowed: 1,
};

#[test]
fn a_result_in_english_serves_english_alone() {
    let checks = Checks::new(vec![on(Some("en"), passed(900))]);
    assert_eq!(seen(&checks, "en"), Some(passed(900)));
    assert_eq!(seen(&checks, "ar"), None, "Arabic needs its own check");
    assert_eq!(seen(&checks, "fr"), None);
}

#[test]
fn a_result_in_arabic_serves_arabic() {
    let checks = Checks::new(vec![on(Some("ar"), passed(900))]);
    assert_eq!(seen(&checks, "ar"), Some(passed(900)));
    assert_eq!(seen(&checks, "en"), None);
}

#[test]
fn an_old_result_without_a_language_serves_none_but_is_kept() {
    let checks = Checks::new(vec![on(None, passed(900))]);
    for lang in ["en", "fr", "ar"] {
        assert_eq!(seen(&checks, lang), None, "{lang}");
    }
    assert_eq!(
        checks.rows().len(),
        1,
        "Automatic still knows the card was checked"
    );
}

#[test]
fn each_language_keeps_its_own_result_and_a_new_one_replaces_its_own_alone() {
    let mut checks = Checks::default();
    checks.put(on(Some("en"), passed(900)));
    checks.put(on(Some("ar"), DIFFERS));
    assert_eq!(
        (seen(&checks, "en"), seen(&checks, "ar")),
        (Some(passed(900)), Some(DIFFERS))
    );
    checks.put(on(Some("ar"), passed(400)));
    assert_eq!(
        (seen(&checks, "en"), seen(&checks, "ar")),
        (Some(passed(900)), Some(passed(400)))
    );
    assert_eq!(checks.rows().len(), 2);
}

#[test]
fn a_new_result_in_any_language_replaces_that_files_old_one() {
    let other = Checked {
        model: print(2),
        ..on(None, passed(300))
    };
    let mut checks = Checks::new(vec![on(None, passed(900)), other.clone()]);
    checks.put(on(Some("ar"), DIFFERS));
    assert_eq!(
        checks.rows(),
        [other, on(Some("ar"), DIFFERS)],
        "another file's stays"
    );
}

#[test]
fn a_newer_driver_checks_every_language_again() {
    let checks = Checks::new(vec![
        on(Some("en"), passed(900)),
        on(Some("ar"), passed(800)),
    ]);
    for lang in ["en", "ar"] {
        let found = checks.verdict(&card("Vulkan0"), "2.0", &print(1), lang);
        assert_eq!(found, None, "{lang}");
    }
}

#[test]
fn a_card_that_failed_mid_run_loses_that_files_result_in_every_language() {
    let mut checks = Checks::new(vec![
        on(Some("en"), passed(900)),
        on(Some("ar"), passed(800)),
    ]);
    assert!(checks.fell(&card("Vulkan0"), &print(1)));
    assert!(checks.rows().is_empty());
}

#[test]
fn a_result_whose_language_is_no_iso_code_is_never_kept() {
    let mut checks = Checks::default();
    for odd in ["", "EN", "fr-FR"] {
        checks.put(on(Some(odd), passed(900)));
    }
    assert!(
        checks.rows().is_empty(),
        "the store reads back only ISO codes"
    );
}
