//! Tests of G22's bench: the clip's length, the gate worker's command line and parts, the end watch and one take on the
//! fake speech service; the table's are in `table`. None starts a worker or sleeps.

mod table;

use std::path::PathBuf;
use std::time::Duration;

use kx_mod_voice::VoiceSettings;
use kx_mod_voice::score::Wrong;
use kx_module_api::ports::{
    SpeechAsk, SpeechDone, SpeechError, SpeechModel, SpeechNote, SpeechStage,
};
use kx_module_api::{Redacted, Zeroizing};
use kx_platform::worker::Launch;
use kx_platform_fake::{FakeSpeech, SpeechCall};

use super::clip::length_ms;
use super::spawn::{Alive, args, only};
use super::take::{Script, take};

/// A WAV header's fields as these tests write them: 16 kHz mono 32-bit float, as the shipped clips are.
const RATE: u32 = 16_000;
const BLOCK: u16 = 4;
const BITS: u16 = 32;
const FLOAT: u16 = 3;

/// One chunk: its id, its size and its body, padded to an even length as RIFF wants.
fn chunk(id: &[u8], body: &[u8]) -> Vec<u8> {
    let size = u32::try_from(body.len()).unwrap();
    let mut out = [id, &size.to_le_bytes(), body].concat();
    if body.len() % 2 == 1 {
        out.push(0);
    }
    out
}

/// A WAV file of `data_bytes` of silence, with `before` chunks ahead of its data.
fn wav(data_bytes: usize, before: &[Vec<u8>]) -> Vec<u8> {
    let rate = RATE * u32::from(BLOCK);
    let fmt = [
        &FLOAT.to_le_bytes()[..],
        &1_u16.to_le_bytes(),
        &RATE.to_le_bytes(),
        &rate.to_le_bytes(),
        &BLOCK.to_le_bytes(),
        &BITS.to_le_bytes(),
    ]
    .concat();
    let mut body = b"WAVE".to_vec();
    body.extend(chunk(b"fmt ", &fmt));
    for c in before {
        body.extend(c);
    }
    body.extend(chunk(b"data", &vec![0; data_bytes]));
    chunk(b"RIFF", &body)
}

#[test]
fn a_clips_length_is_its_data_over_its_byte_rate() {
    let second = usize::try_from(RATE).unwrap() * usize::from(BLOCK);
    assert_eq!(length_ms(&wav(second, &[])), Some(1_000));
    assert_eq!(length_ms(&wav(second / 4, &[])), Some(250));
    let odd = chunk(b"LIST", b"abc");
    assert_eq!(
        length_ms(&wav(second, &[odd])),
        Some(1_000),
        "a padded chunk"
    );
}

#[test]
fn anything_but_a_whole_wav_has_no_length() {
    let good = wav(64, &[]);
    assert_eq!(length_ms(b"not a wav file at all"), None);
    assert_eq!(length_ms(&good[..20]), None, "cut inside its format");
    let mut flat = good.clone();
    flat[28..32].copy_from_slice(&0_u32.to_le_bytes());
    assert_eq!(length_ms(&flat), None, "no byte rate");
    let mut riff = good;
    riff[8..12].copy_from_slice(b"AVI ");
    assert_eq!(length_ms(&riff), None, "another RIFF kind");
}

#[test]
fn each_shipped_clip_lasts_six_seconds_at_most() {
    let cfg = crate::config::load().expect("the settings load");
    let voice = cfg.dir.join(&cfg.g22_bench.voice_dir);
    for (lang, clip) in &cfg.g22_bench.clips {
        let bytes = std::fs::read(voice.join(&clip.file)).expect("the clip reads");
        let ms = length_ms(&bytes).expect("a WAV file");
        assert!((1..=6_000).contains(&ms), "{lang}: {ms} ms");
    }
}

