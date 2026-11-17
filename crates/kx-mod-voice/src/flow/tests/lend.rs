//! Compare models borrowing the microphone: one bar and one light, the key, Stop and Cancel go back to Compare, no
//! dictation starts while it holds the run, and a lock or Quit cancels it.

use super::*;
use kx_module_api::ports::SessionNote;

/// The Stop and Cancel handed to Compare in `out`.
fn lent(out: &[Out]) -> Vec<BarButton> {
    let button = |o: &Out| match o {
        Out::Lent(b) => Some(*b),
        _ => None,
    };
    out.iter().filter_map(button).collect()
}

#[test]
fn while_compare_records_the_bar_listens_and_the_key_glows() {
    let mut rig = Rig::new();
    rig.on(In::Lend(Lend::Listen));
    assert!(rig.lit);
    assert!(matches!(rig.shown(), VoiceBar::Listening(l) if l.lang == french()));
    rig.on(In::Lend(Lend::Write));
    assert!(!rig.lit, "the recording ended");
    assert_eq!(rig.shown(), VoiceBar::Hidden);
    rig.on(In::Lend(Lend::Off));
    assert!(!rig.lit);
    assert!(!opened(&rig.key()).is_empty(), "dictation is free again");
}

#[test]
fn the_key_stop_and_cancel_go_back_to_compare() {
    let presses = [
        (None, BarButton::Stop),
        (Some(BarButton::Stop), BarButton::Stop),
        (Some(BarButton::Cancel), BarButton::Cancel),
    ];
    for (press, want) in presses {
        let mut rig = Rig::new();
        rig.on(In::Lend(Lend::Listen));
        let out = match press {
            Some(button) => rig.bar(button),
            None => rig.key(),
        };
        assert_eq!(lent(&out), [want], "{press:?}");
        assert!(
            opened(&out).is_empty() && calls(&out).is_empty(),
            "{press:?}"
        );
        assert!(!rig.lit, "{press:?}");
    }
}

#[test]
fn no_dictation_starts_while_compare_writes() {
    let mut rig = Rig::new();
    rig.on(In::Lend(Lend::Write));
    let out = rig.key();
    assert!(opened(&out).is_empty() && lent(&out).is_empty());
    assert_eq!(rig.shown(), VoiceBar::Hidden);
}

#[test]
fn taking_the_microphone_closes_a_dictation_bar() {
    let mut rig = Rig::new();
    rig.dictate("bonjour");
    let out = rig.on(In::Lend(Lend::Listen));
    assert_eq!(calls(&out), [RunCall::Close], "the held recording is wiped");
    assert!(matches!(rig.shown(), VoiceBar::Listening(_)));
}

#[test]
fn a_lock_quit_or_the_feature_off_cancels_compare() {
    let mut off = VoiceSettings::from_defaults().unwrap();
    off.on = false;
    let ends = [
        In::Session(SessionNote::Locked),
        In::Quit,
        In::Settings(Box::new(off)),
    ];
    for end in ends {
        let mut rig = Rig::new();
        rig.on(In::Lend(Lend::Listen));
        let out = rig.on(end);
        assert_eq!(lent(&out), [BarButton::Cancel]);
        assert!(!rig.lit);
        assert_eq!(rig.shown(), VoiceBar::Hidden);
    }
}
