//! Paste at once through the actor (owner, 2026-11-17; edge 28): the text goes to the clipboard history, never the
//! typer; a pasted text counts as typed in full, so Undo erases exactly it; a refusal, or no answer within `paste_ms`,
//! types it once and drops a late answer; a password box is typed, also one met during an erase; Try again erases
//! first, and a failed erase pastes nothing; History pastes once.

use kx_module_api::ports::{
    BarButton, BarHint, FieldKind, PanelButton, SpeechNote, TextDone, VoiceBar, VoiceClick,
    VoiceTab,
};
use kx_module_api::{Redacted, Zeroizing};

use super::Msg;
use super::rig::Rig;
use crate::INSERTS;
use crate::panel::rig::ROWS;

/// A rig whose texts go in by pasting.
fn pasting() -> Rig {
    Rig::build(|_, host| host.put("insert", INSERTS[1].0.into()))
}

/// The texts of `asks`, as the keyboard would get them.
fn texts(asks: &[kx_module_api::ports::TextAsk]) -> Vec<(u32, String)> {
    let one = |a: &kx_module_api::ports::TextAsk| (a.erase, a.text.expose().as_str().to_owned());
    asks.iter().map(one).collect()
}

/// True when the bar shows a text whose Undo insert can act.
fn undo(rig: &Rig) -> bool {
    matches!(rig.bar(), VoiceBar::Text(text) if text.undo)
}

#[test]
fn the_text_goes_to_the_clipboard_history_and_never_to_the_typer() {
    let mut rig = pasting();
    rig.speak("hello");
    let pastes = rig.pastes();
    assert_eq!(pastes.len(), 1);
    assert_eq!(pastes[0].text.expose().as_str(), "hello ");
    assert_eq!(pastes[0].wait_ms, rig.actor.settings().paste_ms);
    assert!(rig.asks().is_empty(), "nothing typed");
}

#[test]
fn a_pasted_text_lets_undo_erase_exactly_its_characters() {
    let mut rig = pasting();
    rig.speak("hello");
    assert!(!undo(&rig), "Undo waits for the answer");
    rig.pasted(true);
    assert!(undo(&rig) && rig.asks().is_empty());
    rig.act(VoiceClick::Bar(BarButton::Undo));
    assert_eq!(texts(&rig.asks()), [(6, String::new())]);
}

#[test]
fn a_refused_paste_types_the_text_once_and_a_late_answer_is_dropped() {
    let mut rig = pasting();
    rig.speak("hello");
    rig.pasted(false);
    assert_eq!(texts(&rig.asks()), [(0, "hello ".to_owned())]);
    rig.pasted(true);
    rig.pasted(false);
    assert_eq!(rig.asks().len(), 1, "typed once");
    assert!(!undo(&rig), "Undo waits for the keyboard");
    rig.typed();
    assert!(undo(&rig));
}

#[test]
fn no_answer_within_paste_ms_types_the_text_and_a_late_one_is_dropped() {
    let mut rig = pasting();
    rig.speak("hello");
    let wait = rig.actor.settings().paste_wait();
    assert_eq!(rig.actor.wait(), Some(wait), "the loop wakes for it");
    rig.advance(wait.saturating_sub(std::time::Duration::from_millis(1)));
    assert!(rig.asks().is_empty());
    rig.advance(std::time::Duration::from_millis(1));
    assert_eq!(texts(&rig.asks()), [(0, "hello ".to_owned())]);
    rig.pasted(true);
    assert_eq!(rig.asks().len(), 1, "typed once");
    assert!(!undo(&rig), "the late paste counts nothing");
}

#[test]
fn a_password_box_is_typed_and_never_pasted() {
    let mut rig = pasting();
    rig.fake.field.focus(FieldKind::Password);
    rig.pump();
    rig.speak("hunter");
    assert!(rig.pastes().is_empty());
    assert_eq!(rig.asks().len(), 1);
}

