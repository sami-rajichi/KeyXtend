//! Tests of the meter on a tiny pack built in memory; nothing is downloaded. The pack has no chains, so its chips
//! rank by frequency alone and every count below is worked out by hand.

use std::collections::BTreeMap;
use std::path::Path;

use kx_mod_predict::{Flags, Pack, PackWriter, PredictSettings};
use kx_module_api::ports::{PACK_INDEX, index_text, pack_file};

use super::tally::Tally;
use super::typist::Typist;
use super::verdict::failures;
use super::{Opts, Paths, held_lines, run};
use crate::test_support::TempDir;
use crate::words::fixture::{self, manifest_with};
use crate::words::manifest::{Manifest, MeterRules};
use crate::words::{MANIFEST, WordsError};

/// The pack's words, most frequent first: three chips show `the cold cut` before a letter and `cat car` after `ca`.
const WORDS: [(&str, u8); 6] = [
    ("the", 100),
    ("cold", 98),
    ("cut", 96),
    ("cat", 94),
    ("car", 92),
    ("sat", 50),
];
/// The chips in the lane: the fewest the settings take, so the tiny pack can leave a word out.
const CHIPS: u8 = 3;
/// What the sentence tests type: a capital, a word offered after two letters and a full stop.
const SENTENCE: &str = "The car sat.";

/// The tiny pack of `iso`.
fn pack(iso: &str) -> Vec<u8> {
    let mut writer = PackWriter::new(iso, Flags::default(), 1).unwrap();
    for (word, freq) in WORDS {
        writer.add_word(word, freq).unwrap();
    }
    writer.finish().unwrap()
}

/// The module's defaults with `CHIPS` chips.
fn settings() -> PredictSettings {
    let mut settings = PredictSettings::from_defaults().unwrap();
    settings.chips = CHIPS;
    settings
}

/// The counts after typing `lines` in English, with the learner when `learn`.
fn typed(lines: &[&str], learn: bool) -> Tally {
    let bytes = pack("en");
    let seed = Pack::parse(&bytes).unwrap();
    let mut typist = Typist::new("en", &seed, &settings(), learn).unwrap();
    for line in lines {
        typist.sentence(line);
    }
    typist.into_tally()
}

/// The fixture manifest's meter rules: `en` must save 40%.
fn rules() -> MeterRules {
    manifest_with("").unwrap().meter
}

#[test]
fn a_known_sentence_costs_the_presses_worked_out_by_hand() {
    // Without: Shift and T, h, e and a space (5); c, a, r and a space (4); s, a, t and the stop (4): 13.
    // With: Shift, then the chip The (2); c, a, then the chip car (3); s, then the chip sat, whose space stays before
    // the stop, then the stop (3): 8.
    let t = typed(&[SENTENCE], false);
    assert_eq!((t.sentences, t.words, t.without, t.with), (1, 3, 13, 8));
    assert!((t.saved_pct() - 100.0 * 5.0 / 13.0).abs() < 1e-9, "{t:?}");
}

#[test]
fn a_word_offered_after_two_letters_costs_them_and_a_pick_and_one_never_offered_costs_every_key() {
    // car: c and a, then its chip types `car ` (3 presses for 4 keys); dog is not in the pack: d, o, g.
    let t = typed(&["car dog"], false);
    assert_eq!((t.without, t.with), (7, 6));
    // dog and its space (4), then car at the sentence's end (3).
    let t = typed(&["dog car"], false);
    assert_eq!((t.without, t.with), (7, 7));
    // The chip cat only starts cats, so it is never picked.
    let t = typed(&["cats"], false);
    assert_eq!((t.without, t.with), (4, 4));
}

#[test]
fn a_capital_costs_its_shift_on_both_sides_and_the_chip_comes_capitalised() {
    // Shift, T, h, e: 4. Before Shift the chip reads `the`; with Shift latched it reads `The`: Shift and a pick.
    let t = typed(&["The"], false);
    assert_eq!((t.without, t.with), (4, 2));
    let t = typed(&["the"], false);
    assert_eq!((t.without, t.with), (3, 1), "no Shift without a capital");
}

#[test]
fn a_chip_space_stays_before_closing_marks_but_never_splits_a_joined_word() {
    // cold is the first chip before any letter: picked, then the stop (2 of 5).
    let t = typed(&["cold."], false);
    assert_eq!((t.without, t.with), (5, 2));
    // Before -cut its space would split the word: c, o, l, d and the hyphen are typed; the tracker offers nothing
    // right after a mark, so c too, then the chip cut (7 of 8).
    let t = typed(&["cold-cut"], false);
    assert_eq!((t.without, t.with), (8, 7));
}

#[test]
fn a_digit_token_costs_the_same_on_both_sides() {
    let t = typed(&["42 7"], false);
    assert_eq!((t.without, t.with), (4, 4));
}

#[test]
fn the_learner_run_offers_a_new_word_from_its_second_use() {
    // dog costs its keys each time with the seed alone; learned twice, the third comes as the first chip.
    assert_eq!(typed(&["dog dog dog"], false).with, 11);
    assert_eq!(typed(&["dog dog dog"], true).with, 9);
}

#[test]
fn every_scorer_call_is_timed() {
    let t = typed(&[SENTENCE], false);
    assert!(!t.times.is_empty());
    assert!(t.percentile(50) <= t.percentile(99));
}

