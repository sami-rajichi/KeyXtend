//! The text: typed at once and kept, a password box (edge case 6), nothing in front that takes text (7), and a
//! hidden run (16).

use super::*;
use kx_module_api::ports::BarText;

/// The Text bar's hint and Undo.
fn shown_text(rig: &Rig) -> BarText {
    match rig.shown() {
        VoiceBar::Text(text) => text,
        other => panic!("no Text bar: {other:?}"),
    }
}

/// Settings with Keep recordings on.
fn keeping() -> VoiceSettings {
    let mut settings = VoiceSettings::from_defaults().unwrap();
    settings.keep_voice = true;
    settings
}

#[test]
fn stop_types_the_text_at_once_and_keeps_it() {
    let mut rig = Rig::new();
    let out = rig.dictate("bonjour Maya");
    let settings = VoiceSettings::from_defaults().unwrap();
    let typed = ask(&out).unwrap();
    assert_eq!(typed.erase, 0);
    assert_eq!(typed.text.expose().as_str(), suffixed("bonjour Maya"));
    assert_eq!(typed.gap_ms, settings.type_gap_ms);
    assert_eq!(text(&rig.shown()).as_deref(), Some("bonjour Maya"));
    let entry = Kept {
        take: rig.take,
        text: secret("bonjour Maya"),
        lang: "fr".into(),
        model: Redacted::new("parakeet".into()),
    };
    assert_eq!(kept(&out), [entry]);
    let count = Count {
        model: Redacted::new("parakeet".into()),
        done: done(1),
    };
    assert!(out.contains(&Out::Count(count)));
    assert!(!calls(&out).contains(&RunCall::Keep));
}

#[test]
fn pieces_join_on_one_line_without_hidden_marks() {
    let mut rig = Rig::new();
    rig.key();
    rig.speak();
    rig.bar(BarButton::Stop);
    let out = rig.write(&["Bonjour\r\nMaya,", "  la réunion\u{202e} ", "\u{2028}"]);
    let want = "Bonjour Maya, la réunion";
    assert_eq!(ask(&out).unwrap().text.expose().as_str(), suffixed(want));
    assert_eq!(text(&rig.shown()).as_deref(), Some(want));
}

#[test]
fn pieces_join_in_their_order() {
    let mut rig = Rig::new();
    rig.key();
    rig.speak();
    rig.tell(SpeechNote::Piece {
        index: 1,
        text: secret("monde"),
    });
    rig.tell(SpeechNote::Piece {
        index: 0,
        text: secret("bonjour"),
    });
    rig.bar(BarButton::Stop);
    rig.tell(SpeechNote::Done(done(2)));
    assert_eq!(text(&rig.shown()).as_deref(), Some("bonjour monde"));
}

#[test]
fn arabic_text_reads_right_to_left() {
    let mut rig = Rig::new();
    rig.dictate("شكرًا جزيلًا");
    let VoiceText::Shown { rtl, arabic, .. } = shown_text(&rig).text else {
        panic!("no shown text");
    };
    assert!(rtl && arabic);
}

#[test]
fn keep_recordings_asks_the_run_for_its_sound_once() {
    let mut rig = Rig::with(keeping(), false);
    let out = rig.dictate("bonjour");
    assert_eq!(calls(&out), [RunCall::Keep]);
}

#[test]
fn a_password_box_is_typed_into_shows_dots_and_keeps_nothing() {
    let mut rig = Rig::with(keeping(), false);
    rig.field(FieldKind::Password, BOX);
    let out = rig.dictate("hunter2");
    assert_eq!(
        ask(&out).unwrap().text.expose().as_str(),
        suffixed("hunter2")
    );
    assert!(kept(&out).is_empty() && !calls(&out).contains(&RunCall::Keep));
    let shown = shown_text(&rig);
    assert_eq!(shown.text, VoiceText::Masked);
    assert_eq!(shown.hint, Some(BarHint::Password));
    assert!(!format!("{:?}", rig.page).contains("hunter2"));
    let out = rig.bar(BarButton::Copy);
    assert!(!out.iter().any(|o| matches!(o, Out::Copy(_))));
}

#[test]
fn try_again_after_leaving_a_password_box_stays_dots_and_unkept() {
    let mut rig = Rig::new();
    rig.field(FieldKind::Password, BOX);
    rig.dictate("hunter2");
    rig.field(FieldKind::Text, BOX + 1);
    rig.again();
    let out = rig.write(&["hunter3"]);
    assert!(ask(&out).is_none() && kept(&out).is_empty());
    assert_eq!(shown_text(&rig).text, VoiceText::Masked);
    let out = rig.bar(BarButton::Copy);
    assert!(!out.iter().any(|o| matches!(o, Out::Copy(_))));
}

#[test]
fn nothing_in_front_keeps_the_text_on_the_bar_with_copy_and_in_history() {
    let mut rig = Rig::new();
    rig.field(FieldKind::None, BOX);
    let out = rig.dictate("bonjour");
    assert!(ask(&out).is_none());
    assert_eq!(kept(&out).len(), 1);
    let shown = shown_text(&rig);
    assert_eq!(shown.hint, Some(BarHint::NoField));
    assert!(!shown.undo);
    let out = rig.bar(BarButton::Copy);
    assert!(out.contains(&Out::Copy(secret("bonjour"))));
}

#[test]
fn a_box_the_system_cannot_read_is_typed_into() {
    let mut rig = Rig::new();
    rig.field(FieldKind::Unknown, BOX);
    let out = rig.dictate("bonjour");
    assert!(ask(&out).is_some());
    assert_eq!(shown_text(&rig).hint, None);
}

#[test]
fn a_text_the_keyboard_could_not_type_stays_with_copy() {
    let mut rig = Rig::new();
    let out = rig.dictate("bonjour");
    let round = ask(&out).unwrap().round;
    rig.on(In::Typed(TextDone {
        round,
        count: 0,
        ok: false,
    }));
    let shown = shown_text(&rig);
    assert_eq!(shown.hint, Some(BarHint::NoField));
    assert!(!shown.undo);
}

#[test]
fn a_hidden_run_types_and_keeps_nothing() {
    let mut rig = Rig::with(keeping(), true);
    let out = rig.dictate("bonjour");
    assert!(ask(&out).is_some());
    assert!(kept(&out).is_empty());
    assert!(!out.iter().any(|o| matches!(o, Out::Count(_))));
    assert!(!calls(&out).contains(&RunCall::Keep));
    assert_eq!(text(&rig.shown()).as_deref(), Some("bonjour"));
}

#[test]
fn an_empty_text_types_nothing_and_says_nothing_was_heard() {
    let mut rig = Rig::new();
    let out = rig.dictate("  \n ");
    assert!(ask(&out).is_none() && kept(&out).is_empty());
    assert_eq!(calls(&out), [RunCall::Close]);
    let want = Some((BarHint::Nothing, Some(BarButton::Listen)));
    assert_eq!(note(&rig.shown()), want);
}
