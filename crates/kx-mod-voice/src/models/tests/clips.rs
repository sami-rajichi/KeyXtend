//! The card check's clips (ADR-0044 decision 7): the shipped clips' languages, each an ISO code once, and a model takes
//! the first one it knows.

use super::*;

/// The compiled-in text.
const TEXT: &str = include_str!("../../../models.toml");

#[test]
fn the_shipped_clips_are_english_french_and_arabic_in_that_order() {
    assert_eq!(shipped().clips(), ["en", "fr", "ar"]);
}

#[test]
fn a_model_takes_the_first_clip_whose_language_it_knows() {
    assert_eq!(shipped().clip_for(row("parakeet-v3-q8")), Some("en"));
    let mut arabic = row("cohere-arabic-q8").clone();
    arabic.langs.retain(|lang, _| lang == "ar");
    assert_eq!(shipped().clip_for(&arabic), Some("ar"));
}

#[test]
fn a_model_that_knows_no_clips_language_takes_none() {
    let mut german = row("parakeet-v3-q8").clone();
    german.langs.retain(|lang, _| lang == "de");
    assert_eq!(shipped().clip_for(&german), None);
}

#[test]
fn a_clip_that_is_no_iso_code_or_listed_twice_refuses_the_table() {
    let clips = "clips = [\"en\", \"fr\", \"ar\"]";
    for bad in [
        "clips = [\"en\", \"fr\", \"english\"]",
        "clips = [\"en\", \"en\"]",
        "clips = []",
    ] {
        let text = TEXT.replacen(clips, bad, 1);
        assert_eq!(Table::parse(&text).err(), Some(TableError::Clips), "{bad}");
    }
}
