//! The text goes to the typer with the suffix, after a fresh look when no box was told; the keyboard's answer reaches
//! the flow by its round, Undo insert wants presses while it can act, the typer drops what is queued at the stop, and
//! Copy goes to the clipboard within its wait.

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use kx_module_api::ports::{
    BarButton, BarHint, ClipPart, FieldError, FieldKind, PanelButton, TextAsk, TextDone, VoiceBar,
    VoiceClick, VoiceTab,
};
use kx_module_api::{Bus, Redacted, Zeroizing};

use super::super::typing::{queue, type_all};
use super::Msg;
use super::rig::Rig;

/// True when the bar shows a text whose Undo insert can act.
fn undo(rig: &Rig) -> bool {
    matches!(rig.bar(), VoiceBar::Text(text) if text.undo)
}

#[test]
fn the_text_goes_to_the_typer_with_the_suffix_and_its_gap() {
    let mut rig = Rig::new();
    rig.speak("hello world");
    let asks = rig.asks();
    assert_eq!(asks.len(), 1);
    assert_eq!(asks[0].text.expose().as_str(), "hello world ");
    assert_eq!((asks[0].erase, asks[0].gap_ms), (0, 10));
}

#[test]
fn the_keyboards_answer_lets_undo_act_and_wants_presses_until_a_click_elsewhere() {
    let mut rig = Rig::new();
    rig.speak("hello");
    assert!(!undo(&rig) && rig.presses().is_empty());
    rig.typed();
    assert!(undo(&rig));
    assert_eq!(rig.presses(), [true]);
    rig.send(Msg::Elsewhere);
    assert!(!undo(&rig));
    assert_eq!(rig.presses(), [true, false]);
}

#[test]
fn a_late_mouse_module_hears_the_wish_again_only_while_undo_can_act() {
    let mut rig = Rig::new();
    rig.send(Msg::PressesAsk);
    assert!(rig.presses().is_empty());
    rig.speak("hello");
    rig.typed();
    rig.send(Msg::PressesAsk);
    assert_eq!(rig.presses(), [true, true]);
}

#[test]
fn undo_erases_exactly_what_was_typed() {
    let mut rig = Rig::new();
    rig.speak("hello");
    rig.typed();
    rig.act(kx_module_api::ports::VoiceClick::Bar(BarButton::Undo));
    let asks = rig.asks();
    assert_eq!(asks.len(), 2);
    assert_eq!(asks[1].erase, 6);
    assert!(asks[1].text.expose().is_empty());
    assert_ne!(asks[0].round, asks[1].round);
}

#[test]
fn an_answer_to_another_round_is_dropped() {
    let mut rig = Rig::new();
    rig.speak("hello");
    let round = rig.asks()[0].round.wrapping_add(1);
    let done = TextDone {
        round,
        count: 6,
        ok: true,
    };
    rig.send(Msg::Typed(done));
    assert!(!undo(&rig));
}

// A focus change told out of order can leave no box while the box in front takes text (P11 task 30's card run).
#[test]
fn a_text_ready_while_no_box_was_told_looks_again_and_types_into_the_box_found() {
    let mut rig = Rig::new();
    rig.fake.field.focus(FieldKind::None);
    rig.pump();
    rig.fake.field.move_unheard(FieldKind::Text);
    rig.speak("hello");
    assert_eq!(rig.asks().len(), 1, "typed into the box found");
    assert_eq!(rig.fake.field.looks(), 1, "once, for the text");
}

#[test]
fn a_text_box_told_takes_no_look() {
    let mut rig = Rig::new();
    rig.speak("hello");
    assert_eq!(rig.fake.field.looks(), 0);
}

#[test]
fn with_no_box_found_or_a_failed_look_the_text_only_shows() {
    for fails in [false, true] {
        let mut rig = Rig::new();
        rig.fake.field.focus(FieldKind::None);
        rig.pump();
        if fails {
            rig.fake.field.fail_reads(FieldError::Refused);
        }
        rig.speak("hello");
        assert!(rig.asks().is_empty(), "failed look: {fails}");
        assert!(matches!(rig.bar(), VoiceBar::Text(t) if t.hint == Some(BarHint::NoField)));
        assert_eq!(rig.fake.field.looks(), 1);
    }
}

#[test]
fn a_gone_typer_answers_nothing_typed() {
    let mut rig = Rig::new();
    rig.actor.ports.typist = Box::new(|_| false);
    rig.speak("hello");
    assert!(matches!(rig.bar(), VoiceBar::Text(t) if t.hint == Some(BarHint::NoField)));
}

