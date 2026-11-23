//! Paste at once (owner, 2026-11-17; edge 28): with `insert` on paste a text goes out to be pasted, never as a
//! `TextAsk`; a password box is always typed; Undo erases exactly what went in, and Try again replaces it the same way.

use super::*;
use crate::INSERTS;

/// A rig whose texts go in by pasting.
fn pasting() -> Rig {
    let mut settings = VoiceSettings::from_defaults().unwrap();
    settings.insert = INSERTS[1].0.into();
    Rig::with(settings, false)
}

/// The text asked to be pasted among `out`, if any.
fn pasted(out: &[Out]) -> Option<TextAsk> {
    out.iter().find_map(|o| match o {
        Out::Paste(ask) => Some(ask.clone()),
        _ => None,
    })
}

/// The characters of `ask`'s text, as an answer counts them.
fn count(ask: &TextAsk) -> u32 {
    u32::try_from(ask.text.expose().chars().count()).unwrap()
}

#[test]
fn with_paste_chosen_the_text_goes_to_be_pasted_and_is_never_typed() {
    let mut rig = pasting();
    let out = rig.dictate("bonjour");
    assert!(ask(&out).is_none(), "no TextAsk");
    let paste = pasted(&out).unwrap();
    assert_eq!(paste.text.expose().as_str(), suffixed("bonjour"));
    assert_eq!(paste.erase, 0);
}

#[test]
fn typing_chosen_never_pastes() {
    let mut rig = Rig::new();
    let out = rig.dictate("bonjour");
    assert!(pasted(&out).is_none());
    assert!(ask(&out).is_some());
}

#[test]
fn a_password_box_is_always_typed() {
    let mut rig = pasting();
    rig.field(FieldKind::Password, BOX);
    let out = rig.dictate("bonjour");
    assert!(pasted(&out).is_none(), "never pasted");
    assert_eq!(
        ask(&out).unwrap().text.expose().as_str(),
        suffixed("bonjour")
    );
}

#[test]
fn a_box_the_system_cannot_read_takes_the_paste_as_text() {
    let mut rig = pasting();
    rig.field(FieldKind::Unknown, BOX);
    assert!(pasted(&rig.dictate("bonjour")).is_some());
}

#[test]
fn undo_erases_exactly_what_the_paste_put_in() {
    let mut rig = pasting();
    let paste = pasted(&rig.dictate("bonjour")).unwrap();
    rig.typed(&paste, count(&paste));
    let out = rig.bar(BarButton::Undo);
    assert!(pasted(&out).is_none(), "an erase is typed");
    let undo = ask(&out).unwrap();
    assert_eq!(undo.erase, count(&paste));
    assert!(undo.text.expose().is_empty());
}

#[test]
fn try_again_replaces_a_pasted_text_erasing_it_first() {
    let mut rig = pasting();
    let paste = pasted(&rig.dictate("bonjour")).unwrap();
    rig.typed(&paste, count(&paste));
    rig.again();
    let out = rig.write(&["bonsoir"]);
    assert!(ask(&out).is_none());
    let again = pasted(&out).unwrap();
    assert_eq!(again.erase, count(&paste));
    assert_eq!(again.text.expose().as_str(), suffixed("bonsoir"));
}

#[test]
fn a_paste_that_failed_dims_undo_as_a_refused_text_does() {
    let mut rig = pasting();
    let paste = pasted(&rig.dictate("bonjour")).unwrap();
    let (round, count, ok) = (paste.round, 0, false);
    rig.on(In::Typed(TextDone { round, count, ok }));
    let VoiceBar::Text(shown) = rig.shown() else {
        panic!("no Text bar");
    };
    assert_eq!((shown.hint, shown.undo), (Some(BarHint::NoField), false));
}
