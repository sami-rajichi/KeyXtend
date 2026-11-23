//! The clip a card's check reads (ADR-0052): the language in front's when the model knows it, else the first clip the
//! model knows in the shipped order; the model's own code for that clip's language comes with it.

use super::*;

/// The shipped clips, in their order.
fn shipped() -> Vec<String> {
    ["en", "fr", "ar"].map(String::from).to_vec()
}

/// A model's own codes, by ISO code.
const CODES: [(&str, &str); 5] = [
    ("en", "english"),
    ("fr", "french"),
    ("ar", "arabic"),
    ("de", "german"),
    ("it", "italian"),
];

/// The clip and code a model knowing `knows` gets with `front` in front.
fn pick(front: &str, knows: &[&str]) -> Option<(String, &'static str)> {
    let code = |iso: &str| {
        let spoken = knows.contains(&iso);
        let row = CODES.iter().find(|(i, _)| spoken && *i == iso);
        row.map(|(_, c)| *c)
    };
    clip(front, &shipped(), code).map(|(c, m)| (c.to_owned(), m))
}

/// The clip of `iso`, with the model's code for it.
fn got(iso: &str) -> Option<(String, &'static str)> {
    let code = CODES.iter().find(|(i, _)| *i == iso).map(|(_, c)| *c);
    code.map(|c| (iso.to_owned(), c))
}

#[test]
fn arabic_in_front_checks_a_model_knowing_arabic_and_english_on_the_arabic_clip() {
    assert_eq!(pick("ar", &["ar", "en"]), got("ar"), "not English's first");
}

#[test]
fn french_in_front_checks_a_model_knowing_french_on_the_french_clip() {
    assert_eq!(pick("fr", &["en", "fr", "de"]), got("fr"));
}

#[test]
fn a_model_lacking_the_front_language_takes_its_first_known_clip() {
    assert_eq!(pick("ar", &["fr", "en"]), got("en"), "the shipped order");
    assert_eq!(pick("de", &["de", "fr"]), got("fr"), "German ships no clip");
}

#[test]
fn a_model_of_one_language_checks_on_its_clip_whatever_is_in_front() {
    for front in ["en", "fr", "ar", "de", ""] {
        assert_eq!(pick(front, &["fr"]), got("fr"), "{front:?}");
    }
}

#[test]
fn a_model_knowing_no_shipped_clip_gets_none() {
    assert_eq!(pick("de", &["de", "it"]), None);
    assert_eq!(pick("ar", &[]), None);
}
