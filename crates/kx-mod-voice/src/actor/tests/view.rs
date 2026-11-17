//! The views: told once per change with a new round, the bar's times armed from the flow's deadline, and the panel
//! riding on the view, its gear ending a running dictation first.

use std::time::Duration;

use kx_module_api::ports::{
    BarButton, BarHint, FieldKind, SpeechNote, SpeechStage, VoiceBar, VoiceClick, VoiceLit,
    VoicePage, VoiceSheet, VoiceTab,
};

use super::Msg;
use super::rig::{Rig, english_layout};

#[test]
fn a_view_is_told_once_per_change() {
    let mut rig = Rig::new();
    let told = rig.views().len();
    rig.send(Msg::Layout(english_layout()));
    rig.fake.field.focus(FieldKind::Text);
    rig.pump();
    assert_eq!(rig.views().len(), told, "nothing the bar shows changed");
    rig.listen();
    rig.tell(SpeechNote::Level(10));
    assert_eq!(rig.views().len(), told + 1, "only Listening was told");
    let rounds: Vec<u32> = rig.views().iter().map(|v| v.round).collect();
    assert!(rounds.windows(2).all(|w| w[0] < w[1]), "{rounds:?}");
}

#[test]
fn the_text_fades_at_the_flows_deadline() {
    let mut rig = Rig::new();
    rig.speak("hello");
    assert!(matches!(rig.bar(), VoiceBar::Text(_)));
    let done = rig.actor.settings().done();
    assert_eq!(rig.actor.wait(), Some(done));
    rig.advance(done.checked_sub(Duration::from_millis(1)).unwrap());
    assert!(matches!(rig.bar(), VoiceBar::Text(_)));
    rig.advance(Duration::from_millis(1));
    assert_eq!((rig.bar(), rig.actor.wait()), (VoiceBar::Hidden, None));
}

#[test]
fn nothing_heard_shows_its_hint_once_nothing_s_passed() {
    let mut rig = Rig::new();
    rig.key();
    rig.tell(SpeechNote::Stage(SpeechStage::Open));
    rig.advance(rig.actor.settings().nothing());
    assert!(matches!(rig.bar(), VoiceBar::Listening(l) if l.nothing));
}

#[test]
fn the_gear_opens_the_panel_on_the_view_and_its_clicks_reach_the_panel() {
    let mut rig = Rig::new();
    rig.send(Msg::Open);
    let page = rig.views().pop().unwrap();
    assert!(matches!(
        page.panel.map(|p| p.sheet),
        Some(VoiceSheet::Home(_))
    ));
    rig.act(VoiceClick::Open(VoiceTab::Models));
    let page = rig.views().pop().unwrap();
    assert!(matches!(
        page.panel.map(|p| p.sheet),
        Some(VoiceSheet::Models(_))
    ));
}

#[test]
fn the_gear_while_listening_cancels_the_run_before_the_panel_opens() {
    let mut rig = Rig::new();
    rig.listen();
    let told = rig.views().len();
    rig.send(Msg::Open);
    let run = &rig.fake.speech.runs()[0];
    assert_eq!((run.cancels, run.stops), (1, 0), "nothing is transcribed");
    assert!(
        run.held && !run.closed,
        "the recording stays for Transcribe anyway"
    );
    assert_eq!(rig.lits().last(), Some(&VoiceLit::Off));
    assert!(rig.asks().is_empty(), "nothing is typed");
    let views = rig.views().split_off(told);
    let cancelled =
        |p: &VoicePage| matches!(&p.bar, VoiceBar::Note(n) if n.hint == BarHint::Cancelled);
    let note = views.iter().position(cancelled).unwrap();
    let panel = views.iter().position(|p| p.panel.is_some()).unwrap();
    assert!(
        note < panel,
        "the dictation ends first, then the panel opens"
    );
    let last = views.last().unwrap();
    assert!(matches!(&last.bar, VoiceBar::Note(n) if n.act == Some(BarButton::Anyway)));
}

#[test]
fn the_gear_while_preparing_cancels_the_run_and_the_key_never_turns_red() {
    let mut rig = Rig::new();
    rig.key();
    rig.send(Msg::Open);
    let run = &rig.fake.speech.runs()[0];
    assert_eq!((run.cancels, run.stops), (1, 0));
    assert_eq!(
        rig.lits(),
        [VoiceLit::Off, VoiceLit::Prep, VoiceLit::Off],
        "amber, never red: the microphone never said it opened"
    );
    let page = rig.views().pop().unwrap();
    assert!(
        matches!(&page.bar, VoiceBar::Note(n) if n.hint == BarHint::Cancelled && n.act.is_none())
    );
    assert!(matches!(
        page.panel.map(|p| p.sheet),
        Some(VoiceSheet::Home(_))
    ));
}

#[test]
fn the_gear_while_transcribing_cancels_the_writing_and_keeps_the_recording() {
    let mut rig = Rig::new();
    rig.listen();
    rig.key();
    assert_eq!(rig.bar(), VoiceBar::Transcribing);
    rig.send(Msg::Open);
    let run = &rig.fake.speech.runs()[0];
    assert_eq!((run.cancels, run.stops), (1, 1));
    assert!(
        run.held && !run.closed,
        "the recording stays for Transcribe anyway"
    );
    assert!(rig.asks().is_empty(), "nothing is typed");
    let page = rig.views().pop().unwrap();
    let anyway = Some(BarButton::Anyway);
    assert!(
        matches!(&page.bar, VoiceBar::Note(n) if n.hint == BarHint::Cancelled && n.act == anyway)
    );
    assert!(matches!(
        page.panel.map(|p| p.sheet),
        Some(VoiceSheet::Home(_))
    ));
}

#[test]
fn the_gear_ends_a_dictation_started_under_the_open_panel_and_leaves_the_panel_as_it_was() {
    let mut rig = Rig::new();
    rig.send(Msg::Open);
    rig.act(VoiceClick::Open(VoiceTab::Models));
    rig.listen();
    assert!(matches!(rig.bar(), VoiceBar::Listening(_)));
    let before = rig.views().pop().unwrap().panel;
    rig.send(Msg::Open);
    assert_eq!(rig.fake.speech.runs()[0].cancels, 1);
    let page = rig.views().pop().unwrap();
    assert!(matches!(&page.bar, VoiceBar::Note(n) if n.hint == BarHint::Cancelled));
    assert_eq!(page.panel, before, "the panel stays as it was");
    assert!(matches!(
        page.panel.map(|p| p.sheet),
        Some(VoiceSheet::Models(_))
    ));
}