#[test]
fn texts_still_queued_when_the_actor_stops_are_never_typed() {
    let rig = Rig::new();
    let (bus, typed) = (
        Bus::new(Arc::clone(&rig.host.core)),
        Arc::new(AtomicU32::new(0)),
    );
    let count = Arc::clone(&typed);
    let _keyboard = bus
        .subscribe(move |_: &TextAsk| {
            count.fetch_add(1, Ordering::SeqCst);
        })
        .unwrap();
    let (hand, texts, live) = queue();
    for round in 1..=2 {
        let text = Redacted::new(Zeroizing::new("hello".to_owned()));
        let ask = TextAsk {
            round,
            erase: 0,
            text,
            gap_ms: 0,
        };
        assert!(hand.send(ask));
    }
    drop(hand);
    type_all(&bus, &texts, &live);
    assert_eq!(typed.load(Ordering::SeqCst), 0);
}

#[test]
fn copy_puts_the_text_on_the_clipboard_within_its_wait() {
    let mut rig = Rig::new();
    rig.speak("hello");
    rig.act(kx_module_api::ports::VoiceClick::Bar(BarButton::Copy));
    let writes = rig.fake.clip.writes();
    assert!(
        matches!(&writes[..], [parts] if matches!(&parts[..], [ClipPart::Text(t)] if t.expose() == "hello"))
    );
    assert_eq!(
        rig.fake.clip.last_wait(),
        Some(rig.actor.settings().copy_wait())
    );
}

/// The rows a panel page holds, as the ui tells them.
const ROWS: u8 = 5;

/// History with "one" then "two" dictated, both ticked in that order, on settings `setup` writes.
fn two_ticked(setup: impl FnOnce(&mut kx_test_support::host::FakeHost)) -> Rig {
    let mut rig = Rig::build(|_, host| setup(host));
    rig.speak("one");
    rig.speak("two");
    rig.act(VoiceClick::Rows(ROWS));
    rig.send(Msg::Open);
    rig.act(VoiceClick::Open(VoiceTab::History));
    // Newest first, so "one" is the second row.
    rig.act(VoiceClick::Row(1));
    rig.act(VoiceClick::Row(0));
    rig
}

/// The text of the last clipboard write.
fn copied(rig: &Rig) -> String {
    let writes = rig.fake.clip.writes();
    match writes.last().map(|w| &w[..]) {
        Some([ClipPart::Text(t)]) => t.expose().as_str().to_owned(),
        other => panic!("{other:?}"),
    }
}

// With no suffix, History's texts still never glue together (P11 task 20).
#[test]
fn history_joins_its_texts_by_the_join_and_types_the_suffix_after_the_last() {
    let mut rig = two_ticked(|host| host.put("type_suffix", "".into()));
    rig.act(VoiceClick::Button(PanelButton::Copy));
    assert_eq!(copied(&rig), "one two");
    let before = rig.asks().len();
    rig.act(VoiceClick::Button(PanelButton::Insert));
    let typed: Vec<String> = rig.asks()[before..]
        .iter()
        .map(|a| a.text.expose().as_str().to_owned())
        .collect();
    assert_eq!(typed, ["one ", "two"]);
}

#[test]
fn history_takes_its_join_from_the_settings() {
    let mut rig = two_ticked(|host| host.put("history_join", " / ".into()));
    rig.act(VoiceClick::Button(PanelButton::Copy));
    assert_eq!(copied(&rig), "one / two");
    let before = rig.asks().len();
    rig.act(VoiceClick::Button(PanelButton::Insert));
    let last = rig
        .asks()
        .last()
        .map(|a| a.text.expose().as_str().to_owned());
    assert_eq!(rig.asks().len() - before, 2);
    assert_eq!(last.as_deref(), Some("two "), "the suffix after the last");
}

#[test]
fn a_copy_the_clipboard_refuses_shows_its_line_on_the_bar() {
    let mut rig = Rig::new();
    rig.speak("hello");
    rig.fake
        .clip
        .queue(Err(kx_module_api::ports::ClipError::Busy));
    rig.act(kx_module_api::ports::VoiceClick::Bar(BarButton::Copy));
    assert!(matches!(rig.bar(), VoiceBar::Text(t) if t.hint == Some(BarHint::CopyFailed)));
    rig.act(kx_module_api::ports::VoiceClick::Bar(BarButton::Copy));
    assert!(
        matches!(rig.bar(), VoiceBar::Text(t) if t.hint.is_none()),
        "a copy that went through"
    );
}
