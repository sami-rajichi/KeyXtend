use std::collections::HashSet;

use serde_json::json;

use super::fixture::{BOOK, TAG_PCM, crossings, encode, header, mono, pcm, peak, sine};
use super::recipe::{Book, parse};
use super::units::{db_gain, index, samples};
use super::*;
use crate::test_support::TempDir;

/// The recipe file the project ships.
const SHIPPED: &str = include_str!("../../../assets/sounds/recipes.toml");

fn book() -> Book {
    parse(BOOK).unwrap()
}

/// The highest 16-bit value the ceiling allows.
fn ceiling(book: &Book) -> f64 {
    db_gain(book.format.ceiling_db) * f64::from(i16::MAX)
}

/// Every file of every recipe in `book`, rendered.
fn render_every(book: &Book) -> Vec<(String, Sound)> {
    let mut out = Vec::new();
    for recipe in &book.recipes {
        for stem in recipe.stems() {
            let sound = render_clip(book, recipe, &stem).unwrap();
            out.push((stem, sound));
        }
    }
    out
}

/// Asserts `wav` is in `book`'s shipping format, within its length limit and under its ceiling.
fn assert_shipping(book: &Book, wav: &[u8]) -> Vec<i16> {
    let (tag, channels, rate, bits, data) = header(wav);
    assert_eq!(
        (tag, channels, bits),
        (TAG_PCM, book.format.channels, book.format.bits)
    );
    assert_eq!(rate, book.format.rate);
    let pcm = pcm(wav);
    assert_eq!(data as usize, pcm.len() * 2);
    assert!(pcm.len() <= samples(book.format.max_ms, f64::from(rate)));
    assert!(
        f64::from(peak(&pcm)) <= ceiling(book),
        "the peak is over the ceiling"
    );
    pcm
}

#[test]
fn a_recipe_renders_the_same_bytes_twice() {
    let book = book();
    let thud = &book.recipes[1];
    let first = render_clip(&book, thud, "thud").unwrap();
    let second = render_clip(&book, thud, "thud").unwrap();
    assert_eq!(first.bytes, second.bytes);
}

#[test]
fn a_render_is_a_pcm16_mono_48k_file_of_the_recipe_length_under_the_ceiling() {
    let book = book();
    for (stem, sound) in render_every(&book) {
        let pcm = assert_shipping(&book, &sound.bytes);
        let recipe = book
            .recipes
            .iter()
            .find(|r| stem.starts_with(&r.name))
            .unwrap();
        assert_eq!(pcm.len(), samples(recipe.length_ms, 48_000.0), "{stem}");
        assert!((sound.ms - recipe.length_ms).abs() < 1e-9, "{stem}");
        let target = ceiling(&book) * db_gain(recipe.gain_db);
        assert!(f64::from(peak(&pcm)) > target * db_gain(-book.variants.level_db) - 1.0);
        assert_eq!(*pcm.last().unwrap(), 0, "{stem} does not fade out");
    }
}

#[test]
fn variants_differ_but_each_is_deterministic() {
    let book = book();
    let first = render_every(&book);
    let second = render_every(&book);
    let names: Vec<&str> = first.iter().map(|(stem, _)| stem.as_str()).collect();
    assert_eq!(names, ["tap-1", "tap-2", "tap-3", "thud"]);
    for ((_, a), (_, b)) in first.iter().zip(&second) {
        assert_eq!(a.bytes, b.bytes);
    }
    let taps: HashSet<&Vec<u8>> = first[..3].iter().map(|(_, s)| &s.bytes).collect();
    assert_eq!(taps.len(), 3);
}

#[test]
fn render_all_writes_one_file_per_variant() {
    let dir = TempDir::new("sounds-render");
    let out = dir.path().join("out");
    let written = render_all(&book(), &out).unwrap();
    let mut names: Vec<String> = std::fs::read_dir(&out)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(names, ["tap-1.wav", "tap-2.wav", "tap-3.wav", "thud.wav"]);
    assert_eq!(written.len(), 4);
    assert_eq!(written[3].0, out.join("thud.wav"));
    assert!((written[3].1 - 100.0).abs() < 1e-9);
}

#[test]
fn import_turns_a_44k_stereo_24_bit_sine_into_48k_mono_16_bit_at_the_same_pitch() {
    let book = book();
    let quiet = vec![0.0; 2_205];
    let tone = sine(1_000.0, 44_100, 300, 1.0);
    let signal: Vec<f64> = quiet.iter().chain(&tone).chain(&quiet).copied().collect();
    let frames: Vec<Vec<f64>> = signal.iter().map(|s| vec![s * 0.5, s * 0.25]).collect();
    let sound = import_clip(&book, &encode(TAG_PCM, 44_100, 24, &frames), Cut::default()).unwrap();
    let pcm = assert_shipping(&book, &sound.bytes);
    // The tone is 300 ms from phase zero, so a trim keeps all of it but its first, silent sample.
    let tone_len = samples(300.0, 48_000.0);
    assert!(
        pcm.len().abs_diff(tone_len) <= 2,
        "trimmed to {}",
        pcm.len()
    );
    let hz = index(crossings(&pcm)) / 2.0 / (index(pcm.len()) / 48_000.0);
    assert!((hz - 1_000.0).abs() < 3.0, "pitch {hz} Hz");
    assert!(
        f64::from(peak(&pcm)) >= ceiling(&book) - 1.0,
        "not normalised"
    );
}

