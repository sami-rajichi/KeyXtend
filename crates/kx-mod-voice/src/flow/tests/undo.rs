//! Undo insert (edge case 8) and Try again (9): Undo erases exactly the text while nothing else was typed, and Try
//! again replaces it while Undo can act, else leaves the new text on the bar; History keeps the newest only.

use super::*;

/// A dictation of `text` that the keyboard typed in full; gives the count typed.
fn typed_in(rig: &mut Rig, text: &str) -> u32 {
    let out = rig.dictate(text);
    let ask = ask(&out).unwrap();
    let count = u32::try_from(ask.text.expose().chars().count()).unwrap();
    rig.typed(&ask, count);
    count
}

/// Undo insert acts on the bar shown.
fn undo_lit(rig: &Rig) -> bool {
    match rig.shown() {
        VoiceBar::Text(text) => text.undo,
        other => panic!("no Text bar: {other:?}"),
    }
}

#[test]
fn undo_erases_exactly_the_text_typed() {
    let mut rig = Rig::new();
    let count = typed_in(&mut rig, "bonjour");
    assert!(undo_lit(&rig));
    let out = rig.bar(BarButton::Undo);
    let undo = ask(&out).unwrap();
    assert_eq!(undo.erase, count);
    assert!(undo.text.expose().is_empty());
    assert!(!undo_lit(&rig));
    assert!(ask(&rig.bar(BarButton::Undo)).is_none());
    assert_eq!(text(&rig.shown()).as_deref(), Some("bonjour"));
}

#[test]
fn undo_waits_for_the_keyboard_to_finish_typing() {
    let mut rig = Rig::new();
    rig.dictate("bonjour");
    assert!(!undo_lit(&rig));
    assert!(ask(&rig.bar(BarButton::Undo)).is_none());
}

#[test]
fn a_key_or_a_click_elsewhere_dims_undo() {
    let mut rig = Rig::new();
    typed_in(&mut rig, "bonjour");
    rig.on(In::Elsewhere);
    assert!(!undo_lit(&rig));
    assert!(ask(&rig.bar(BarButton::Undo)).is_none());
}

#[test]
fn another_box_taking_focus_dims_undo() {
    let mut rig = Rig::new();
    typed_in(&mut rig, "bonjour");
    rig.field(FieldKind::Text, BOX + 1);
    assert!(!undo_lit(&rig));
}

#[test]
fn the_keyboards_own_typing_does_not_dim_undo() {
    let mut rig = Rig::new();
    let out = rig.dictate("bonjour");
    rig.on(In::Elsewhere);
    rig.typed(&ask(&out).unwrap(), 8);
    assert!(undo_lit(&rig));
}

#[test]
fn try_again_replaces_the_text_while_undo_can_act() {
    let mut rig = Rig::new();
    let count = typed_in(&mut rig, "bonjour");
    rig.book.0.insert("fr".into(), Ok(ready("cohere", "fr")));
    assert!(rig.flow.can_again());
    let out = rig.again();
    let settings = VoiceSettings::from_defaults().unwrap();
    let want = settings.ask(Some(ready("cohere", "fr").model), None);
    assert_eq!(calls(&out), [RunCall::Again(want)]);
    assert_eq!(rig.shown(), VoiceBar::Transcribing);
    let out = rig.write(&["bonsoir"]);
    let typed = ask(&out).unwrap();
    assert_eq!(typed.erase, count);
    assert_eq!(typed.text.expose().as_str(), suffixed("bonsoir"));
    let entry = &kept(&out)[0];
    assert_eq!(entry.take, rig.take);
    assert_eq!(entry.text, secret("bonsoir"));
    assert_eq!(entry.model, Redacted::new("cohere".into()));
}

#[test]
fn try_again_after_other_typing_leaves_the_new_text_on_the_bar() {
    let mut rig = Rig::new();
    typed_in(&mut rig, "bonjour");
    rig.on(In::Elsewhere);
    rig.again();
    let out = rig.write(&["bonsoir"]);
    assert!(ask(&out).is_none());
    assert_eq!(kept(&out)[0].text, secret("bonsoir"));
    assert_eq!(text(&rig.shown()).as_deref(), Some("bonsoir"));
    assert!(!undo_lit(&rig));
    let out = rig.bar(BarButton::Copy);
    assert!(out.contains(&Out::Copy(secret("bonsoir"))));
}

#[test]
fn history_keeps_the_newest_text_of_a_dictation_only() {
    let mut rig = Rig::new();
    let first = kept(&rig.dictate("bonjour")).remove(0);
    rig.again();
    let second = kept(&rig.write(&["bonsoir"])).remove(0);
    assert_eq!(first.take, second.take);
    assert_eq!(second.text, secret("bonsoir"));
}

#[test]
fn try_again_that_writes_nothing_brings_the_text_back() {
    let mut rig = Rig::new();
    let count = typed_in(&mut rig, "bonjour");
    rig.again();
    let out = rig.write(&[]);
    assert!(calls(&out).is_empty() && kept(&out).is_empty() && ask(&out).is_none());
    assert_eq!(text(&rig.shown()).as_deref(), Some("bonjour"));
    assert!(undo_lit(&rig) && rig.flow.can_again());
    assert_eq!(ask(&rig.bar(BarButton::Undo)).unwrap().erase, count);
}

#[test]
fn try_again_needs_a_text_on_the_bar() {
    let mut rig = Rig::new();
    rig.key();
    rig.speak();
    assert!(!rig.flow.can_again());
    assert!(calls(&rig.again()).is_empty());
}

#[test]
fn try_again_without_a_model_now_shows_its_note() {
    let mut rig = Rig::new();
    typed_in(&mut rig, "bonjour");
    rig.book
        .0
        .insert("fr".into(), Err(Lack::Missing { bytes: BYTES }));
    let out = rig.again();
    assert!(calls(&out).is_empty());
    let hint = BarHint::NoModel {
        lang: french(),
        bytes: BYTES,
    };
    assert_eq!(note(&rig.shown()), Some((hint, Some(BarButton::Download))));
}
