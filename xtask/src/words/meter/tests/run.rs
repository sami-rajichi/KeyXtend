//! The meter end to end on temp folders: what it prints, which languages it measures, and when it fails.

use std::path::Path;

use kx_mod_predict::{Flags, PackWriter};
use kx_module_api::ports::{PACK_INDEX, index_text, pack_file};

use super::super::speed::KINDS;
use super::super::{NOTHING, Opts, Paths, RELEASE_RUN, SLOW, UNOPTIMISED, run};
use super::{CHIPS, SENTENCE, pack, rules};
use crate::test_support::TempDir;
use crate::words::WordsError;
use crate::words::read::held_file;

/// A packs folder holding the tiny pack of each language of `index`, which its index lists, and held sentences for
/// each language of `held`.
fn folders(dir: &Path, index: &[&str], held: &[&str]) -> Paths {
    let paths = Paths {
        packs: dir.join("packs"),
        held: dir.join("held"),
    };
    std::fs::create_dir_all(&paths.packs).unwrap();
    std::fs::create_dir_all(&paths.held).unwrap();
    std::fs::write(paths.packs.join(PACK_INDEX), index_text(index).unwrap()).unwrap();
    for iso in index {
        std::fs::write(paths.packs.join(pack_file(iso).unwrap()), pack(iso)).unwrap();
    }
    for iso in held {
        std::fs::write(held_file(&paths.held, iso), format!("{SENTENCE}\n")).unwrap();
    }
    paths
}

/// A Chinese pack, whose script spaces no words, so `guesses` returns at once.
fn han_pack() -> Vec<u8> {
    let mut writer = PackWriter::new("zh", Flags::default(), 1).unwrap();
    for (word, freq) in [("中国", 100), ("学生", 90)] {
        writer.add_word(word, freq).unwrap();
    }
    writer.finish().unwrap()
}

/// What `run` printed and gave for `opts` over `paths`.
fn ran(opts: &Opts<'_>, paths: &Paths) -> (Vec<String>, Result<(), WordsError>) {
    let mut said = Vec::new();
    let result = run(&rules(), opts, paths, &mut |l| said.push(l.to_owned()));
    (said, result)
}

/// A run that counts presses with the lane's chips.
fn counting() -> Opts<'static> {
    Opts {
        chips: Some(CHIPS),
        ..Opts::default()
    }
}

/// A run that times the scorer.
fn timing() -> Opts<'static> {
    Opts {
        speed: true,
        ..Opts::default()
    }
}

/// Why the meter failed.
fn why(result: Result<(), WordsError>) -> String {
    match result {
        Err(WordsError::Meter(why)) => why,
        other => panic!("the meter should fail: {other:?}"),
    }
}

#[test]
fn the_run_prints_counts_only_and_fails_a_gate_language_under_the_target() {
    let dir = TempDir::new("words-meter-run");
    let paths = folders(dir.path(), &["de", "en", "fr"], &["en", "fr"]);
    let (said, result) = ran(&counting(), &paths);
    let why = why(result);
    assert!(why.contains("38.4%"), "5/13 printed rounded down: {why}");
    assert_eq!(
        said.len(),
        4,
        "two runs per language with held sentences: {said:?}"
    );
    for text in said.iter().chain([&why]) {
        assert!(!text.contains("car") && !text.contains("sat"), "{text}");
    }
    assert!(
        said.iter().all(|l| l.contains(SLOW) == UNOPTIMISED),
        "{said:?}"
    );
}

#[test]
fn the_run_measures_only_the_language_asked_for() {
    let dir = TempDir::new("words-meter-lang");
    let paths = folders(dir.path(), &["de", "en", "fr"], &["en", "fr"]);
    let fr = Opts {
        lang: Some("fr"),
        ..counting()
    };
    let (said, result) = ran(&fr, &paths);
    result.unwrap();
    assert_eq!(said.len(), 3, "two runs, then the verdict: {said:?}");
    assert!(
        said.iter().take(2).all(|l| l.starts_with("fr ")),
        "{said:?}"
    );
    let unknown = Opts {
        lang: Some("it"),
        ..counting()
    };
    assert!(ran(&unknown, &paths).1.is_err());
}

#[test]
fn a_full_run_fails_when_a_gate_language_is_not_shipped_or_has_no_held_sentences() {
    for (index, held) in [(&["fr"][..], &["fr"][..]), (&["en", "fr"], &["fr"])] {
        let dir = TempDir::new("words-meter-gate");
        let paths = folders(dir.path(), index, held);
        let why = why(ran(&counting(), &paths).1);
        assert!(why.starts_with("en "), "{index:?} {held:?}: {why}");
    }
}

#[test]
fn a_run_that_measures_nothing_fails() {
    let dir = TempDir::new("words-meter-nothing");
    let paths = folders(dir.path(), &["de", "en", "fr"], &["en", "fr"]);
    let de = Opts {
        lang: Some("de"),
        ..counting()
    };
    let (said, result) = ran(&de, &paths);
    assert!(said.is_empty() && why(result).contains(NOTHING), "{said:?}");
    let empty = TempDir::new("words-meter-empty");
    let paths = folders(empty.path(), &[], &[]);
    assert!(why(ran(&timing(), &paths).1).contains(NOTHING));
}

#[test]
fn the_speed_run_times_every_shipped_language_in_both_query_kinds() {
    let dir = TempDir::new("words-meter-speed");
    let paths = folders(dir.path(), &["de", "en", "fr"], &["en", "fr"]);
    let (said, result) = ran(&timing(), &paths);
    for (line, iso) in said.iter().zip(["de ", "en ", "fr "]) {
        assert!(
            line.starts_with(iso),
            "de needs no held sentences: {said:?}"
        );
        assert!(KINDS.iter().all(|(kind, _)| line.contains(kind)), "{line}");
        assert_eq!(line.contains(SLOW), UNOPTIMISED, "{line}");
        assert!(!line.contains("car"), "{line}");
    }
    if UNOPTIMISED {
        assert_eq!(said.len(), 3, "{said:?}");
        assert!(
            why(result).contains(RELEASE_RUN),
            "the refusal names the release run"
        );
    } else {
        result.unwrap();
        assert_eq!(said.len(), 4, "then the verdict: {said:?}");
    }
}

#[test]
fn a_pack_whose_script_spaces_no_words_sorts_nothing() {
    let dir = TempDir::new("words-meter-han");
    let paths = folders(dir.path(), &[], &[]);
    std::fs::write(paths.packs.join(PACK_INDEX), index_text(&["zh"]).unwrap()).unwrap();
    std::fs::write(paths.packs.join(pack_file("zh").unwrap()), han_pack()).unwrap();
    let (said, _) = ran(&timing(), &paths);
    let empty = said
        .first()
        .map(|l| l.matches("sort buffer up to 0 B").count());
    assert_eq!(empty, Some(KINDS.len()), "{said:?}");
}
