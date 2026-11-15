//! The asks in a job of their own: the devices with each card's driver from the system, so a driver update can run
//! the check again (edge 26); a file's facts; and a card's check beside a live dictation, its warm-up told.

use super::*;
use crate::worker::fake::voice::{card_failed, open as opened};
use kx_ipc::{Device as Found, DeviceKind as Kind, Target};
use kx_module_api::ports::{CardId, CardUse, CheckAsk, DeviceKind, FactsAsk, Memory, ModelFacts};

/// The driver the engine reports, which the system's own replaces when it has one.
const ENGINE: &str = "engine-1";
/// A card built into the processor, as the engine lists it.
const BUILT_IN: &str = "Vulkan1";
/// A test device's memory, in MB.
const TOTAL_MB: u32 = 4_096;
const FREE_MB: u32 = 2_048;
/// The test clip's words, and the card's and processor's times on it, in ms.
const WORDS: &str = "hello";
const CARD_MS: u32 = 900;
const CPU_MS: u32 = 9_000;

/// A device the engine found.
fn found(kind: Kind, id: &str, name: &str) -> Found {
    Found {
        kind,
        id: id.to_owned(),
        name: name.to_owned(),
        total_mb: TOTAL_MB,
        free_mb: FREE_MB,
        driver: ENGINE.to_owned(),
    }
}

/// The processor, a card the system names, and a built-in card it does not.
fn three(id: u32) -> VoiceNote {
    let list = vec![
        found(Kind::Cpu, "cpu", "Processor"),
        found(Kind::Card, CARD, CARD_NAME),
        found(Kind::BuiltIn, BUILT_IN, "Built-in"),
    ];
    VoiceNote::Devices { id, list }
}

/// The test model's facts.
fn facts(id: u32) -> VoiceNote {
    let (runner, langs) = (RUNNER.to_owned(), vec![LANG.to_owned()]);
    VoiceNote::Facts { id, runner, langs }
}

/// A check's answer.
fn checked(id: u32) -> VoiceNote {
    VoiceNote::Checked {
        id,
        card_ms: CARD_MS,
        cpu_ms: CPU_MS,
        card_text: Zeroizing::new(WORDS.to_owned()),
        cpu_text: Zeroizing::new(WORDS.to_owned()),
    }
}

/// The card warmed halfway.
fn warm_half(id: u32) -> VoiceNote {
    VoiceNote::Warm { id, percent: 50 }
}

/// The model loads.
fn loading(id: u32) -> VoiceNote {
    stage(id, Stage::Loading)
}

/// The test card's id.
fn card(id: &str) -> CardId {
    CardId::new(id).unwrap()
}

/// A check of the test card on `FILE_A`'s model with the English clip.
fn check_ask() -> CheckAsk {
    let card = CardUse {
        id: card(CARD),
        need_mb: 900,
        margin_mb: 512,
    };
    CheckAsk {
        job: limits(),
        model: model(FILE_A),
        card,
        threads: 0,
        clip: "en".to_owned(),
    }
}

#[test]
fn devices_run_in_a_job_of_their_own_each_card_with_the_systems_driver() {
    let (script, _, speech) = solo();
    script.act(VoiceAct::Say(vec![three]));
    let seen = calls(&script);
    let list = speech.devices(&limits()).unwrap();
    let kinds: Vec<DeviceKind> = list.iter().map(|d| d.kind.clone()).collect();
    let cards = [
        DeviceKind::Card(card(CARD)),
        DeviceKind::Card(card(BUILT_IN)),
    ];
    assert_eq!(kinds[0], DeviceKind::Cpu);
    assert_eq!(kinds[1..], cards, "built-in graphics is picked as a card");
    let drivers: Vec<&str> = list.iter().map(|d| d.driver.as_str()).collect();
    assert_eq!(drivers, ["", DRIVER, ENGINE], "the system's driver first");
    let memory = Memory {
        total_mb: TOTAL_MB,
        free_mb: FREE_MB,
    };
    assert_eq!(list[1].memory, memory);
    assert_eq!(script.launches(), [job(limits().job_mb)], "job_mb alone");
    assert!(matches!(asks(&script)[..], [VoiceAsk::Devices { .. }]));
    assert!(said_bye(&seen), "it ends once it answered");
}

#[test]
fn only_built_in_graphics_has_the_built_in_mark_and_only_the_processor_its_cores() {
    let (script, _, speech) = solo();
    script.act(VoiceAct::Say(vec![three]));
    let list = speech.devices(&limits()).unwrap();
    let marks: Vec<(u16, bool)> = list.iter().map(|d| (d.cores, d.built_in)).collect();
    let cores = u16::try_from(CORES).unwrap();
    assert_eq!(marks, [(cores, false), (0, false), (0, true)]);
}