#[test]
fn the_sound_file_follows_the_voice_jobs_own_words() {
    let launch = Launch {
        args: vec!["voice".into()],
        memory_mb: 7,
    };
    let extra = ["--sound".to_owned(), r"C:\clips\a.wav".to_owned()];
    let got = args(&launch, &extra);
    assert_eq!(got.args, ["voice", "--sound", r"C:\clips\a.wav"]);
    assert_eq!(got.memory_mb, 7, "the host's own cap");
}

#[test]
fn a_weak_workers_folder_holds_only_the_plain_parts() {
    let names = |list: &[&str]| list.iter().map(|n| (*n).to_owned()).collect::<Vec<_>>();
    let weak = names(&["part-x64.dll", "part-sse42.dll"]);
    let plain = names(&["keyxtend-worker.exe", "engine.dll", "PART-SSE42.DLL"]);
    assert_eq!(only(&plain, "part-", &weak, CARD), Ok(()));
    let fast = names(&["keyxtend-worker.exe", "part-x64.dll", "part-haswell.dll"]);
    assert!(
        only(&fast, "part-", &weak, CARD).is_err(),
        "a part with AVX"
    );
    let none = names(&["keyxtend-worker.exe", "engine.dll"]);
    assert!(only(&none, "part-", &weak, CARD).is_err(), "no part at all");
}

/// A card part's name, which no processor part shares.
const CARD: &str = "engine-card.dll";

#[test]
fn a_weak_workers_folder_made_from_the_card_build_leaves_the_card_part_out() {
    let names = |list: &[&str]| list.iter().map(|n| (*n).to_owned()).collect::<Vec<_>>();
    let weak = names(&["part-x64.dll"]);
    let carded = names(&["keyxtend-worker.exe", "part-x64.dll", "ENGINE-CARD.DLL"]);
    let e = only(&carded, "part-", &weak, CARD).expect_err("the card part");
    assert!(e.contains(CARD), "{e}");
    let left_out = names(&["keyxtend-worker.exe", "part-x64.dll"]);
    assert_eq!(only(&left_out, "part-", &weak, CARD), Ok(()));
}

#[test]
fn the_end_watch_waits_for_every_worker_it_saw_start() {
    let alive = Alive::default();
    assert!(alive.none_within(Duration::ZERO), "none started");
    alive.up();
    alive.up();
    alive.down();
    assert!(!alive.none_within(Duration::ZERO), "one still runs");
    alive.down();
    assert!(alive.none_within(Duration::ZERO));
}

/// The product's ask for a model file that only the fake reads.
fn ask() -> SpeechAsk {
    let model = SpeechModel {
        file: Redacted::new(PathBuf::from(r"C:\models\m.gguf")),
        lang: "en".into(),
        runner: "transcribe".into(),
        cpu_mb: 64,
    };
    VoiceSettings::from_defaults()
        .unwrap()
        .ask(Some(model), None)
}

/// The level notes a take of the fake hears before Stop.
const LEVELS: u32 = 3;

/// A take of the fake that hears `LEVELS` levels, and waits `within` for them and for its text.
fn script(within: Duration) -> Script<'static> {
    Script {
        levels: LEVELS,
        within,
        sentence: "open the file now",
        also: &[],
        lang: "en",
    }
}

fn piece(index: u16, text: &str) -> SpeechNote {
    let text = Redacted::new(Zeroizing::new(text.to_owned()));
    SpeechNote::Piece { index, text }
}

/// The microphone opening, then `levels` levels of sound.
fn opened(levels: u32) -> Vec<SpeechNote> {
    let sound = (0..levels).map(|_| SpeechNote::Level(1));
    [OPEN].into_iter().chain(sound).collect()
}

const OPEN: SpeechNote = SpeechNote::Stage(SpeechStage::Open);
const DONE: SpeechDone = SpeechDone {
    pieces: 2,
    sound_ms: 900,
    wait_ms: 120,
    load_ms: 450,
    memory_mb: 700,
    card: false,
};
const WAIT: Duration = Duration::from_secs(1);

