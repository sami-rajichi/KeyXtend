//! What a text's pauses let in: a click outside our windows, a board key, a picked word or another text that goes out
//! stops the rest, and the answer counts what went, so Undo insert erases exactly that; a click on our windows or a
//! latch neither stops it nor is spent by it. A send the app refuses ends it with no.

use super::{BREAK, Heard, Mic, ask, done};
use crate::held::A;
use crate::told::{BKSP, text};
use crate::typing::{tap, unicode};
use kx_module_api::ports::{FaceAction, InjectError, KeyId, PointerPressed, TypeWord};
use kx_module_api::{Event, Redacted};
use std::sync::Arc;
use std::thread;

/// Hears a press, outside our windows unless `ours`, from another thread inside each pause, as the mouse module tells
/// it.
fn press_during(m: &Mic, ours: bool) {
    let bus = m.b.bus.clone();
    m.b.rig.fakes.pause.during(move || {
        let bus = bus.clone();
        let told = thread::spawn(move || bus.publish(&PointerPressed { ours }).is_ok());
        assert!(told.join().unwrap(), "the press was told");
    });
}

/// Publishes `event` from another thread inside the first pause only, as another module does.
fn send_during<E: Event + Send + 'static>(m: &Mic, event: E) {
    let bus = m.b.bus.clone();
    let mut once = Some(event);
    m.b.rig.fakes.pause.during(move || {
        if let Some(event) = once.take() {
            let bus = bus.clone();
            let told = thread::spawn(move || bus.publish(&event).is_ok());
            assert!(told.join().unwrap(), "the event was told");
        }
    });
}

/// Taps key `id` on the face inside the first pause only, as the UI's thread does.
fn tap_during(m: &Mic, id: &'static str) {
    let face = Arc::clone(&m.b.rig.face);
    let mut once = Some(FaceAction::Tap {
        id: KeyId::new(id),
        back: false,
    });
    m.b.rig.fakes.pause.during(move || {
        if let Some(action) = once.take() {
            face.act(action).unwrap();
        }
    });
}

#[test]
fn a_click_outside_our_windows_stops_the_rest_and_counts_what_went() {
    let m = Mic::quiet(None);
    press_during(&m, false);
    m.ask(1, 0, "abc", 10);
    assert_eq!(m.b.rig.fakes.injector.strokes(), [unicode("a")]);
    assert_eq!(m.heard(), [BREAK, done(1, 1, false)]);
}

#[test]
fn a_click_outside_while_erasing_stops_before_any_text() {
    let m = Mic::quiet(None);
    press_during(&m, false);
    m.ask(1, 2, "ab", 10);
    assert_eq!(m.b.rig.fakes.injector.strokes(), [tap(BKSP)]);
    assert_eq!(m.heard(), [BREAK, done(1, 0, false)]);
}

#[test]
fn a_click_on_our_windows_goes_on() {
    let m = Mic::quiet(None);
    press_during(&m, true);
    m.ask(1, 0, "abc", 10);
    let want = [unicode("a"), unicode("b"), unicode("c")];
    assert_eq!(m.b.rig.fakes.injector.strokes(), want);
    assert_eq!(m.heard(), [BREAK, done(1, 3, true)]);
}

#[test]
fn a_board_key_sent_mid_text_stops_the_rest_and_is_told_in_its_place() {
    let m = Mic::quiet(None);
    tap_during(&m, A);
    m.ask(1, 0, "xyz", 10);
    let want = [unicode("x"), unicode("a")];
    assert_eq!(m.b.rig.fakes.injector.strokes(), want);
    let heard = [BREAK, Heard::Piece(text("a")), done(1, 1, false)];
    assert_eq!(m.heard(), heard);
}

#[test]
fn a_picked_word_mid_text_stops_the_rest_and_is_told_in_its_place() {
    let m = Mic::quiet(None);
    let word = TypeWord {
        round: 1,
        erase: 0,
        text: Redacted::new("hi".into()),
        line: false,
    };
    send_during(&m, word);
    m.ask(1, 0, "xyz", 10);
    let want = [unicode("x"), unicode("hi")];
    assert_eq!(m.b.rig.fakes.injector.strokes(), want);
    let heard = [BREAK, Heard::Piece(text("hi")), done(1, 1, false)];
    assert_eq!(m.heard(), heard);
}

#[test]
fn a_second_text_mid_text_stops_the_first_and_each_answers() {
    let m = Mic::quiet(None);
    send_during(&m, ask(2, 0, "ab", 0));
    m.ask(1, 0, "xyz", 10);
    let want = [unicode("x"), unicode("ab")];
    assert_eq!(m.b.rig.fakes.injector.strokes(), want);
    let heard = [BREAK, BREAK, done(2, 2, true), done(1, 1, false)];
    assert_eq!(m.heard(), heard);
}

#[test]
fn a_latch_clicked_mid_text_neither_stops_it_nor_is_spent() {
    let m = Mic::quiet(None);
    tap_during(&m, "shift");
    m.ask(1, 0, "xy", 10);
    let want = [unicode("x"), unicode("y")];
    assert_eq!(m.b.rig.fakes.injector.strokes(), want);
    assert_eq!(m.heard(), [BREAK, done(1, 2, true)]);
    assert_eq!(m.b.key("shift")["on"], true);
}

#[test]
fn a_send_the_app_refuses_ends_the_text_with_no_and_tells_nothing() {
    let m = Mic::quiet(None);
    let refused = InjectError::Refused { sent: 0, total: 2 };
    m.b.rig.fakes.injector.set_error(Some(refused));
    m.ask(1, 0, "ab", 10);
    assert_eq!(m.heard(), [done(1, 0, false)]);
    assert!(
        m.b.rig.fakes.pause.waits().is_empty(),
        "nothing more was tried"
    );
}

#[test]
fn a_piece_the_app_took_in_part_counts_as_typed_and_ends_the_text() {
    let m = Mic::quiet(None);
    let part = InjectError::Refused { sent: 1, total: 2 };
    m.b.rig.fakes.injector.set_error(Some(part));
    m.ask(1, 0, "ab", 10);
    assert_eq!(m.heard(), [BREAK, done(1, 1, false)]);
}
