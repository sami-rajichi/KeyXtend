//! The lines a typed text may gain on the bar: typing stopped part way, and a Copy the clipboard refused (P11 task 24).

use super::*;
use kx_module_api::ports::BarText;

/// The Text bar shown.
fn shown_text(rig: &Rig) -> BarText {
    match rig.shown() {
        VoiceBar::Text(text) => text,
        other => panic!("no Text bar: {other:?}"),
    }
}

/// The keyboard's answer to `ask`: `count` characters typed, then it stopped.
fn stopped_at(rig: &mut Rig, ask: &TextAsk, count: u32) {
    let round = ask.round;
    rig.on(In::Typed(TextDone {
        round,
        count,
        ok: false,
    }));
}

#[test]
fn typing_stopped_part_way_says_so_and_keeps_copy() {
    let mut rig = Rig::new();
    let out = rig.dictate("bonjour");
    stopped_at(&mut rig, &ask(&out).unwrap(), 3);
    let shown = shown_text(&rig);
    assert_eq!(shown.hint, Some(BarHint::Stopped));
    assert!(!shown.undo, "Undo cannot erase a text typed in part");
    let out = rig.bar(BarButton::Copy);
    assert!(out.contains(&Out::Copy(secret("bonjour"))));
}

#[test]
fn a_text_nothing_of_which_was_typed_keeps_the_no_field_line() {
    let mut rig = Rig::new();
    let out = rig.dictate("bonjour");
    stopped_at(&mut rig, &ask(&out).unwrap(), 0);
    assert_eq!(shown_text(&rig).hint, Some(BarHint::NoField));
}

#[test]
fn a_refused_copy_shows_its_line_until_a_copy_goes_through() {
    let mut rig = Rig::new();
    rig.field(FieldKind::None, BOX);
    rig.dictate("bonjour");
    rig.bar(BarButton::Copy);
    rig.on(In::Copied(false));
    assert_eq!(shown_text(&rig).hint, Some(BarHint::CopyFailed));
    rig.bar(BarButton::Copy);
    rig.on(In::Copied(true));
    assert_eq!(
        shown_text(&rig).hint,
        Some(BarHint::NoField),
        "the line before"
    );
}

#[test]
fn a_refused_copy_gives_the_line_its_full_time_on_the_bar() {
    let mut rig = Rig::new();
    let settings = VoiceSettings::from_defaults().unwrap();
    rig.dictate("bonjour");
    let wait = u64::try_from(settings.done().as_millis()).unwrap();
    rig.wait(wait - 1);
    rig.on(In::Copied(false));
    rig.tick_after(1);
    assert_eq!(
        shown_text(&rig).hint,
        Some(BarHint::CopyFailed),
        "still shown"
    );
    rig.tick_after(wait);
    assert_eq!(rig.shown(), VoiceBar::Hidden);
}

#[test]
fn a_copy_answer_with_no_text_on_the_bar_changes_nothing() {
    let mut rig = Rig::new();
    let out = rig.on(In::Copied(false));
    assert!(out.is_empty());
    rig.key();
    rig.tell(SpeechNote::Stage(SpeechStage::Open));
    rig.on(In::Copied(false));
    assert!(matches!(rig.shown(), VoiceBar::Listening(_)));
}