#[test]
fn a_take_keeps_the_jobs_load_and_memory_times_its_own_wait_and_counts_the_words_wrong() {
    let fake = FakeSpeech::new();
    let loading = SpeechNote::Stage(SpeechStage::Loading);
    fake.on(SpeechCall::Open, [vec![loading], opened(LEVELS)].concat());
    let card = SpeechNote::Fault(SpeechError::NoCard);
    let notes = vec![
        card,
        piece(0, "open a file"),
        piece(1, "now"),
        SpeechNote::Done(DONE),
    ];
    fake.on(SpeechCall::Stop, notes);
    let got = take(&fake, ask(), &script(WAIT)).unwrap();
    assert_eq!((got.load_ms, got.memory_mb), (450, 700));
    assert!(
        got.open_ms <= 1_000,
        "the open, on the bench's own clock: {got:?}"
    );
    assert!(got.first_ms.is_some_and(|f| f <= got.total_ms), "{got:?}");
    assert!(got.total_ms <= 1_000, "the bench's own clock: {got:?}");
    assert_eq!(got.wrong, Some(Wrong { words: 1, total: 4 }));
    let runs = fake.runs();
    assert_eq!((runs[0].stops, runs[0].closed), (1, true));
}

#[test]
fn a_take_counts_the_words_wrong_against_the_closest_reading() {
    let also = ["open the dragonflies".to_owned()];
    for (heard, words) in [
        ("open the dragonflies", 0),
        ("open the dragon flies", 0),
        ("open dragonflies", 1),
    ] {
        let fake = FakeSpeech::new();
        fake.on(SpeechCall::Open, opened(LEVELS));
        fake.on(
            SpeechCall::Stop,
            vec![piece(0, heard), SpeechNote::Done(DONE)],
        );
        let script = Script {
            sentence: "open the dragon flies",
            also: &also,
            ..script(WAIT)
        };
        let got = take(&fake, ask(), &script).unwrap();
        assert_eq!(got.wrong.map(|w| w.words), Some(words), "{heard}");
    }
}

#[test]
fn a_take_stops_only_once_the_whole_clip_was_heard_and_never_a_run_that_ended() {
    let silent = FakeSpeech::new();
    let short = FakeSpeech::new();
    short.on(SpeechCall::Open, opened(LEVELS - 1));
    let early = FakeSpeech::new();
    early.on(
        SpeechCall::Open,
        [opened(LEVELS), vec![SpeechNote::Done(DONE)]].concat(),
    );
    for (fake, why) in [
        (silent, "no microphone"),
        (short, "the clip not all heard"),
        (early, "ended before Stop"),
    ] {
        assert!(
            take(&fake, ask(), &script(Duration::ZERO)).is_err(),
            "{why}"
        );
        let runs = fake.runs();
        assert_eq!((runs[0].stops, runs[0].closed), (0, true), "{why}");
    }
}

#[test]
fn a_fault_or_a_wipe_ends_the_take_and_closes_the_run() {
    for note in [SpeechNote::Fault(SpeechError::Memory), SpeechNote::Wiped] {
        let fake = FakeSpeech::new();
        fake.on(SpeechCall::Open, vec![OPEN, note.clone()]);
        let e = take(&fake, ask(), &script(WAIT)).unwrap_err();
        assert!(!e.is_empty(), "{note:?}");
        assert!(fake.runs()[0].closed, "{note:?}");
    }
}

#[test]
fn no_text_within_the_wait_ends_the_take() {
    let fake = FakeSpeech::new();
    fake.on(SpeechCall::Open, opened(LEVELS));
    fake.on(SpeechCall::Stop, vec![piece(0, "open the file now")]);
    assert!(take(&fake, ask(), &script(Duration::ZERO)).is_err());
    assert!(fake.runs()[0].closed);
}
