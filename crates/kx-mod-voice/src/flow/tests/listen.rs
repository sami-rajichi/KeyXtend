//! Listening: nothing heard (edge case 3), and the limits that stop it, `record_max_s` and `silence_stop` (4).

use super::*;
use kx_module_api::ports::BarText;

/// `nothing_s` in the defaults, in ms.
fn nothing_ms() -> u64 {
    let settings = VoiceSettings::from_defaults().unwrap();
    u64::from(settings.nothing_s) * 1_000
}

/// The Listening bar's hint.
fn hinting(rig: &Rig) -> bool {
    match rig.shown() {
        VoiceBar::Listening(shown) => shown.nothing,
        other => panic!("no Listening bar: {other:?}"),
    }
}

#[test]
fn stop_with_no_speech_types_nothing_and_offers_listen_again() {
    let mut rig = Rig::new();
    rig.key();
    rig.tell(SpeechNote::Stage(SpeechStage::Open));
    rig.wait(nothing_ms());
    let out = rig.key();
    assert_eq!(calls(&out), [RunCall::Cancel, RunCall::Close]);
    assert!(ask(&out).is_none() && kept(&out).is_empty());
    assert!(!rig.lit);
    let want = Some((BarHint::Nothing, Some(BarButton::Listen)));
    assert_eq!(note(&rig.shown()), want);
    let out = rig.bar(BarButton::Listen);
    assert_eq!(opened(&out).len(), 1);
    assert!(matches!(rig.shown(), VoiceBar::Listening(_)));
}

#[test]
fn listen_again_acts_only_where_the_note_offers_it() {
    let mut rig = Rig::new();
    rig.key();
    rig.speak();
    rig.bar(BarButton::Cancel);
    assert!(opened(&rig.bar(BarButton::Listen)).is_empty());
    assert_eq!(note(&rig.shown()).map(|n| n.0), Some(BarHint::Cancelled));
}

/// A fresh job's start before the microphone opens, in ms, shorter than `nothing_s`.
const START_MS: u64 = 2_000;

#[test]
fn listening_counts_from_the_microphone_opening_not_the_click() {
    let mut rig = Rig::new();
    rig.key();
    rig.wait(START_MS);
    rig.tell(SpeechNote::Stage(SpeechStage::Open));
    let due = Mono::from_us((START_MS + nothing_ms()) * 1_000);
    assert_eq!(rig.flow.deadline(), Some(due), "the hint after the opening");
    assert!(matches!(rig.shown(), VoiceBar::Listening(l) if l.elapsed_ms == 0));
}

#[test]
fn after_nothing_s_the_bar_hints_and_keeps_listening() {
    let mut rig = Rig::new();
    rig.key();
    rig.tell(SpeechNote::Stage(SpeechStage::Open));
    assert_eq!(
        rig.flow.deadline(),
        Some(Mono::from_us(nothing_ms() * 1_000))
    );
    rig.tick_after(nothing_ms() - 1);
    assert!(!hinting(&rig));
    let out = rig.tick_after(1);
    assert!(hinting(&rig));
    assert!(calls(&out).is_empty());
    assert!(rig.lit);
    rig.tell(SpeechNote::Heard);
    assert!(!hinting(&rig));
}

#[test]
fn a_stop_clicked_on_the_view_before_the_hint_still_stops() {
    let mut rig = Rig::new();
    rig.key();
    rig.tell(SpeechNote::Stage(SpeechStage::Open));
    let round = rig.round();
    rig.tick_after(nothing_ms());
    assert!(rig.round() > round);
    let click = VoiceClick::Bar(BarButton::Stop);
    let out = rig.on(In::Act(VoiceAct { round, click }));
    assert_eq!(calls(&out), [RunCall::Cancel, RunCall::Close]);
}

#[test]
fn speech_heard_stops_into_transcribing() {
    let mut rig = Rig::new();
    rig.key();
    rig.speak();
    let out = rig.bar(BarButton::Stop);
    assert_eq!(calls(&out), [RunCall::Stop]);
    assert_eq!(rig.shown(), VoiceBar::Transcribing);
    assert!(!rig.lit);
}

#[test]
fn record_max_s_stops_and_types_with_a_note() {
    let mut rig = Rig::new();
    rig.key();
    rig.speak();
    let out = rig.tell(SpeechNote::Stage(SpeechStage::Limit));
    assert!(calls(&out).is_empty());
    assert!(!rig.lit);
    assert_eq!(rig.shown(), VoiceBar::Transcribing);
    let out = rig.write(&["un long texte"]);
    assert!(ask(&out).is_some());
    let VoiceBar::Text(BarText { hint, .. }) = rig.shown() else {
        panic!("no Text bar");
    };
    assert_eq!(hint, Some(BarHint::TooLong));
}

#[test]
fn record_max_s_with_no_speech_types_nothing() {
    let mut rig = Rig::new();
    rig.key();
    rig.tell(SpeechNote::Stage(SpeechStage::Open));
    let out = rig.tell(SpeechNote::Stage(SpeechStage::Limit));
    assert_eq!(calls(&out), [RunCall::Cancel, RunCall::Close]);
    assert_eq!(note(&rig.shown()).map(|n| n.0), Some(BarHint::Nothing));
}

#[test]
fn silence_stop_is_off_to_start() {
    let mut rig = Rig::new();
    let out = rig.key();
    assert_eq!(opened(&out)[0].silence_stop_s, None);
}

#[test]
fn silence_stop_stops_after_silence_stop_s_when_on() {
    let mut settings = VoiceSettings::from_defaults().unwrap();
    settings.silence_stop = true;
    let wait = settings.silence_stop_s;
    let mut rig = Rig::with(settings, false);
    let out = rig.key();
    assert_eq!(opened(&out)[0].silence_stop_s, Some(wait));
    rig.speak();
    rig.tell(SpeechNote::Quiet);
    rig.tell(SpeechNote::Stage(SpeechStage::Silence));
    assert_eq!(rig.shown(), VoiceBar::Transcribing);
    assert!(!rig.lit);
    let out = rig.write(&["bonjour"]);
    assert!(ask(&out).is_some());
}

#[test]
fn levels_reach_the_bar_only_while_listening() {
    let mut rig = Rig::new();
    rig.key();
    rig.speak();
    let out = rig.tell(SpeechNote::Level(90));
    let level = kx_module_api::ports::VoiceLevel { level: 90 };
    assert!(out.contains(&Out::Level(level)));
    rig.bar(BarButton::Stop);
    let out = rig.tell(SpeechNote::Level(90));
    assert!(!out.iter().any(|o| matches!(o, Out::Level(_))));
}
