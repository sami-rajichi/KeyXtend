//! The sound tokens load the owner's sounds and refuse bad values; every file they name is a whole shipped sound,
//! and the bundle holds each once with its checked samples.

#![allow(
    clippy::expect_used,
    reason = "these helpers run only in tests, where a failed setup should fail the test"
)]

mod fuzz;
mod wav;

use std::path::{Path, PathBuf};

use kx_platform::ui::Clip;
use kx_test_support::tempdir::TempDir;
use kx_theme::SOUNDS_PATH;
use kx_theme::assets::{SOUND_DIR, bundle, sound_files};
use kx_theme::sounds::wav::{Fmt, HEADER_SLACK, PCM, chunks, samples};
use kx_theme::sounds::{Cue, Event, Sounds};

/// Bits in a byte.
const BYTE_BITS: u16 = 8;

/// The repo's `assets/` folder.
fn assets() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets")
}

/// The shipped sound tokens, as written in the file kx-theme compiles in.
fn shipped_text() -> String {
    std::fs::read_to_string(SOUNDS_PATH).expect("sounds.toml reads")
}

/// The shipped tokens, loaded.
fn shipped() -> Sounds {
    Sounds::shipped().expect("sounds.toml loads")
}

/// The shipped tokens with the first `from` replaced by `to`; `from` must occur in them.
fn edited(from: &str, to: &str) -> String {
    let text = shipped_text();
    assert!(text.contains(from), "sounds.toml lacks {from:?}");
    text.replacen(from, to, 1)
}

/// Asserts `text` is refused with a message that names `word`.
fn refused_text(text: &str, word: &str) {
    let err = Sounds::parse(text).expect_err("the tokens are refused");
    assert!(err.contains(word), "{err} lacks {word:?}");
}

/// Asserts the edited tokens are refused with a message that names `word`.
fn refused(from: &str, to: &str, word: &str) {
    refused_text(&edited(from, to), word);
}

#[test]
fn the_shipped_tokens_load_the_owners_four_sounds_in_order() {
    let s = shipped();
    let events: Vec<Event> = s.cues.iter().map(|(e, _)| *e).collect();
    assert_eq!(events, Event::ALL);
    let takes: Vec<String> = (1..=6).map(|i| format!("cherry-key-{i}.wav")).collect();
    let files: Vec<&[String]> = s.cues.iter().map(|(_, c)| c.files.as_slice()).collect();
    let mouse = ["mouse-click.wav", "mouse-press.wav", "card-down.wav"].map(|f| vec![f.to_owned()]);
    assert_eq!(files, [&takes[..], &mouse[0], &mouse[1], &mouse[2]]);
    assert!(s.play.voices >= 1);
    let names: Vec<&str> = Event::ALL.iter().map(|e| e.name()).collect();
    assert_eq!(
        names,
        ["key", "rclick", "pick", "drop"],
        "the tables sounds.toml holds"
    );
    let read = Sounds::parse(&shipped_text()).expect("the file at SOUNDS_PATH loads");
    assert_eq!(read, s, "SOUNDS_PATH names the file compiled in");
}

#[test]
fn the_format_is_the_ports_mono_pcm16_at_a_rate_every_speaker_plays() {
    let f = shipped().format;
    assert_eq!(f.channels, Clip::CHANNELS);
    assert_eq!(f.bits, Clip::SAMPLE_BYTES * BYTE_BITS);
    assert!(Clip::RATES.contains(&f.rate), "{} Hz", f.rate);
    assert_eq!(f.frame_bytes(), usize::from(Clip::SAMPLE_BYTES));
}

#[test]
fn every_named_file_is_whole_at_the_token_rate_and_within_the_limit() {
    let s = shipped();
    for name in s.files() {
        let bytes = std::fs::read(assets().join(SOUND_DIR).join(name)).expect("a named file reads");
        let data = samples(&bytes, &s.format).unwrap_or_else(|e| panic!("{name}: {e}"));
        let fmt = Fmt::parse(chunks(&bytes).expect("it has chunks").fmt).expect("its fmt reads");
        assert_eq!(fmt.rate, s.format.rate, "{name}");
        let whole = data.len().is_multiple_of(usize::from(Clip::SAMPLE_BYTES));
        assert!(!data.is_empty() && whole, "{name}: an even byte count");
        assert!(
            data.len() / s.format.frame_bytes() <= s.format.max_frames(),
            "{name}"
        );
    }
}

#[test]
fn unknown_keys_and_tables_are_refused() {
    refused("voices = 3", "voices = 3\ncolour = 1", "colour");
    refused("max_ms = 400", "max_ms = 400\nloud = true", "loud");
    refused("[play]", "[colour]\nx = 1\n\n[play]", "colour");
    refused("gain_db = 0", "gain_db = 0\nvolume = 1", "volume");
}

