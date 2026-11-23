//! Tests of G22's bench settings.

use std::path::Path;

use super::*;

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

/// A models folder for the tests, an example drive path; this PC's own is set in the local file.
const MODELS: &str = r"E:\models";

#[test]
fn a_bad_value_is_refused() {
    let good = shipped();
    let bad: [fn(&mut G22Bench); 19] = [
        |g| g.worker = "../../target/p11-gates/release/keyxtend.exe".into(),
        |g| g.weak_worker = "../../target/p11-gates/weak/keyxtend.exe".into(),
        |g| g.weak_worker = g.worker.clone(),
        |g| g.part_prefix.clear(),
        |g| g.weak_parts.clear(),
        |g| g.weak_parts.push("vulkan-1.dll".into()),
        |g| g.weak_parts.push(g.part_prefix.clone()),
        |g| g.weak_parts.push(format!("..\\{}x.dll", g.part_prefix)),
        |g| g.models_dir = "models".into(),
        |g| g.models_dir = r"E:\models\..\other".into(),
        |g| g.voice_dir.clear(),
        |g| g.voice_dir = r"E:\clips".into(),
        |g| g.detector.clear(),
        |g| g.detector = r"..\silero.gguf".into(),
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
fn the_models_folder_comes_unset_and_once_set_is_a_drive_path_whose_asks_pass() {
    let mut g = shipped();
    g.models_dir.clear();
    assert_eq!(g.check(), Ok(()), "unset loads, so every other gate runs");
    let unset = g.models().expect_err("no run without it");
    assert!(unset.contains(LOCAL_FILE), "{unset}");
    g.models_dir = MODELS.into();
    assert_eq!(g.check(), Ok(()));
    assert_eq!(g.models(), Ok(MODELS));
}

#[test]
fn the_tracked_settings_name_no_models_folder() {
    let p11: toml::Table = include_str!("../../../kx-gates-p11.toml")
        .parse()
        .expect("the P11 file parses");
    assert!(p11["g22_bench"].get("models_dir").is_none());
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
fn the_clips_are_exactly_the_tables_languages() {
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
}

/// A change that spoils every clip, and why it must be refused.
type Spoil = (fn(&mut Clip), &'static str);

#[test]
fn each_clip_is_a_plain_file_with_words_and_other_readings_unlike_it() {
    let bad: [Spoil; 5] = [
        (|c| c.sentence = " . ".into(), "a sentence of no words"),
        (|c| c.file.clear(), "a clip with no file"),
        (
            |c| c.file = r"E:\clips\speech-en.wav".into(),
            "a clip outside the voice folder",
        ),
        (
            |c| c.also = vec![c.sentence.clone()],
            "another reading that is the sentence",
        ),
        (
            |c| c.also = vec![" . ".into()],
            "another reading of no words",
        ),
    ];
    for (spoil, why) in bad {
        let mut g = shipped();
        g.clips.values_mut().for_each(spoil);
        assert!(g.check().is_err(), "{why}");
    }
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
fn each_sentence_is_the_one_its_clip_reads() {
    assert_eq!(shipped().check_credits(VOICE_README), Ok(()));
    let dir = crate::config::load().expect("the settings load").dir;
    assert_eq!(
        shipped().credited(&dir),
        Ok(()),
        "read from the voice folder"
    );
}

/// The 32 characters Windows-1252 reads bytes 0x80 to 0x9F as; the 5 it leaves unset read as the byte's own.
const CP1252_HIGH: [u32; 32] = [
    0x20AC, 0x81, 0x201A, 0x192, 0x201E, 0x2026, 0x2020, 0x2021, 0x2C6, 0x2030, 0x160, 0x2039,
    0x152, 0x8D, 0x17D, 0x8F, 0x90, 0x2018, 0x2019, 0x201C, 0x201D, 0x2022, 0x2013, 0x2014, 0x2DC,
    0x2122, 0x161, 0x203A, 0x153, 0x9D, 0x17E, 0x178,
];

/// `text`'s UTF-8 bytes read as Windows-1252, as PowerShell 5.1 reads a file with no byte-order mark.
fn as_1252(text: &str) -> String {
    let high = |b: u8| char::from_u32(CP1252_HIGH[usize::from(b - 0x80)]);
    let one = |b: u8| match b {
        0x80..=0x9F => high(b).expect("a character"),
        _ => char::from(b),
    };
    text.bytes().map(one).collect()
}

#[test]
fn a_sentence_re_encoded_or_a_clip_the_credits_lack_is_refused_by_its_language_alone() {
    let good = shipped();
    let en = &good.clips["en"].sentence;
    assert_eq!(&as_1252(en), en, "plain text reads the same");
    let mut garbled = good.clone();
    let ar = garbled.clips.get_mut("ar").expect("Arabic");
    ar.sentence = as_1252(&ar.sentence);
    let e = garbled
        .check_credits(VOICE_README)
        .expect_err("a re-encoded sentence");
    let (real, bad) = (&good.clips["ar"].sentence, &garbled.clips["ar"].sentence);
    assert!(
        e.contains(" ar ") && !e.contains(real) && !e.contains(bad),
        "{e}"
    );
    assert!(!e.contains('\n'), "one sentence");
    let mut renamed = good.clone();
    renamed.clips.get_mut("fr").expect("French").file = "other-fr.wav".into();
    let e = renamed.check_credits(VOICE_README).expect_err("no row");
    assert!(e.contains(" fr "), "{e}");
    let mut cut = good;
    let fr = cut.clips.get_mut("fr").expect("French");
    fr.sentence.truncate(fr.sentence.len() / 2);
    assert!(
        cut.check_credits(VOICE_README).is_err(),
        "part of the sentence"
    );
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

#[test]
fn a_clips_words_wrong_are_the_closest_readings_count_in_its_language() {
    let en = shipped().clips["en"].clone();
    let heard = en.also[0].clone();
    assert_eq!(
        en.wrong(&heard, "en").map(|w| w.words),
        Some(0),
        "the other reading"
    );
    let credited = score::wrong(&en.sentence, &heard, "en").expect("rules");
    assert!(
        credited.words > 0,
        "the credited spelling alone counts it wrong"
    );
    let none = en.wrong("", "en").expect("rules");
    assert!(
        none.words == none.total && none.total > 0,
        "nothing heard is every word wrong"
    );
    assert_eq!(en.wrong(&heard, "xx"), None, "no rules, no count");
}