#[test]
fn import_cuts_the_given_window_and_caps_the_length() {
    let book = book();
    let wav = encode(TAG_PCM, 48_000, 16, &mono(&sine(440.0, 48_000, 1_000, 0.3)));
    let cut = Cut {
        start_ms: Some(100),
        len_ms: Some(50),
    };
    assert_eq!(
        pcm(&import_clip(&book, &wav, cut).unwrap().bytes).len(),
        2_400
    );
    let whole = import_clip(&book, &wav, Cut::default()).unwrap();
    assert_eq!(pcm(&whole.bytes).len(), 19_200);
    assert!((whole.ms - 400.0).abs() < 1e-9);
}

#[test]
fn trim_mode_caps_the_length_across_a_rate_change() {
    let book = book();
    let wav = encode(TAG_PCM, 44_100, 16, &mono(&sine(440.0, 44_100, 1_000, 0.3)));
    let sound = import_clip(&book, &wav, Cut::default()).unwrap();
    assert_eq!(pcm(&sound.bytes).len(), 19_200);
}

#[test]
fn a_window_longer_than_the_limit_is_refused() {
    let book = book();
    let wav = encode(TAG_PCM, 44_100, 16, &mono(&sine(440.0, 44_100, 1_000, 0.3)));
    for (start_ms, len_ms) in [(Some(0), Some(500)), (Some(100), None), (None, Some(401))] {
        let err = import_clip(&book, &wav, Cut { start_ms, len_ms }).unwrap_err();
        assert!(
            matches!(err, SoundError::TooLong(_)),
            "{start_ms:?} {len_ms:?}"
        );
        assert!(err.to_string().contains("400"), "{err}");
    }
    let fits = Cut {
        start_ms: Some(600),
        len_ms: Some(400),
    };
    assert_eq!(
        pcm(&import_clip(&book, &wav, fits).unwrap().bytes).len(),
        19_200
    );
}

#[test]
fn import_refuses_an_empty_window_silence_and_bad_files() {
    let book = book();
    let wav = encode(TAG_PCM, 48_000, 16, &mono(&sine(440.0, 48_000, 100, 0.3)));
    let late = Cut {
        start_ms: Some(100),
        len_ms: None,
    };
    assert!(matches!(
        import_clip(&book, &wav, late),
        Err(SoundError::Window)
    ));
    let silent = encode(TAG_PCM, 48_000, 16, &mono(&[0.0; 480]));
    assert!(matches!(
        import_clip(&book, &silent, Cut::default()),
        Err(SoundError::Silent)
    ));
    for len in 0..wav.len() {
        assert!(
            import_clip(&book, &wav[..len], Cut::default()).is_err(),
            "{len} bytes"
        );
    }
}

#[test]
fn the_shipped_recipes_parse_and_render_within_the_format_limits() {
    let book = parse(SHIPPED).unwrap();
    let names: HashSet<&str> = book.recipes.iter().map(|r| r.name.as_str()).collect();
    assert_eq!(names.len(), book.recipes.len(), "recipe names repeat");
    for (stem, sound) in render_every(&book) {
        assert_shipping(&book, &sound.bytes);
        assert!(sound.ms <= book.format.max_ms, "{stem} is too long");
    }
}

/// `cargo metadata` JSON whose `[workspace.metadata.sounds]` is `section`.
fn metadata(section: &serde_json::Value) -> String {
    json!({ "workspace_root": "/ws", "metadata": { "sounds": section } }).to_string()
}

#[test]
fn the_sounds_section_loads_with_the_workspace_root() {
    let section = json!({ "recipes": "assets/sounds/recipes.toml", "out_dir": "target/sounds" });
    let (root, config) = load(&metadata(&section)).unwrap();
    assert_eq!(root, PathBuf::from("/ws"));
    assert_eq!(config.recipes, "assets/sounds/recipes.toml");
    assert_eq!(config.out_dir, "target/sounds");
}

#[test]
fn a_bad_or_missing_sounds_section_is_refused() {
    for bad in [
        json!({ "recipes": "../r.toml", "out_dir": "target/sounds" }),
        json!({ "recipes": "r.toml", "out_dir": "C:/sounds" }),
        json!({ "recipes": "r.toml" }),
        json!({ "recipes": "r.toml", "out_dir": "o", "extra": 1 }),
    ] {
        let err = load(&metadata(&bad)).unwrap_err();
        assert!(matches!(err, LoadError::BadConfig(_)), "{bad}: {err}");
    }
    let none = r#"{"workspace_root": "/ws", "metadata": {}}"#;
    assert!(matches!(load(none), Err(LoadError::MissingSection(_))));
}