/// Speaks "hello", pasted, then Try again writes `text`, whose erase of "hello " the typer is asked first.
fn again(rig: &mut Rig, text: &str) {
    rig.speak("hello");
    rig.pasted(true);
    rig.act(VoiceClick::Button(PanelButton::Again));
    rig.tell(SpeechNote::Piece {
        index: 0,
        text: Redacted::new(Zeroizing::new(text.to_owned())),
    });
    rig.tell(SpeechNote::Done(super::rig::done(1)));
    assert_eq!(texts(&rig.asks()), [(6, String::new())], "the erase first");
}

#[test]
fn try_again_erases_the_pasted_text_by_typing_then_pastes_the_new_one() {
    let mut rig = pasting();
    again(&mut rig, "howdy");
    assert_eq!(rig.pastes().len(), 1, "no paste before the erase");
    rig.typed();
    let pastes = rig.pastes();
    assert_eq!(pastes.len(), 2);
    assert_eq!(pastes[1].text.expose().as_str(), "howdy ");
    rig.pasted(true);
    rig.act(VoiceClick::Bar(BarButton::Undo));
    assert_eq!(texts(&rig.asks()).last(), Some(&(6, String::new())));
}

#[test]
fn a_password_box_met_while_the_erase_is_typed_gets_the_new_text_typed() {
    let mut rig = pasting();
    again(&mut rig, "howdy");
    rig.fake.field.focus(FieldKind::Password);
    rig.pump();
    rig.typed();
    assert_eq!(rig.pastes().len(), 1, "never pasted");
    assert_eq!(texts(&rig.asks()).last(), Some(&(0, "howdy ".to_owned())));
}

#[test]
fn a_failed_erase_pastes_nothing_and_says_the_text_did_not_go_in() {
    let mut rig = pasting();
    again(&mut rig, "howdy");
    let round = rig.asks().last().unwrap().round;
    rig.send(Msg::Typed(TextDone {
        round,
        count: 0,
        ok: false,
    }));
    assert_eq!(rig.pastes().len(), 1, "nothing pasted");
    let VoiceBar::Text(shown) = rig.bar() else {
        panic!("no Text bar: {:?}", rig.bar());
    };
    assert_eq!((shown.hint, shown.undo), (Some(BarHint::NoField), false));
}

#[test]
fn history_pastes_its_ticked_texts_once_or_types_them_once() {
    let mut rig = pasting();
    rig.speak("one");
    rig.pasted(true);
    rig.speak("two");
    rig.pasted(true);
    rig.act(VoiceClick::Rows(ROWS));
    rig.send(Msg::Open);
    rig.act(VoiceClick::Open(VoiceTab::History));
    rig.act(VoiceClick::Row(1));
    rig.act(VoiceClick::Row(0));
    rig.act(VoiceClick::Button(PanelButton::Insert));
    let last = rig
        .pastes()
        .last()
        .map(|p| p.text.expose().as_str().to_owned());
    assert_eq!(last.as_deref(), Some("one two "));
    rig.pasted(false);
    assert_eq!(texts(&rig.asks()), [(0, "one two ".to_owned())]);
    rig.fake.field.focus(FieldKind::Password);
    rig.pump();
    let pastes = rig.pastes().len();
    rig.act(VoiceClick::Row(1));
    rig.act(VoiceClick::Row(0));
    rig.act(VoiceClick::Button(PanelButton::Insert));
    assert_eq!(rig.pastes().len(), pastes, "a password box is typed");
    let typed: Vec<_> = texts(&rig.asks()[1..]).into_iter().map(|t| t.1).collect();
    assert_eq!(typed, ["one ", "two "], "as typing does");
}

#[test]
fn typing_chosen_never_asks_the_clipboard_history() {
    let mut rig = Rig::new();
    rig.speak("hello");
    assert!(rig.pastes().is_empty());
    assert_eq!(rig.asks().len(), 1);
}
