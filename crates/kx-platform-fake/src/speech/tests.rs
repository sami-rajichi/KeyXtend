//! The fake speech, held to the port: asks checked by their own rules and recorded, notes told on cue in order, a
//! death, failures once per call, the held recording, and clones that serve another thread. Reads are in `info`.

use super::*;
use kx_module_api::Secret;
use kx_module_api::ports::{CardId, CardUse, LEVEL_FULL, SpeechDone, SpeechModel, SpeechStage};
use kx_test_support::paths::native_path;
use std::thread::{self, ThreadId};

mod info;
mod life;
mod pieces;

/// How long a test waits for another thread; it never sleeps.
const WITHIN: Duration = Duration::from_secs(10);
/// The model a run is opened with, written for Windows; `model` reads it through `native_path`.
const FILE: &str = r"D:\Models\parakeet.gguf";
/// The model Try again takes.
const OTHER: &str = r"D:\Models\cohere.gguf";
/// A transcript no output may show.
const SAID: &str = "bonjour-7731";
/// Sound bytes no output may show.
const SOUND: [u8; 4] = [0x7f, 0x3e, 0x11, 0x5a];

/// Test values; the real ones come from the voice module's settings.
fn job() -> JobLimits {
    JobLimits {
        job_mb: 512,
        idle_min: 5,
        ask_ms: 3_000,
        load_s: 60,
        warm_s: 180,
        piece_wait_s: 120,
        restart_ms: 1_000,
        restart_max_ms: 30_000,
    }
}

fn model(file: &str) -> SpeechModel {
    SpeechModel {
        file: Redacted::new(native_path(file)),
        lang: "fr".into(),
        runner: "transducer".into(),
        cpu_mb: 1_200,
    }
}

/// A dictation on the processor with the model at `FILE`.
fn ask() -> SpeechAsk {
    SpeechAsk {
        job: job(),
        model: Some(model(FILE)),
        card: None,
        threads: 0,
        piece_s: 30,
        pad_ms: 300,
        level_hz: 30,
        record_max_s: 600,
        silence_stop_s: None,
    }
}

/// `ask` with the model at `file`.
fn ask_with(file: &str) -> SpeechAsk {
    let model = Some(model(file));
    SpeechAsk { model, ..ask() }
}

fn secret(text: &str) -> Secret {
    Redacted::new(Zeroizing::new(text.to_owned()))
}

fn piece(index: u16) -> SpeechNote {
    let text = secret(SAID);
    SpeechNote::Piece { index, text }
}

fn done(pieces: u16) -> SpeechNote {
    SpeechNote::Done(SpeechDone {
        pieces,
        sound_ms: 4_000,
        wait_ms: 300,
        load_ms: 0,
        memory_mb: 1_500,
        card: false,
    })
}

/// What a test run heard, with the thread it heard it on.
type Heard = Arc<Mutex<Vec<(SpeechNote, ThreadId)>>>;

/// A `tell` that records each note into the returned list.
fn hearer() -> (Heard, Tell) {
    let heard = Heard::default();
    let log = Arc::clone(&heard);
    let tell = move |note| log.lock().unwrap().push((note, thread::current().id()));
    (heard, Box::new(tell))
}

/// A run opened with `ask` whose notes are recorded.
fn opened(fake: &FakeSpeech) -> (Box<dyn SpeechRun>, Heard) {
    let (heard, tell) = hearer();
    (fake.open(ask(), tell).ok().unwrap(), heard)
}

/// The notes `heard` holds, in order.
fn notes(heard: &Heard) -> Vec<SpeechNote> {
    heard
        .lock()
        .unwrap()
        .iter()
        .map(|(n, _)| n.clone())
        .collect()
}

#[test]
fn open_checks_the_ask_by_its_own_rule_and_records_it() {
    let fake = FakeSpeech::new();
    let silent = SpeechAsk {
        level_hz: 0,
        ..ask()
    };
    let refused = fake.open(silent, Box::new(|_| {})).err();
    assert_eq!(refused, Some(SpeechError::Range("level_hz")));
    assert!(fake.runs().is_empty(), "a refused ask opens no run");
    let (_run, _heard) = opened(&fake);
    let runs = fake.runs();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].ask, ask());
    assert!(runs[0].held && !runs[0].closed && !runs[0].dead);
}

