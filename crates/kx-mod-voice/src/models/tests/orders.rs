//! The orders: each language's default, Nemotron never first, the general order for a language without a list, a
//! language no model knows, and the picks with their fallback to the default (plan edge cases 18 and 19).

use super::*;
use kx_module_api::ports::Date;
use std::collections::BTreeMap;

/// The ids of `ranked`'s rows, in order.
fn ids<'a>(ranked: &Ranked<'a>) -> Vec<&'a str> {
    ranked.rows.iter().map(|r| r.id.as_str()).collect()
}

/// The day the orders were tested.
fn tested() -> Date {
    Date::new(2026, 11, 14).unwrap()
}

#[test]
fn the_defaults_are_parakeet_for_english_and_french_and_cohere_arabic_for_arabic() {
    let t = shipped();
    for (lang, id) in [
        ("en", "parakeet-v3-q8"),
        ("fr", "parakeet-v3-q8"),
        ("ar", "cohere-arabic-q8"),
    ] {
        let ranked = t.order(lang);
        assert_eq!(ranked.first().map(|r| r.id.as_str()), Some(id), "{lang}");
        assert_eq!(
            t.default_for(lang).map(|r| r.id.as_str()),
            Some(id),
            "{lang}"
        );
        assert!(!ranked.untested, "{lang} has its own tested list");
        assert_eq!(ranked.tested, tested(), "{lang}");
    }
}

#[test]
fn each_tested_order_lists_exactly_the_rows_that_know_its_language() {
    let t = shipped();
    for lang in ["en", "fr", "ar"] {
        let mut listed = ids(&t.order(lang));
        let mut knowing: Vec<&str> = t
            .rows()
            .iter()
            .filter(|r| r.code(lang).is_some())
            .map(|r| r.id.as_str())
            .collect();
        listed.sort_unstable();
        knowing.sort_unstable();
        assert_eq!(listed, knowing, "{lang}");
    }
    assert_eq!(
        t.order("ar").rows.len(),
        8,
        "Parakeet and Canary know no Arabic"
    );
    assert_eq!(t.order("fr").rows.len(), 9, "Cohere Arabic knows no French");
}

#[test]
fn nemotron_is_never_first_and_is_the_only_row_that_never_leads() {
    let t = shipped();
    let langs = t.langs();
    assert!(langs.len() > 100, "{}", langs.len());
    for lang in langs {
        let first = t.order(lang).first().unwrap();
        assert_ne!(first.id, "nemotron-0.6b-q8", "{lang}");
        assert!(first.lead, "{lang}");
    }
    let followers: Vec<&str> = t
        .rows()
        .iter()
        .filter(|r| !r.lead)
        .map(|r| r.id.as_str())
        .collect();
    assert_eq!(followers, ["nemotron-0.6b-q8"]);
}

#[test]
fn a_language_without_a_list_takes_the_general_order_of_the_models_that_know_it() {
    let t = shipped();
    let general = ids(&t.general());
    for lang in ["de", "es", "tr", "ja", "haw", "nb", "fil", "yue"] {
        let ranked = t.order(lang);
        assert!(ranked.untested, "{lang} is marked untested");
        assert!(!ranked.rows.is_empty(), "{lang}");
        assert!(ranked.rows.iter().all(|r| r.code(lang).is_some()), "{lang}");
        let mut at = general.iter();
        assert!(
            ids(&ranked).iter().all(|id| at.any(|g| g == id)),
            "{lang} keeps the general order"
        );
        let knowing = t.rows().iter().filter(|r| r.code(lang).is_some()).count();
        assert_eq!(ranked.rows.len(), knowing, "{lang}");
    }
    assert_eq!(
        ids(&t.order("de"))[..3],
        ["parakeet-v3-q8", "cohere-q8", "whisper-turbo-q5km"]
    );
    assert_eq!(
        ids(&t.order("haw")),
        [
            "whisper-turbo-q5km",
            "whisper-large-v3-q5km",
            "whisper-small-q5km"
        ]
    );
    assert_eq!(
        ids(&t.order("yue")),
        [
            "whisper-turbo-q5km",
            "whisper-large-v3-q5km",
            "qwen3-1.7b-q8",
            "qwen3-0.6b-q8"
        ]
    );
}

#[test]
fn a_language_no_model_knows_has_no_order_and_no_default() {
    let t = shipped();
    for lang in ["kl", "iu", ""] {
        assert!(t.order(lang).rows.is_empty(), "{lang:?}");
        assert!(t.default_for(lang).is_none(), "{lang:?}");
        assert!(t.used(lang, &BTreeMap::new()).is_none(), "{lang:?}");
    }
}

#[test]
fn a_pick_names_its_row_or_file_and_anything_else_takes_the_default() {
    let t = shipped();
    let picks = BTreeMap::from([
        ("fr".to_owned(), "canary-v2-q8".to_owned()),
        ("ar".to_owned(), "parakeet-v3-q8".to_owned()),
        ("en".to_owned(), "gone-model".to_owned()),
        ("de".to_owned(), "my-model.gguf".to_owned()),
    ]);
    let used = |lang: &str| t.used(lang, &picks);
    assert!(matches!(used("fr"), Some(Pick::Row(r)) if r.id == "canary-v2-q8"));
    assert!(
        matches!(used("ar"), Some(Pick::Row(r)) if r.id == "cohere-arabic-q8"),
        "a row that knows no Arabic"
    );
    assert!(
        matches!(used("en"), Some(Pick::Row(r)) if r.id == "parakeet-v3-q8"),
        "a row no longer in the table"
    );
    assert!(matches!(used("de"), Some(Pick::File("my-model.gguf"))));
    assert!(
        matches!(used("es"), Some(Pick::Row(r)) if r.id == "parakeet-v3-q8"),
        "no pick"
    );
    assert!(t.pick("fr", "parakeet-v3-q8").is_some() && t.pick("fr", "cohere-arabic-q8").is_none());
    for bad in [
        "",
        "x.exe",
        r"a\b.gguf",
        "a/b.gguf",
        "C:b.gguf",
        "NUL.gguf",
        "a\u{200B}.gguf",
    ] {
        assert!(t.pick("fr", bad).is_none(), "{bad:?}");
    }
}