#[test]
fn a_gain_above_zero_or_not_a_number_is_refused() {
    refused("gain_db = 0", "gain_db = 1", "gain_db");
    refused("gain_db = 0", "gain_db = nan", "gain_db");
    let quiet =
        Sounds::parse(&edited("gain_db = 0", "gain_db = -6")).expect("a quieter event loads");
    assert!(quiet.cues[0].1.gain_db < 0.0);
}

#[test]
fn an_event_needs_a_file_and_every_event_is_there() {
    let text = shipped_text();
    let start = text.find("files = [").expect("a file list");
    let end = start + text[start..].find(']').expect("its end");
    refused_text(
        &format!("{}files = []{}", &text[..start], &text[end + 1..]),
        "files",
    );
    let drop = text.find("\n[drop]").expect("the drop table");
    refused_text(&text[..drop], "drop");
    refused("[play]", "[ply]", "play");
}

#[test]
fn a_file_that_is_not_a_plain_wav_name_is_refused() {
    let name = "\"card-down.wav\"";
    let bad = [
        "sounds/card-down.wav",
        "../card-down.wav",
        "a\\b.wav",
        "card-down.mp3",
        ".wav",
        "Card-Down.wav",
    ];
    for file in bad {
        refused(name, &format!("{file:?}"), "drop");
    }
}

#[test]
fn a_format_the_port_cannot_play_is_refused() {
    refused("bits = 16", "bits = 24", "bits");
    refused("channels = 1", "channels = 2", "channels");
    let (low, high) = (Clip::RATES.start() - 1, Clip::RATES.end() + 1);
    for rate in [0, low, high] {
        refused("rate = 48000", &format!("rate = {rate}"), "rate");
    }
    refused("ceiling_db = -3.0", "ceiling_db = 1.0", "ceiling_db");
    refused("trim_db = -50.0", "trim_db = 0.0", "trim_db");
    refused("max_ms = 400", "max_ms = 0", "max_ms");
    refused("fade_ms = 5", "fade_ms = 500", "fade_ms");
    refused("voices = 3", "voices = 0", "voices");
}

#[test]
fn each_file_is_listed_once_in_the_order_first_named() {
    let mut s = shipped();
    let named: Vec<String> = s.cues.iter().flat_map(|(_, c)| c.files.clone()).collect();
    assert_eq!(s.files(), named, "no shipped file is named twice");
    let again = named[0].clone();
    s.cues[3].1.files.push(again);
    assert_eq!(s.files(), named, "a file named again is listed once");
}

#[test]
fn the_bundle_holds_each_named_sound_once_with_its_samples() {
    let b = bundle(&assets()).expect("the bundle is found");
    let s = shipped();
    assert_eq!(b.sound_tokens, s, "the tokens the sounds were read with");
    let names: Vec<&str> = b.sounds.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(names, s.files());
    for f in &b.sounds {
        let bytes = std::fs::read(assets().join(SOUND_DIR).join(&f.name)).expect("it reads");
        assert_eq!(
            f.samples,
            samples(&bytes, &s.format).expect("it is valid"),
            "{}",
            f.name
        );
    }
}

#[test]
fn a_named_sound_missing_or_invalid_is_an_error_naming_it() {
    let dir = TempDir::new("sounds").expect("a temp folder");
    let folder = dir.path().join(SOUND_DIR);
    std::fs::create_dir(&folder).expect("the sounds folder");
    let good = std::fs::read(assets().join(SOUND_DIR).join("mouse-press.wav")).expect("it reads");
    std::fs::write(folder.join("good.wav"), &good).expect("written");
    let stereo = wav::wav(PCM, 2, 48_000, 16, &[0; 8]);
    std::fs::write(folder.join("bad.wav"), stereo).expect("written");
    std::fs::write(folder.join("notes.txt"), "not a sound").expect("written");
    // One byte over the largest sound, so its size refuses it before it is read.
    let f = shipped().format;
    let max = f.max_frames() * f.frame_bytes() + HEADER_SLACK;
    std::fs::File::create(folder.join("big.wav"))
        .and_then(|big| big.set_len(u64::try_from(max + 1).expect("a small size")))
        .expect("sized");
    let with = |name: &str| Sounds {
        cues: vec![(
            Event::Key,
            Cue {
                files: vec!["good.wav".into(), name.into()],
                gain_db: 0.0,
            },
        )],
        ..shipped()
    };
    let found = sound_files(dir.path(), &with("good.wav")).expect("an unnamed file is fine");
    assert_eq!(found.len(), 1);
    for name in ["bad.wav", "gone.wav", "big.wav"] {
        let err = sound_files(dir.path(), &with(name)).expect_err("refused");
        assert!(err.contains(name), "{err}");
    }
    let err = sound_files(dir.path(), &with("big.wav")).expect_err("refused");
    assert!(err.contains("larger than"), "{err}");
}