#[test]
fn a_listen_only_ask_opens_and_a_lone_card_is_refused() {
    let fake = FakeSpeech::new();
    let listen = SpeechAsk {
        model: None,
        ..ask()
    };
    assert!(fake.open(listen.clone(), Box::new(|_| {})).is_ok());
    let card = CardUse {
        id: CardId::new("Vulkan0").unwrap(),
        need_mb: 900,
        margin_mb: 512,
    };
    let lone = SpeechAsk {
        card: Some(card),
        ..listen
    };
    let refused = fake.open(lone, Box::new(|_| {})).err();
    assert_eq!(refused, Some(SpeechError::Range("card")));
}

#[test]
fn levels_pieces_and_the_end_are_told_in_order_on_the_tests_thread() {
    let fake = FakeSpeech::new();
    let start = [SpeechStage::Loading, SpeechStage::Open].map(SpeechNote::Stage);
    fake.on(SpeechCall::Open, start.to_vec());
    let end = vec![SpeechNote::Stage(SpeechStage::Shut), piece(1), done(2)];
    fake.on(SpeechCall::Stop, end.clone());
    let (mut run, heard) = opened(&fake);
    assert_eq!(notes(&heard), start);
    let said = [SpeechNote::Level(9), SpeechNote::Heard, piece(0)];
    let quiet = [SpeechNote::Level(LEVEL_FULL), SpeechNote::Quiet];
    for note in said.iter().chain(&quiet) {
        assert!(fake.tell(note.clone()), "{note:?}");
    }
    assert_eq!(run.stop(), Ok(()));
    let all = [&start[..], &said, &quiet, &end].concat();
    assert_eq!(notes(&heard), all);
    let me = thread::current().id();
    assert!(heard.lock().unwrap().iter().all(|(_, t)| *t == me));
    assert_eq!(fake.runs()[0].stops, 1);
}

#[test]
fn each_cue_tells_once_in_turn_then_nothing() {
    let fake = FakeSpeech::new();
    let (first, second) = (vec![SpeechNote::Level(1)], vec![SpeechNote::Level(2)]);
    fake.on(SpeechCall::Open, first.clone());
    fake.on(SpeechCall::Open, second.clone());
    let heard: Vec<_> = (0..3).map(|_| notes(&opened(&fake).1)).collect();
    assert_eq!(heard, [first, second, Vec::new()]);
}

#[test]
fn notes_go_to_the_newest_live_run_and_none_after_close() {
    let fake = FakeSpeech::new();
    assert!(!fake.tell(SpeechNote::Heard), "no run hears it");
    let (mut old, old_heard) = opened(&fake);
    let (mut new, new_heard) = opened(&fake);
    assert!(fake.tell(SpeechNote::Heard));
    new.close();
    assert!(fake.tell(SpeechNote::Quiet), "the older run is live again");
    old.close();
    assert!(!fake.tell(SpeechNote::Level(3)));
    assert_eq!(notes(&new_heard), [SpeechNote::Heard]);
    assert_eq!(notes(&old_heard), [SpeechNote::Quiet]);
}

#[test]
fn clones_share_the_runs_and_a_waiter_wakes_for_an_open_on_another_thread() {
    let fake = FakeSpeech::new();
    let port: Arc<dyn Speech> = Arc::new(fake.clone());
    let (heard, tell) = hearer();
    let other = thread::spawn(move || port.open(ask(), tell).ok());
    assert!(fake.wait(WITHIN, |runs| runs.len() == 1));
    let run = other.join().unwrap();
    assert!(run.is_some());
    assert!(fake.tell(SpeechNote::Heard));
    assert_eq!(notes(&heard), [SpeechNote::Heard]);
    assert!(!fake.wait(Duration::ZERO, |runs| runs.len() == 2));
}

#[test]
fn debug_never_prints_text_or_sound() {
    let fake = FakeSpeech::new();
    fake.set_sound(vec![SOUND.to_vec()]);
    fake.on(SpeechCall::Stop, vec![piece(0)]);
    let (_run, _heard) = opened(&fake);
    let shown = format!("{fake:?} {:?}", fake.runs());
    assert!(
        !shown.contains(SAID) && !shown.contains("parakeet"),
        "{shown}"
    );
    let bytes = format!("{:?}", SOUND.to_vec());
    assert!(!shown.contains(&bytes[1..bytes.len() - 1]), "{shown}");
    assert!(shown.contains("runs: 1"), "{shown}");
}
