//! The bar's close (edge case 10): by Close, after `done_ms` or at a new dictation, wiping the sound each time, while
//! the pointer or an open panel keeps it; and Try again dimmed once `idle_min` wiped the recording (11).

use super::*;
use kx_module_api::ports::{DeviceLabel, Pages, VoiceHome, VoicePanel, VoiceSheet};

/// `done_ms` in the defaults.
pub(super) fn done_ms() -> u64 {
    VoiceSettings::from_defaults().unwrap().done_ms.into()
}

/// The Voice panel's home page.
pub(super) fn panel() -> VoicePanel {
    let home = VoiceHome {
        model: None,
        device: DeviceLabel::Cpu,
        compared: None,
        history: 0,
        changed: 0,
        cloud: None,
    };
    VoicePanel {
        sheet: VoiceSheet::Home(home),
        lang: french(),
        pages: Pages { at: 0, count: 1 },
        ticked: 0,
        acts: Vec::new(),
        note: None,
    }
}

#[test]
fn close_wipes_the_recording_and_hides_the_bar() {
    let mut rig = Rig::new();
    rig.dictate("bonjour");
    let out = rig.bar(BarButton::Close);
    assert_eq!(calls(&out), [RunCall::Close]);
    assert_eq!(rig.shown(), VoiceBar::Hidden);
    assert!(!rig.flow.can_again());
}

#[test]
fn the_text_fades_after_done_ms_and_wipes_the_recording() {
    let mut rig = Rig::new();
    rig.dictate("bonjour");
    assert!(rig.flow.deadline().is_some());
    rig.tick_after(done_ms() - 1);
    assert!(text(&rig.shown()).is_some());
    let out = rig.tick_after(1);
    assert_eq!(calls(&out), [RunCall::Close]);
    assert_eq!(rig.shown(), VoiceBar::Hidden);
    assert_eq!(rig.flow.deadline(), None);
}

#[test]
fn a_note_fades_after_done_ms_too() {
    let mut rig = Rig::new();
    rig.key();
    rig.speak();
    rig.bar(BarButton::Cancel);
    let out = rig.tick_after(done_ms());
    assert_eq!(calls(&out), [RunCall::Close]);
    assert_eq!(rig.shown(), VoiceBar::Hidden);
}

#[test]
fn a_new_dictation_closes_the_old_bar_first() {
    let mut rig = Rig::new();
    rig.dictate("bonjour");
    let old = rig.take;
    rig.wait(1_000);
    let out = rig.key();
    let closed = out.iter().position(|o| {
        *o == Out::Run {
            take: old,
            call: RunCall::Close,
        }
    });
    let open = out.iter().position(|o| matches!(o, Out::Open { .. }));
    assert!(closed.unwrap() < open.unwrap());
    assert_ne!(rig.take, old);
}

#[test]
fn the_pointer_over_the_bar_keeps_it_open() {
    let mut rig = Rig::new();
    rig.dictate("bonjour");
    rig.click(VoiceClick::Over(true));
    assert_eq!(rig.flow.deadline(), None);
    rig.tick_after(done_ms());
    assert!(text(&rig.shown()).is_some());
    rig.click(VoiceClick::Over(false));
    rig.tick_after(done_ms() - 1);
    assert!(text(&rig.shown()).is_some());
    rig.tick_after(1);
    assert_eq!(rig.shown(), VoiceBar::Hidden);
}

#[test]
fn the_pointer_over_a_closed_bar_does_not_keep_the_next_one() {
    let mut rig = Rig::new();
    rig.dictate("bonjour");
    rig.click(VoiceClick::Over(true));
    rig.bar(BarButton::Close);
    rig.click(VoiceClick::Over(true));
    rig.wait(1_000);
    rig.dictate("bonsoir");
    assert!(rig.flow.deadline().is_some());
    rig.tick_after(done_ms());
    assert_eq!(rig.shown(), VoiceBar::Hidden);
}

#[test]
fn an_open_panel_keeps_the_bar_and_done_ms_starts_again_when_it_closes() {
    let mut rig = Rig::new();
    rig.dictate("bonjour");
    rig.on(In::Panel(Some(Box::new(panel()))));
    assert_eq!(rig.page.as_ref().unwrap().panel, Some(panel()));
    rig.tick_after(done_ms() * 2);
    assert!(text(&rig.shown()).is_some());
    rig.on(In::Panel(None));
    rig.tick_after(done_ms() - 1);
    assert!(text(&rig.shown()).is_some());
    rig.tick_after(1);
    assert_eq!(rig.shown(), VoiceBar::Hidden);
}

#[test]
fn a_click_on_an_older_bar_is_dropped() {
    let mut rig = Rig::new();
    rig.dictate("bonjour");
    let round = rig.round();
    rig.wait(1_000);
    rig.dictate("bonsoir");
    let click = VoiceClick::Bar(BarButton::Close);
    let out = rig.on(In::Act(VoiceAct { round, click }));
    assert!(calls(&out).is_empty());
    assert_eq!(text(&rig.shown()).as_deref(), Some("bonsoir"));
    assert_eq!(calls(&rig.bar(BarButton::Close)), [RunCall::Close]);
}

#[test]
fn a_view_is_told_once_per_change() {
    let mut rig = Rig::new();
    rig.key();
    let out = rig.tell(SpeechNote::Quiet);
    assert!(!out.iter().any(|o| matches!(o, Out::View(_))));
    rig.wait(1_000);
    let out = rig.tell(SpeechNote::Heard);
    assert!(!out.iter().any(|o| matches!(o, Out::View(_))));
}

#[test]
fn a_late_starter_hears_the_light_and_the_view_again() {
    let mut rig = Rig::new();
    rig.key();
    rig.speak();
    let round = rig.round();
    let out = rig.on(In::Ask);
    assert!(out.contains(&Out::Lit(VoiceLit::Rec)));
    let view = out.iter().find_map(|o| match o {
        Out::View(view) => Some(view.0.round),
        _ => None,
    });
    assert_eq!(view, Some(round));
}

#[test]
fn try_again_dims_once_idle_min_wiped_the_recording() {
    let mut rig = Rig::new();
    rig.dictate("bonjour");
    assert!(rig.flow.can_again());
    rig.tell(SpeechNote::Wiped);
    assert!(!rig.flow.can_again());
    assert!(calls(&rig.again()).is_empty());
    assert!(text(&rig.shown()).is_some());
}

#[test]
fn transcribe_anyway_goes_once_idle_min_wiped_the_recording() {
    let mut rig = Rig::new();
    rig.key();
    rig.speak();
    rig.bar(BarButton::Cancel);
    rig.tell(SpeechNote::Wiped);
    assert_eq!(note(&rig.shown()), Some((BarHint::Cancelled, None)));
    assert!(calls(&rig.bar(BarButton::Anyway)).is_empty());
}
