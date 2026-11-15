//! The views: told once per change with a new round, the bar's times armed from the flow's deadline, and the panel
//! riding on the view.

use std::time::Duration;

use kx_module_api::ports::{FieldKind, SpeechNote, VoiceBar, VoiceClick, VoiceSheet, VoiceTab};

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
