//! Compare through the panel: the ready models of the language in front; Start records once, the bar's Stop runs
//! each model in turn; the results tick to use one; nothing starts while a dictation runs.

use kx_module_api::Redacted;
use kx_module_api::ports::{
    BarButton, CompareSheet, DeviceLabel, DeviceName, PanelButton, RunStep, SpeechDone, SpeechNote,
    VoiceClick, VoiceSheet, VoiceTab,
};
use kx_module_api::{Secret, Zeroizing};

use super::super::rig::{Rig, row};
use super::Do;
use crate::compare::Act;
use crate::flow::RunCall;

/// The set text, nine words.
const SET: &str = "The evening train leaves at a quarter past seven.";

/// The Compare page as shown.
fn sheet(rig: &mut Rig) -> CompareSheet {
    match rig.view().unwrap().sheet {
        VoiceSheet::Compare(sheet) => sheet,
        other => panic!("not Compare: {other:?}"),
    }
}

/// The rig on the Compare page with Parakeet and Whisper Small in the folder.
fn on_compare() -> Rig {
    let mut rig = Rig::foldered();
    rig.holds(&[row("parakeet-v3-q8"), row("whisper-small-q5km")]);
    rig.click(&VoiceClick::Open(VoiceTab::Compare));
    rig
}

fn set() -> Redacted<String> {
    Redacted::new(SET.to_owned())
}

fn done(wait_ms: u32) -> SpeechNote {
    SpeechNote::Done(SpeechDone {
        pieces: 1,
        sound_ms: 4_000,
        wait_ms,
        load_ms: 0,
        memory_mb: 800,
        card: false,
    })
}

fn piece(text: &str) -> SpeechNote {
    let text: Secret = Redacted::new(Zeroizing::new(text.to_owned()));
    SpeechNote::Piece { index: 0, text }
}

/// Starts the run and stops the recording once speech was heard, giving its id.
fn recorded(rig: &mut Rig) -> u32 {
    let out = rig.click(&VoiceClick::Compare(set()));
    let [Do::Compare(Act::Open { take, ask })] = &out[..] else {
        panic!("{out:?}");
    };
    assert_eq!(ask.model, None, "the recording loads no model");
    let take = *take;
    rig.panel.compare_note(take, SpeechNote::Heard);
    let stop = rig.click(&VoiceClick::Bar(BarButton::Stop));
    assert_eq!(
        stop,
        [Do::Compare(Act::Run {
            take,
            call: RunCall::Stop
        })]
    );
    take
}

#[test]
fn the_compare_page_names_the_ready_models_and_where_they_run() {
    let mut rig = on_compare();
    let want = CompareSheet::Ready {
        models: 2,
        device: DeviceLabel::Auto(DeviceName::Cpu),
        recording: false,
    };
    assert_eq!(sheet(&mut rig), want);
    rig.click(&VoiceClick::Compare(set()));
    assert!(matches!(
        sheet(&mut rig),
        CompareSheet::Ready {
            recording: true,
            ..
        }
    ));
    let acts = rig.view().unwrap().acts;
    assert_eq!(
        acts,
        [PanelButton::Back, PanelButton::Close, PanelButton::Cancel]
    );
}

#[test]
fn each_model_writes_in_turn_then_one_result_is_used() {
    let mut rig = on_compare();
    let take = recorded(&mut rig);
    let first = rig.panel.compare_note(take, done(0));
    assert!(matches!(
        &first[..],
        [Do::Compare(Act::Run {
            call: RunCall::Again(_),
            ..
        })]
    ));
    let CompareSheet::Running { at, of, rows, .. } = sheet(&mut rig) else {
        panic!("not running");
    };
    assert_eq!(
        (at, of, rows[0].step, rows[1].step),
        (1, 2, RunStep::Writing, RunStep::Waiting)
    );
    rig.panel.compare_note(take, piece(SET));
    rig.panel.compare_note(take, done(1_000));
    rig.panel.compare_note(
        take,
        piece("The evening rain leaves at quarter past seven."),
    );
    let last = rig.panel.compare_note(take, done(2_000));
    assert!(last.iter().any(|d| matches!(d, Do::Compare(Act::Save(_)))));
    used_second(&mut rig);
}

/// The results show numbers, never texts, and Use on the second writes its pick for English.
fn used_second(rig: &mut Rig) {
    let small = row("whisper-small-q5km");
    let CompareSheet::Results { rows, .. } = sheet(rig) else {
        panic!("no results");
    };
    let wrong: Vec<_> = rows.iter().map(|r| (r.wrong, r.text.is_none())).collect();
    assert_eq!(
        wrong,
        [(Some(0), true), (Some(2), true)],
        "numbers, never texts"
    );
    rig.click(&VoiceClick::Row(1));
    let out = rig.click(&VoiceClick::Button(PanelButton::Use));
    let Some(Do::Set(_, picks)) = out.first() else {
        panic!("{out:?}");
    };
    assert_eq!(
        picks.get("en").and_then(|v| v.as_str()),
        Some(small.id.as_str())
    );
}

#[test]
fn back_while_recording_cancels_the_run() {
    let mut rig = on_compare();
    rig.click(&VoiceClick::Compare(set()));
    let out = rig.click(&VoiceClick::Button(PanelButton::Back));
    assert!(out.iter().any(|d| matches!(
        d,
        Do::Compare(Act::Run {
            call: RunCall::Cancel,
            ..
        })
    )));
    assert!(!rig.panel.compare().busy());
}

#[test]
fn nothing_starts_while_a_dictation_runs() {
    let mut rig = on_compare();
    rig.dictating = true;
    assert!(rig.click(&VoiceClick::Compare(set())).is_empty());
}

#[test]
fn nothing_heard_shows_its_note() {
    let mut rig = on_compare();
    rig.click(&VoiceClick::Compare(set()));
    rig.click(&VoiceClick::Bar(BarButton::Stop));
    assert_eq!(
        rig.view().unwrap().note,
        Some(kx_module_api::ports::BarHint::Nothing)
    );
}
