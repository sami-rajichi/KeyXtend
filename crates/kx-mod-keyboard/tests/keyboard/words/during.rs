//! What a picked word's pause lets in: a click outside our windows, or a board key that goes out, stops the rest of the
//! word, with what went told before it; a click on our windows or a latch clicked then neither stops it nor joins it.

use super::{A, BKSP, Desk, text};
use crate::typing::{tap, unicode};
use kx_module_api::ports::{FaceAction, KeyId, Piece, PointerPressed, Typed};
use kx_platform_fake::Stroke;
use std::sync::{Arc, Mutex};
use std::thread;

/// The French dead `^` and the key of virtual key E on AZERTY.
const CIRCUMFLEX: &str = "c-0x001A";
const E: &str = "c-0x0012";

/// Hears a press, outside our windows unless `ours`, from another thread inside each pause, as the mouse module
/// tells it.
fn press_during(d: &Desk, ours: bool) {
    let bus = d.bus.clone();
    d.rig.fakes.pause.during(move || {
        let bus = bus.clone();
        let told = thread::spawn(move || bus.publish(&PointerPressed { ours }).is_ok());
        assert!(told.join().unwrap(), "the press was told");
    });
}

/// Taps key `id` on the face inside the first pause only, as the UI's thread does.
fn tap_during(d: &Desk, id: &'static str) {
    act_during(
        d,
        FaceAction::Tap {
            id: KeyId::new(id),
            back: false,
        },
    );
}

/// Hands `action` to the face inside the first pause only, as the UI's thread does.
fn act_during(d: &Desk, action: FaceAction) {
    let face = Arc::clone(&d.rig.face);
    let mut once = Some(action);
    d.rig.fakes.pause.during(move || {
        if let Some(action) = once.take() {
            face.act(action).unwrap();
        }
    });
}

/// What the fake app saw when key `id` was tapped alone.
fn alone(id: &str) -> Vec<Stroke> {
    let d = Desk::new(None);
    d.rig.tap(id).unwrap();
    d.rig.fakes.injector.strokes()
}

/// A Backspace as the fake app saw it.
fn bksp() -> Stroke {
    tap(BKSP)
}

#[test]
fn a_click_outside_our_windows_during_a_pick_stops_the_rest_at_once() {
    for erase in [1, 3] {
        let d = Desk::new(None);
        press_during(&d, false);
        d.word(erase, "hello ");
        assert_eq!(d.rig.fakes.injector.strokes(), [bksp()], "erase {erase}");
        assert_eq!(d.pieces(), [Piece::Erase], "what went is told");
        assert_eq!(d.rig.fakes.pause.waits().len(), 1);
    }
}

#[test]
fn a_stopped_picks_erase_is_heard_before_the_press_that_stopped_it() {
    let d = Desk::new(None);
    let heard = Arc::new(Mutex::new(Vec::new()));
    let (pieces, presses) = (Arc::clone(&heard), Arc::clone(&heard));
    let _ears = [
        d.bus
            .subscribe(move |e: &Typed| pieces.lock().unwrap().push(Some(e.piece.clone())))
            .unwrap(),
        d.bus
            .subscribe(move |_: &PointerPressed| presses.lock().unwrap().push(None))
            .unwrap(),
    ];
    press_during(&d, false);
    d.word(3, "hello ");
    assert_eq!(
        *heard.lock().unwrap(),
        [Some(Piece::Erase), None],
        "the Erase, then the press"
    );
}

#[test]
fn a_click_on_our_windows_or_one_before_the_pick_stops_nothing() {
    let d = Desk::new(None);
    press_during(&d, true);
    d.word(2, "ab");
    let all = [bksp(), bksp(), unicode("ab")];
    assert_eq!(
        d.rig.fakes.injector.strokes(),
        all,
        "a click on our windows"
    );
    let d = Desk::new(None);
    d.bus.publish(&PointerPressed { ours: false }).unwrap();
    d.word(2, "ab");
    assert_eq!(
        d.rig.fakes.injector.strokes(),
        all,
        "a click before the pick"
    );
}

#[test]
fn a_latch_clicked_during_a_pause_keeps_the_text_and_stays_latched() {
    for latch in ["ctrl", "shift"] {
        let d = Desk::new(None);
        tap_during(&d, latch);
        d.word(1, "ab");
        let all = [bksp(), unicode("ab")];
        assert_eq!(d.rig.fakes.injector.strokes(), all, "{latch}");
        d.rig.act(FaceAction::Relabel).unwrap();
        assert_eq!(
            d.rig.state()["keys"][latch]["on"],
            true,
            "{latch} not spent"
        );
    }
}

#[test]
fn a_shift_latched_before_the_pick_is_spent_by_it() {
    let d = Desk::new(None);
    d.rig.tap("shift").unwrap();
    d.word(1, "Ab");
    d.rig.act(FaceAction::Relabel).unwrap();
    assert_eq!(d.rig.state()["keys"]["shift"]["on"], false);
}

#[test]
fn a_board_key_tapped_or_pressed_during_a_pause_goes_out_and_stops_the_rest() {
    for action in [
        FaceAction::Tap {
            id: KeyId::new(A),
            back: false,
        },
        FaceAction::Press { id: KeyId::new(A) },
    ] {
        let d = Desk::new(None);
        act_during(&d, action.clone());
        d.word(3, "hello ");
        let mut want = vec![bksp()];
        want.extend(alone(A));
        assert_eq!(d.rig.fakes.injector.strokes(), want, "{action:?}");
        assert_eq!(d.pieces(), [Piece::Erase, text("a")], "{action:?}");
        assert_eq!(d.rig.fakes.pause.waits().len(), 1, "{action:?}");
    }
}

#[test]
fn a_dead_key_clicked_during_a_pause_stops_the_text_and_still_waits() {
    let d = Desk::new(None);
    d.rig.switch("fr-fr");
    d.pieces();
    tap_during(&d, CIRCUMFLEX);
    d.word(1, "ab");
    let strokes = d.rig.fakes.injector.strokes();
    assert_eq!(strokes.first(), Some(&bksp()));
    assert!(!strokes.contains(&unicode("ab")), "the ^ stops the word");
    let told = [Piece::Erase, Piece::Break];
    assert_eq!(d.pieces(), told, "the ^ breaks the word where it went");
    assert_eq!(d.modes().last().map(|m| m.dead), Some(true), "the ^ waits");
    d.rig.tap(E).unwrap();
    assert_eq!(d.modes().last().map(|m| m.dead), Some(false), "ê ends it");
}
