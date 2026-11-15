//! The reads: devices, a file's facts and a card's check answer as set, each ask checked by its own rule and
//! recorded, and a check's notes told before it answers.

use super::*;
use kx_module_api::ports::{DeviceKind, MAX_PERCENT, Memory};

fn card_id() -> CardId {
    CardId::new("Vulkan0").unwrap()
}

fn devices() -> Vec<Device> {
    let cpu = Device {
        kind: DeviceKind::Cpu,
        name: "Processor".into(),
        memory: Memory {
            total_mb: 16_000,
            free_mb: 9_000,
        },
        cores: 8,
        built_in: false,
        driver: String::new(),
    };
    let card = Device {
        kind: DeviceKind::Card(card_id()),
        cores: 0,
        built_in: true,
        driver: "31.0".into(),
        ..cpu.clone()
    };
    vec![cpu, card]
}

#[test]
fn the_clips_are_none_until_set_then_as_set_in_their_order() {
    let fake = FakeSpeech::new();
    assert!(fake.clips().is_empty(), "no clip shipped until set");
    fake.set_clips(&["fr", "en"]);
    assert_eq!(fake.clips(), ["fr", "en"]);
}

fn facts_ask(file: &str) -> FactsAsk {
    FactsAsk {
        job: job(),
        file: Redacted::new(PathBuf::from(file)),
        size_mb: 800,
    }
}

fn check_ask(id: CardId) -> CheckAsk {
    let card = CardUse {
        id,
        need_mb: 900,
        margin_mb: 512,
    };
    CheckAsk {
        job: job(),
        model: model(FILE),
        card,
        threads: 4,
        clip: "ar".into(),
    }
}

fn checked() -> CheckDone {
    CheckDone {
        card_ms: 200,
        cpu_ms: 1_800,
        card_text: secret(SAID),
        cpu_text: secret(SAID),
    }
}

#[test]
fn devices_answer_gone_until_set_then_as_set_each_time() {
    let fake = FakeSpeech::new();
    assert_eq!(fake.devices(&job()), Err(SpeechError::Gone));
    fake.set_devices(Ok(devices()));
    assert_eq!(fake.devices(&job()), Ok(devices()));
    assert_eq!(fake.devices(&job()), Ok(devices()), "it repeats");
    fake.set_devices(Err(SpeechError::Timeout));
    assert_eq!(fake.devices(&job()), Err(SpeechError::Timeout));
    assert_eq!(fake.device_asks(), vec![job(); 4]);
}

#[test]
fn a_bad_job_is_refused_by_its_own_rule_and_not_recorded() {
    let fake = FakeSpeech::new();
    fake.set_devices(Ok(devices()));
    let tight = JobLimits { job_mb: 1, ..job() };
    assert_eq!(fake.devices(&tight), Err(SpeechError::Range("job_mb")));
    assert!(fake.device_asks().is_empty());
}

#[test]
fn facts_answer_per_file_and_an_unknown_file_is_missing() {
    let fake = FakeSpeech::new();
    let facts = ModelFacts {
        runner: "whisper".into(),
        langs: vec!["ar".into(), "fr".into()],
    };
    fake.set_facts(Path::new(FILE), Ok(facts.clone()));
    fake.set_facts(Path::new(OTHER), Err(SpeechError::Runner));
    assert_eq!(fake.facts(&facts_ask(FILE)), Ok(facts));
    assert_eq!(fake.facts(&facts_ask(OTHER)), Err(SpeechError::Runner));
    let unknown = facts_ask(r"D:\Models\none.gguf");
    assert_eq!(fake.facts(&unknown), Err(SpeechError::NoFile));
    assert_eq!(
        fake.facts(&facts_ask("none.gguf")),
        Err(SpeechError::BadPath)
    );
    assert_eq!(
        fake.facts_asks(),
        [facts_ask(FILE), facts_ask(OTHER), unknown]
    );
}

#[test]
fn a_check_tells_its_notes_in_order_then_answers_as_set_for_its_card() {
    let fake = FakeSpeech::new();
    let warm = [0, MAX_PERCENT].map(|percent| SpeechNote::Warming { percent });
    let told = [&[SpeechNote::Stage(SpeechStage::Loading)][..], &warm].concat();
    fake.set_check(&card_id(), told.clone(), Ok(checked()));
    let (heard, tell) = hearer();
    assert_eq!(fake.check(check_ask(card_id()), tell), Ok(checked()));
    assert_eq!(notes(&heard), told);
    let me = thread::current().id();
    assert!(heard.lock().unwrap().iter().all(|(_, t)| *t == me));
    assert_eq!(fake.check_asks(), [check_ask(card_id())]);
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "never returns")]
fn a_read_answer_the_port_never_returns_is_a_test_mistake() {
    FakeSpeech::new().set_devices(Err(SpeechError::NoMic));
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "never returns")]
fn a_check_answer_the_port_never_returns_is_a_test_mistake() {
    FakeSpeech::new().set_check(&card_id(), Vec::new(), Err(SpeechError::Language));
}

#[test]
fn an_unset_card_is_gone_and_a_bad_check_is_refused() {
    let fake = FakeSpeech::new();
    fake.set_check(
        &card_id(),
        vec![SpeechNote::Heard],
        Err(SpeechError::CardRoom),
    );
    let (heard, tell) = hearer();
    let other = CardId::new("Vulkan1").unwrap();
    assert_eq!(fake.check(check_ask(other), tell), Err(SpeechError::NoCard));
    assert!(notes(&heard).is_empty(), "an unset card tells nothing");
    let odd = CheckAsk {
        clip: "arabic".into(),
        ..check_ask(card_id())
    };
    let refused = fake.check(odd, Box::new(|_| {}));
    assert_eq!(refused, Err(SpeechError::Range("clip")));
    let room = fake.check(check_ask(card_id()), Box::new(|_| {}));
    assert_eq!(room, Err(SpeechError::CardRoom));
    assert_eq!(
        fake.check_asks().len(),
        2,
        "the refused ask is not recorded"
    );
}