#[test]
fn the_verdict_fails_a_gate_language_under_the_target_and_never_another() {
    let rules = rules();
    let asked = ["en".to_owned(), "fr".to_owned()];
    let saved = |en: f64, fr: f64| BTreeMap::from([("en".to_owned(), en), ("fr".to_owned(), fr)]);
    assert!(failures(&rules, &asked, &saved(41.0, 0.0)).is_empty());
    assert_eq!(failures(&rules, &asked, &saved(39.0, 99.0)).len(), 1);
    assert_eq!(
        failures(&rules, &asked, &BTreeMap::new()).len(),
        1,
        "a gate language with no held sentences"
    );
    let fr = ["fr".to_owned()];
    assert!(
        failures(&rules, &fr, &BTreeMap::new()).is_empty(),
        "a gate language not asked for"
    );
}

#[test]
fn a_meter_rule_out_of_its_range_is_refused_and_its_edges_are_taken() {
    let text = fixture::manifest_text("");
    let swaps = [
        (r#"gate = ["en"]"#, r#"gate = ["EN"]"#, false),
        ("min_saved_pct = 40", "min_saved_pct = 101", false),
        ("max_sentences = 2", "max_sentences = 0", false),
        ("percentiles = [50, 99]", "percentiles = []", false),
        ("percentiles = [50, 99]", "percentiles = [99, 50]", false),
        ("percentiles = [50, 99]", "percentiles = [0, 50]", false),
        ("percentiles = [50, 99]", "percentiles = [50, 101]", false),
        ("ceiling_us = 10000", "ceiling_us = 0", false),
        ("speed_learns = 10", "speed_learns = 0", false),
        ("speed_samples = 2", "speed_samples = 0", false),
        ("min_saved_pct = 40", "min_saved_pct = 100", true),
        ("percentiles = [50, 99]", "percentiles = [1, 100]", true),
    ];
    for (from, to, taken) in swaps {
        assert!(text.contains(from), "{from}");
        let result = Manifest::parse(&text.replacen(from, to, 1));
        assert_eq!(result.is_ok(), taken, "{to}: {result:?}");
    }
}

#[test]
fn the_project_gate_is_english_and_french_at_forty_percent() {
    let rules = Manifest::parse(MANIFEST).unwrap().meter;
    assert_eq!(
        (rules.gate, rules.min_saved_pct),
        (vec!["en".to_owned(), "fr".to_owned()], 40)
    );
}

#[test]
fn max_sentences_limits_the_lines_read() {
    let dir = TempDir::new("words-meter-held");
    let file = dir.path().join("en.txt");
    std::fs::write(&file, "one\n\ntwo\nthree\n").unwrap();
    assert_eq!(held_lines(&file, 3).unwrap(), ["one", "two"]);
}

/// A packs folder with the tiny pack of `en` and `fr` and an index that also lists `de`, and held sentences for
/// `en` and `fr` only.
fn folders(dir: &Path) -> Paths {
    let (packs, held) = (dir.join("packs"), dir.join("held"));
    std::fs::create_dir_all(&packs).unwrap();
    std::fs::create_dir_all(&held).unwrap();
    let index = index_text(&["de", "en", "fr"]).unwrap();
    std::fs::write(packs.join(PACK_INDEX), index).unwrap();
    for iso in ["en", "fr"] {
        std::fs::write(packs.join(pack_file(iso).unwrap()), pack(iso)).unwrap();
        std::fs::write(held.join(format!("{iso}.txt")), format!("{SENTENCE}\n")).unwrap();
    }
    Paths { packs, held }
}

#[test]
fn the_run_prints_counts_only_and_fails_a_gate_language_under_the_target() {
    let dir = TempDir::new("words-meter-run");
    let paths = folders(dir.path());
    let opts = Opts {
        chips: Some(CHIPS),
        ..Opts::default()
    };
    let mut said = Vec::new();
    let result = run(&rules(), &opts, &paths, &mut |l| said.push(l.to_owned()));
    let Err(WordsError::Meter(why)) = result else {
        panic!("en saves 38% of 40%: {result:?}");
    };
    assert_eq!(
        said.len(),
        4,
        "two runs per language with held sentences: {said:?}"
    );
    for text in said.iter().chain([&why]) {
        assert!(!text.contains("car") && !text.contains("sat"), "{text}");
    }
}

#[test]
fn the_run_measures_only_the_language_asked_for() {
    let dir = TempDir::new("words-meter-lang");
    let paths = folders(dir.path());
    let opts = Opts {
        chips: Some(CHIPS),
        lang: Some("fr"),
        ..Opts::default()
    };
    let mut said = Vec::new();
    run(&rules(), &opts, &paths, &mut |l| said.push(l.to_owned())).unwrap();
    assert_eq!(said.len(), 3, "two runs, then the verdict: {said:?}");
    assert!(
        said.iter().take(2).all(|l| l.starts_with("fr ")),
        "{said:?}"
    );
    let unknown = Opts {
        lang: Some("it"),
        ..Opts::default()
    };
    assert!(run(&rules(), &unknown, &paths, &mut |_| {}).is_err());
}

#[test]
fn the_speed_run_fills_a_learner_and_times_both_query_kinds() {
    let dir = TempDir::new("words-meter-speed");
    let paths = folders(dir.path());
    let opts = Opts {
        speed: true,
        ..Opts::default()
    };
    let mut said = Vec::new();
    run(&rules(), &opts, &paths, &mut |l| said.push(l.to_owned())).unwrap();
    assert_eq!(
        said.len(),
        3,
        "one line per language, then the verdict: {said:?}"
    );
    assert!(said.iter().all(|l| !l.contains("car")), "{said:?}");
}
