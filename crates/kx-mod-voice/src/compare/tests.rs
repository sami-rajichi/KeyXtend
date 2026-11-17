//! Compare models (plan task 19): one recording, then each ready model in a fresh job on it; numbers kept, never texts;
//! no set text gives no count; one model's fault does not stop the others.

use super::*;
use crate::flow::Ready;
use kx_module_api::Redacted;
use kx_module_api::ports::{
    CardId, CardUse, Date, Fingerprint, RunStep, ServiceName, SpeechError, SpeechModel, SpeechStage,
};
use kx_module_api::{Secret, Zeroizing};
use std::path::PathBuf;

mod cloud;
mod ends;
mod props;

/// The set text the tests read, nine words.
const SET: &str = "The evening train leaves at a quarter past seven.";

fn settings() -> VoiceSettings {
    VoiceSettings::from_defaults().unwrap()
}

fn secret(text: &str) -> Secret {
    Redacted::new(Zeroizing::new(text.to_owned()))
}

fn print(n: u8) -> Fingerprint {
    Fingerprint::new([n; Fingerprint::BYTES])
}

/// Model `n`, on card `Vulkan0` when `card`.
fn entrant(n: u8, card: bool) -> Entrant {
    let model = SpeechModel {
        file: Redacted::new(PathBuf::from(format!(r"D:\models\m{n}.gguf"))),
        lang: "en".into(),
        runner: "transcribe".into(),
        cpu_mb: 900,
    };
    let card = card.then(|| CardUse {
        id: CardId::new("Vulkan0").unwrap(),
        need_mb: 900,
        margin_mb: 512,
    });
    let ready = Ready {
        model,
        card,
        name: Redacted::new(format!("m{n}")),
        print: print(n),
        bytes: 1,
    };
    Entrant {
        name: Redacted::new(format!("Model {n}")),
        ready,
    }
}

/// Cloud model `n` of Groq.
fn remote(n: u8) -> Remote {
    Remote {
        name: Redacted::new(format!("Cloud {n}")),
        shown: ServiceName::Named("Groq".into()),
        service: "groq".into(),
        model: Redacted::new(format!("m{n}")),
    }
}

/// A run of `models` over `set` in English.
fn plan(set: &str, models: Vec<Entrant>) -> Plan {
    Plan {
        lang: "en".into(),
        set: Redacted::new(set.to_owned()),
        entrants: models,
        clouds: Vec::new(),
        day: Date::new(2026, 11, 14).unwrap(),
        auto: true,
    }
}

/// A model's end, `wait_ms` after Stop, at `mb`.
fn done(wait_ms: u32, mb: u32) -> SpeechNote {
    SpeechNote::Done(SpeechDone {
        pieces: 1,
        sound_ms: 4_000,
        wait_ms,
        load_ms: 0,
        memory_mb: mb,
        card: false,
    })
}

fn piece(text: &str) -> SpeechNote {
    SpeechNote::Piece {
        index: 0,
        text: secret(text),
    }
}

/// The ask each `Again` carries, in order.
fn agains(acts: &[Act]) -> Vec<SpeechAsk> {
    let again = |a: &Act| match a {
        Act::Run {
            call: RunCall::Again(ask),
            ..
        } => Some(ask.clone()),
        _ => None,
    };
    acts.iter().filter_map(again).collect()
}

/// A compare that recorded speech and stopped, with its run's id.
fn recorded(models: Vec<Entrant>, set: &str) -> (Compare, u32) {
    let mut compare = Compare::default();
    let acts = compare.start(plan(set, models), &settings());
    let Some(Act::Open { take, ask }) = acts.first() else {
        panic!("no open: {acts:?}");
    };
    assert_eq!(ask.model, None, "the recording's job loads no model");
    let take = *take;
    compare.note(take, SpeechNote::Heard);
    assert_eq!(
        compare.stop(),
        [Act::Run {
            take,
            call: RunCall::Stop
        }]
    );
    (compare, take)
}

/// Models 1 and 2 run to the end, model 2 missing two words: the asks of their turns, then the last effects.
fn run_two() -> (Compare, Vec<SpeechAsk>, Vec<Act>) {
    let (mut compare, take) = recorded(vec![entrant(1, false), entrant(2, false)], SET);
    let mut asks = agains(&compare.note(take, done(0, 0)));
    compare.note(take, piece(SET));
    asks.extend(agains(&compare.note(take, done(1_000, 800))));
    compare.note(
        take,
        piece("The evening rain leaves at quarter past seven."),
    );
    let last = compare.note(take, done(2_000, 700));
    (compare, asks, last)
}

#[test]
fn it_records_once_then_runs_each_ready_model_in_a_fresh_job() {
    let (_, asks, last) = run_two();
    let models: Vec<_> = asks.into_iter().map(|a| a.model).collect();
    let want = [entrant(1, false), entrant(2, false)].map(|e| Some(e.ready.model));
    assert_eq!(models, want);
    let close = |a: &Act| {
        matches!(
            a,
            Act::Run {
                call: RunCall::Close,
                ..
            }
        )
    };
    assert!(last.iter().any(close));
}

#[test]
fn the_numbers_are_kept_never_the_texts() {
    let (compare, _, last) = run_two();
    let Some(Act::Save(board)) = last.iter().find(|a| matches!(a, Act::Save(_))) else {
        panic!("no save: {last:?}");
    };
    let numbers: Vec<_> = board
        .scores
        .iter()
        .map(|s| (s.model, s.wrong, s.words, s.wait_ms, s.mb))
        .collect();
    assert_eq!(
        numbers,
        [
            (print(1), Some(0), 9, 1_000, 800),
            (print(2), Some(2), 9, 2_000, 700)
        ]
    );
    assert_eq!(
        (board.lang.as_str(), board.auto, board.card.clone()),
        ("en", true, None)
    );
    assert!(
        compare.results().iter().all(|l| l.text.is_none()),
        "no text is kept"
    );
}

#[test]
fn no_set_text_gives_no_count_and_shows_each_text() {
    let (mut compare, take) = recorded(vec![entrant(1, false)], "");
    compare.note(take, done(0, 0));
    compare.note(take, piece("Guten Abend"));
    let last = compare.note(take, done(500, 600));
    let Some(Act::Save(board)) = last.iter().find(|a| matches!(a, Act::Save(_))) else {
        panic!("no save");
    };
    assert_eq!((board.scores[0].wrong, board.scores[0].words), (None, 0));
    let shown = compare.results()[0]
        .text
        .as_ref()
        .map(|t| t.expose().to_string());
    assert_eq!(shown.as_deref(), Some("Guten Abend"));
}

#[test]
fn one_models_fault_does_not_stop_the_others() {
    let (mut compare, take) = recorded(vec![entrant(1, false), entrant(2, false)], SET);
    compare.note(take, done(0, 0));
    let next = compare.note(take, SpeechNote::Fault(SpeechError::BadFile));
    assert_eq!(agains(&next)[0].model, Some(entrant(2, false).ready.model));
    let (at, runs) = compare.running().unwrap();
    let steps: Vec<RunStep> = runs.iter().map(|(_, step)| *step).collect();
    assert_eq!(at, Some(1));
    assert_eq!(
        steps,
        [RunStep::Failed, RunStep::Writing],
        "it stays listed"
    );
    compare.note(take, piece(SET));
    compare.note(take, done(900, 800));
    let names: Vec<String> = compare
        .results()
        .iter()
        .map(|l| l.name.expose().clone())
        .collect();
    assert_eq!(names, ["Model 2"]);
}
