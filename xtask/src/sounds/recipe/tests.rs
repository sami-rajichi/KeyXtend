use super::*;
use crate::sounds::fixture::BOOK;

/// The fixture book with `from` replaced by `to`, which must occur in it.
fn edited(from: &str, to: &str) -> String {
    assert!(BOOK.contains(from), "the fixture lacks {from:?}");
    BOOK.replacen(from, to, 1)
}

/// Asserts the edited book is refused with a message that names `word`.
fn refused(from: &str, to: &str, word: &str) {
    let err = parse(&edited(from, to)).unwrap_err();
    assert!(matches!(err, SoundError::Recipe(_)), "got: {err}");
    assert!(err.to_string().contains(word), "{err} lacks {word:?}");
}

#[test]
fn the_fixture_parses() {
    let book = parse(BOOK).unwrap();
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
fn a_format_the_writer_cannot_make_is_refused() {
    refused("bits = 16", "bits = 24", "bits");
    refused("channels = 1", "channels = 2", "channels");
    refused("ceiling_db = -3.0", "ceiling_db = 1.0", "ceiling_db");
    refused("fade_ms = 5", "fade_ms = 500", "fade_ms");
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
    let err = parse(&BOOK[..cut]).unwrap_err();
    assert!(err.to_string().contains("layer"), "got: {err}");
    let start = BOOK.find("partials = [").unwrap();
    let end = BOOK[start..].find(']').unwrap() + start;
    let empty = format!("{}partials = []{}", &BOOK[..start], &BOOK[end + 1..]);
    let err = parse(&empty).unwrap_err();
    assert!(err.to_string().contains("partials"), "got: {err}");
}

#[test]
fn frequencies_stay_below_nyquist_after_jitter_and_glide() {
    refused("hz = 3000", "hz = 23900", "hz");
    refused("hz = 1300", "hz = 20000", "hz");
    let fits = parse(&edited("hz = 3000", "hz = 19000")).unwrap();
    assert_eq!(fits.recipes.len(), 2);
}

#[test]
fn a_recipe_writes_its_name_or_one_stem_per_variant() {
    let book = parse(BOOK).unwrap();
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
    let err = parse(&quiet_partials).unwrap_err();
    assert!(err.to_string().contains("tap"), "got: {err}");
}
