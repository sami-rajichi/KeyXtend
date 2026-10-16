use super::*;
use crate::sounds::fixture::{BOOK, FORMAT};

/// The fixture book with `from` replaced by `to`, which must occur in it.
fn edited(from: &str, to: &str) -> String {
    assert!(BOOK.contains(from), "the fixture lacks {from:?}");
    BOOK.replacen(from, to, 1)
}

/// Asserts the edited book is refused with a message that names `word`.
fn refused(from: &str, to: &str, word: &str) {
    let err = parse(&edited(from, to), FORMAT).unwrap_err();
    assert!(matches!(err, SoundError::Recipe(_)), "got: {err}");
    assert!(err.to_string().contains(word), "{err} lacks {word:?}");
}

#[test]
fn the_fixture_parses() {
    let book = parse(BOOK, FORMAT).unwrap();
    assert_eq!(book.format.rate, 48_000);
    assert_eq!(book.recipes.len(), 2);
    assert_eq!(book.recipes[0].variants, Some(3));
    assert!(matches!(book.recipes[1].layers[0], Layer::Noise(_)));
}

#[test]
fn unknown_fields_and_kinds_are_refused() {
    refused("level = 0.5", "level = 0.5\ncolour = 1", "colour");
    refused("kind = \"noise\"", "kind = \"chime\"", "chime");
    refused("filter = \"bandpass\"", "filter = \"notch\"", "notch");
}

#[test]
fn a_recipe_file_that_still_holds_the_format_is_refused() {
    let format = &FORMAT[..FORMAT.find("[play]").unwrap()];
    let err = parse(&format!("{format}{BOOK}"), FORMAT).unwrap_err();
    assert!(matches!(err, SoundError::Recipe(_)), "got: {err}");
    assert!(err.to_string().contains("[format]"), "{err}");
}

#[test]
fn a_bad_format_file_is_refused_with_its_reason() {
    let err = parse(BOOK, &FORMAT.replacen("bits = 16", "bits = 24", 1)).unwrap_err();
    assert!(matches!(err, SoundError::Format(_)), "got: {err}");
    assert!(err.to_string().contains("bits"), "{err}");
    let err = parse(BOOK, &FORMAT.replacen("[drop]", "[dorp]", 1)).unwrap_err();
    assert!(err.to_string().contains("drop"), "{err}");
}

#[test]
fn a_format_the_writer_cannot_make_is_refused() {
    // The sounds file already refuses these, so they are set past its parse.
    let refused = |book: &Book, word: &str| {
        let err = book.validate().unwrap_err().to_string();
        assert!(err.contains(word), "{err} lacks {word:?}");
    };
    let mut book = parse(BOOK, FORMAT).unwrap();
    book.format.bits = 24;
    refused(&book, "bits");
    let mut book = parse(BOOK, FORMAT).unwrap();
    book.format.channels = 2;
    refused(&book, "channels");
}

#[test]
fn names_must_be_unique_and_plain() {
    refused("name = \"thud\"", "name = \"tap\"", "tap");
    refused("name = \"thud\"", "name = \"../thud\"", "thud");
    refused("name = \"thud\"", "name = \"Thud\"", "Thud");
}

#[test]
fn lengths_gains_and_variants_stay_in_range() {
    refused("length_ms = 100", "length_ms = 401", "length_ms");
    refused("length_ms = 100", "length_ms = 0", "length_ms");
    refused("gain_db = -2", "gain_db = 1", "gain_db");
    refused("variants = 3", "variants = 0", "variants");
}

#[test]
fn layers_start_inside_the_sound_with_sane_values() {
    refused("start_ms = 2", "start_ms = 100", "start_ms");
    refused("attack_ms = 1\n", "attack_ms = -1\n", "attack_ms");
    refused("decay_ms = 40", "decay_ms = 0", "decay_ms");
    refused("q = 0.7", "q = 0", "q");
    refused("level = 1.0\nglide", "level = nan\nglide", "level");
    refused("ratio = 0.8", "ratio = 0", "glide");
}

#[test]
fn a_recipe_needs_layers_and_modes_need_partials() {
    let cut = BOOK.rfind("[[recipe.layer]]").unwrap();
    let err = parse(&BOOK[..cut], FORMAT).unwrap_err();
    assert!(err.to_string().contains("layer"), "got: {err}");
    let start = BOOK.find("partials = [").unwrap();
    let end = BOOK[start..].find(']').unwrap() + start;
    let empty = format!("{}partials = []{}", &BOOK[..start], &BOOK[end + 1..]);
    let err = parse(&empty, FORMAT).unwrap_err();
    assert!(err.to_string().contains("partials"), "got: {err}");
}

#[test]
fn frequencies_stay_below_nyquist_after_jitter_and_glide() {
    refused("hz = 3000", "hz = 23900", "hz");
    refused("hz = 1300", "hz = 20000", "hz");
    let fits = parse(&edited("hz = 3000", "hz = 19000"), FORMAT).unwrap();
    assert_eq!(fits.recipes.len(), 2);
}

#[test]
fn a_recipe_writes_its_name_or_one_stem_per_variant() {
    let book = parse(BOOK, FORMAT).unwrap();
    assert_eq!(book.recipes[0].stems(), ["tap-1", "tap-2", "tap-3"]);
    assert_eq!(book.recipes[1].stems(), ["thud"]);
}

#[test]
fn two_recipes_may_not_write_the_same_file() {
    refused("name = \"thud\"", "name = \"tap-2\"", "\"tap-2\"");
}

#[test]
fn a_recipe_whose_layers_are_all_silent_is_refused() {
    let silent = "level = 0.0\nglide = { ratio = 0.8";
    refused("level = 1.0\nglide = { ratio = 0.8", silent, "thud");
    let quiet_partials = edited("level = 1.0 },", "level = 0.0 },")
        .replacen("level = 0.5 },", "level = 0.0 },", 1)
        .replacen("level = 0.5\n", "level = 0.0\n", 1);
    let err = parse(&quiet_partials, FORMAT).unwrap_err();
    assert!(err.to_string().contains("tap"), "got: {err}");
}
