//! Quit, or a lock or a sleep told by `SessionWatch`, while listening: stop, wipe, light off (edge case 14); and the
//! feature turned off.

use super::*;
use kx_module_api::ports::SessionNote;

#[test]
fn a_lock_or_a_sleep_while_listening_stops_wipes_and_unlights() {
    for away in [SessionNote::Locked, SessionNote::Sleeping] {
        let mut rig = Rig::new();
        rig.key();
        rig.speak();
        let out = rig.on(In::Session(away));
        assert_eq!(calls(&out), [RunCall::Cancel, RunCall::Close], "{away:?}");
        assert!(!rig.lit, "{away:?}");
        assert_eq!(rig.shown(), VoiceBar::Hidden, "{away:?}");
    }
}

#[test]
fn a_lock_while_transcribing_types_nothing() {
    let mut rig = Rig::new();
    rig.key();
    rig.speak();
    rig.bar(BarButton::Stop);
    rig.on(In::Session(SessionNote::Locked));
    let out = rig.write(&["bonjour"]);
    assert!(ask(&out).is_none() && kept(&out).is_empty());
}

#[test]
fn a_lock_wipes_the_recording_of_an_open_text_bar() {
    let mut rig = Rig::new();
    rig.dictate("bonjour");
    let out = rig.on(In::Session(SessionNote::Locked));
    assert_eq!(calls(&out), [RunCall::Close]);
    assert_eq!(rig.shown(), VoiceBar::Hidden);
}

#[test]
fn an_unlock_or_a_wake_changes_nothing() {
    for back in [SessionNote::Unlocked, SessionNote::Woke] {
        let mut rig = Rig::new();
        rig.key();
        rig.speak();
        let out = rig.on(In::Session(back));
        assert!(calls(&out).is_empty(), "{back:?}");
        assert!(rig.lit, "{back:?}");
    }
}

#[test]
fn quit_while_listening_stops_wipes_and_unlights() {
    let mut rig = Rig::new();
    rig.key();
    rig.speak();
    let out = rig.on(In::Quit);
    assert_eq!(calls(&out), [RunCall::Cancel, RunCall::Close]);
    assert!(!rig.lit);
    assert_eq!(rig.shown(), VoiceBar::Hidden);
}

#[test]
fn the_feature_off_stops_and_ignores_the_key() {
    let mut rig = Rig::new();
    rig.key();
    rig.speak();
    let mut off = VoiceSettings::from_defaults().unwrap();
    off.on = false;
    let out = rig.on(In::Settings(Box::new(off)));
    assert_eq!(calls(&out), [RunCall::Cancel, RunCall::Close]);
    rig.wait(1_000);
    assert!(opened(&rig.key()).is_empty());
    assert_eq!(rig.shown(), VoiceBar::Hidden);
}
