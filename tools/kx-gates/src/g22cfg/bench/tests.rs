//! Tests of G22's bench settings.

use std::path::Path;

use super::*;

/// The gate worker's command line, which names its sound flag.
const WORKER_ARGS: &str = include_str!("../../../../../apps/keyxtend-worker/src/args.rs");
/// The shipped voice files' credits, which name each clip's sentence.
const VOICE_README: &str = include_str!("../../../../../assets/voice/README.md");
/// The folder of those files and credits, from kx-gates' own folder.
const VOICE_DIR: &str = "../../assets/voice";

/// The shipped bench settings.
fn shipped() -> G22Bench {
    crate::config::load().expect("the settings load").g22_bench
}

#[test]
fn the_shipped_bench_passes_its_check() {
    assert_eq!(shipped().check(), Ok(()));
}

#[test]
fn a_bad_value_is_refused() {
    let good = shipped();
    let bad: [fn(&mut G22Bench); 27] = [
        |g| g.worker = "../../target/p11-gates/release/keyxtend.exe".into(),
        |g| g.weak_worker = "../../target/p11-gates/weak/keyxtend.exe".into(),
        |g| g.weak_worker = g.worker.clone(),
        |g| g.part_prefix.clear(),
        |g| g.weak_parts.clear(),
        |g| g.weak_parts.push("vulkan-1.dll".into()),
        |g| g.weak_parts.push(g.part_prefix.clone()),
        |g| g.weak_parts.push(format!("..\\{}x.dll", g.part_prefix)),
        |g| g.sound_flag = "sound".into(),
        |g| g.sound_flag = "--".into(),
        |g| g.sound_flag = "--so und".into(),
        |g| g.models_dir = "models".into(),
        |g| g.models_dir = r"D:\dev\..\models".into(),
        |g| g.voice_dir.clear(),
        |g| g.voice_dir = r"D:\dev\tmp".into(),
        |g| g.detector.clear(),
        |g| g.detector = r"..\silero.gguf".into(),
        |g| g.weak_cores = 0,
        |g| g.weak_cores = u8::MAX,
        |g| g.weak_core_pct = 0,
        |g| g.weak_core_pct = FULL_PCT + 1,
        |g| g.weak_mb = 0,
        |g| g.tail_ms = 0,
        |g| g.max_rtf = 0.0,
        |g| g.max_weak_rtf = f64::NAN,
        |g| g.max_weak_rtf = g.max_rtf / 2.0,
        |g| g.end_wait_ms = 0,
    ];
    for (i, spoil) in bad.iter().enumerate() {
        let mut g = good.clone();
        spoil(&mut g);
        assert!(g.check().is_err(), "case {i} passed");
    }
}

#[test]
fn the_weak_limit_keeps_each_piece_within_the_hosts_wait() {
    let settings = VoiceSettings::from_defaults().expect("the defaults load");
    let ceiling = f64::from(settings.piece_wait_s) / f64::from(settings.piece_s);
    let mut g = shipped();
    g.max_weak_rtf = ceiling;
    assert_eq!(g.check(), Ok(()), "at the ceiling");
    g.max_weak_rtf = ceiling * 2.0;
    assert!(g.check().is_err(), "past it");
}

#[test]
fn the_clips_are_the_tables_languages_each_a_plain_file_with_words() {
    let good = shipped();
    let mut gone = good.clone();
    gone.clips.pop_first();
    assert!(gone.check().is_err(), "a language left out");
    let mut extra = good.clone();
    let clip = good.clips.values().next().cloned().expect("a clip");
    extra.clips.insert("de".into(), clip);
    assert!(
        extra.check().is_err(),
        "a language the table has no clip for"
    );
    let mut mute = good.clone();
    mute.clips
        .values_mut()
        .for_each(|c| c.sentence = " . ".into());
    assert!(mute.check().is_err(), "a sentence of no words");
    let mut nameless = good.clone();
    nameless.clips.values_mut().for_each(|c| c.file.clear());
    assert!(nameless.check().is_err(), "a clip with no file");
    let mut outside = good.clone();
    let owner = r"D:\dev\tmp\kx-p11-trial\owner-en.wav";
    outside
        .clips
        .values_mut()
        .for_each(|c| c.file = owner.into());
    assert!(outside.check().is_err(), "a clip outside the voice folder");
    let mut same = good.clone();
    same.clips
        .values_mut()
        .for_each(|c| c.also = vec![c.sentence.clone()]);
    assert!(
        same.check().is_err(),
        "another reading that is the sentence"
    );
    let mut blank = good;
    blank
        .clips
        .values_mut()
        .for_each(|c| c.also = vec![" . ".into()]);
    assert!(blank.check().is_err(), "another reading of no words");
}

#[test]
fn the_english_reading_forgives_only_the_split_the_reference_makes() {
    let en = &shipped().clips["en"];
    let joined = en.sentence.replace("dragon flies", "dragonflies");
    assert_eq!(
        en.also,
        std::slice::from_ref(&joined),
        "the one reading models write"
    );
    let words = |set: &str| score::wrong(set, &joined, "en").map(|w| w.words);
    assert_eq!(
        words(&en.sentence),
        Some(2),
        "the credited sentence's quirk"
    );
    assert_eq!(words(&en.also[0]), Some(0));
}

#[test]
fn the_sound_flag_is_the_gate_workers_own() {
    let flag = shipped().sound_flag;
    let line = format!("SOUND_FLAG: &str = \"{flag}\";");
    assert!(WORKER_ARGS.contains(&line), "keyxtend-worker names {flag}");
}

#[test]
fn each_sentence_is_the_one_its_clip_reads() {
    for (lang, clip) in shipped().clips {
        let name = format!("`{}`", clip.file);
        let row = (VOICE_README.lines()).find(|l| l.contains(&name) && l.contains(&clip.sentence));
        assert!(row.is_some(), "{lang}: the README's sentence for {name}");
    }
}

#[test]
fn the_voice_folder_is_the_shipped_one_whose_readme_credits_each_file() {
    let g = shipped();
    let dir = crate::config::load().expect("the settings load").dir;
    let real = |p: &Path| std::fs::canonicalize(p).expect("the folder is there");
    let shipped_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(VOICE_DIR);
    assert_eq!(real(&dir.join(&g.voice_dir)), real(&shipped_dir));
    for name in g.clips.values().map(|c| &c.file).chain([&g.detector]) {
        assert!(VOICE_README.contains(&format!("`{name}`")), "{name}");
        assert!(shipped_dir.join(name).is_file(), "{name}");
    }
}