#[test]
fn the_processors_memory_is_the_systems_free_and_total_when_it_tells_them() {
    let (script, _, speech) = solo();
    script.act(VoiceAct::Say(vec![three]));
    let list = speech.devices(&limits()).unwrap();
    assert_eq!(list[0].memory, SYSTEM, "the system's own reading");
    let engine = Memory {
        total_mb: TOTAL_MB,
        free_mb: FREE_MB,
    };
    assert_eq!(list[2].memory, engine, "a card keeps the engine's");
    let script = Script::default();
    let turns = Turns(Arc::new((vec![script.clone()], AtomicUsize::new(0))));
    let clock = Arc::new(Hands::default());
    let quiet = WorkerSpeech::new(turns, clock, assets(), Arc::new(Quiet));
    script.act(VoiceAct::Say(vec![three]));
    let list = quiet.devices(&limits()).unwrap();
    assert_eq!(
        list[0].memory, engine,
        "the engine's when the system is quiet"
    );
}

#[test]
fn the_clips_are_the_staged_ones_in_their_order() {
    let (_, _, speech) = solo();
    assert_eq!(speech.clips(), ["en", "de"], "as staged, never sorted");
}

#[test]
fn a_files_facts_are_read_in_a_job_capped_at_its_size_and_job_mb() {
    let (script, _, speech) = solo();
    script.act(VoiceAct::Say(vec![facts]));
    let ask = FactsAsk {
        job: limits(),
        file: Redacted::new(PathBuf::from(FILE_A)),
        size_mb: MODEL_MB,
    };
    let read = speech.facts(&ask).unwrap();
    let want = ModelFacts {
        runner: RUNNER.to_owned(),
        langs: vec![LANG.to_owned()],
    };
    assert_eq!(read, want);
    assert_eq!(script.launches(), [job(limits().job_mb + MODEL_MB)]);
    assert!(matches!(&asks(&script)[..], [VoiceAsk::Facts { path, .. }] if path == FILE_A));
}

#[test]
fn a_cards_check_runs_in_its_own_job_beside_a_live_dictation_and_tells_its_warm_up() {
    let (dictation, checking) = (Script::default(), Script::default());
    let clock = Hands::default();
    let speech = host(&[dictation.clone(), checking.clone()], &clock);
    let (tell, heard) = ear();
    dictation.act(VoiceAct::Say(vec![opened]));
    let _run = speech.open(ask(Some(FILE_A)), tell).unwrap();
    next(&heard, 1);
    checking.act(VoiceAct::Say(vec![loading, warm_half, checked]));
    let seen = calls(&checking);
    let (side, told) = ear();
    let done = speech.check(check_ask(), side).unwrap();
    assert_eq!((done.card_ms, done.cpu_ms), (CARD_MS, CPU_MS));
    assert_eq!(done.card_text.expose().as_str(), WORDS);
    let warming = SpeechNote::Warming { percent: 50 };
    assert_eq!(next(&told, 2), [SpeechNote::Stage(Stage::Loading), warming]);
    assert_eq!(checking.launches(), [job(limits().job_mb + MODEL_MB)]);
    let VoiceAsk::Check { model, clip, .. } = &asks(&checking)[0] else {
        panic!("a check");
    };
    assert_eq!(clip, CLIP, "the shipped clip for its language");
    let target = Target::Card {
        id: CARD.into(),
        need_mb: 1_412,
    };
    assert_eq!(
        (&model.target, usize::from(model.threads)),
        (&target, CORES)
    );
    assert!(
        !dictation.calls().contains(&Call::Bye),
        "the dictation kept its job"
    );
    assert!(said_bye(&seen), "the check's job ended");
}

#[test]
fn a_check_the_card_fails_answers_its_fault() {
    let (script, _, speech) = solo();
    script.act(VoiceAct::Say(vec![loading, card_failed]));
    let failed = speech.check(check_ask(), ear().0);
    assert_eq!(failed, Err(SpeechError::CardFailed));
}

#[test]
fn a_check_with_no_clip_for_its_language_is_refused_before_any_job() {
    let (script, _, speech) = solo();
    let mut ask = check_ask();
    ask.clip = "ar".to_owned();
    assert_eq!(speech.check(ask, ear().0), Err(SpeechError::NoFile));
    assert_eq!(script.tries(), 0);
}
