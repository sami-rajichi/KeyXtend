//! What a picked word's pause lets in: a click outside our windows stops the rest of the word at once, with what went
//! already told; a click on our windows, a latch or a dead key clicked then neither stops it nor joins it.

use super::{BKSP, Desk, text};
use crate::typing::{tap, unicode};
use kx_module_api::ports::{FaceAction, KeyId, Piece, PointerPressed};
use kx_platform_fake::Stroke;
use std::sync::Arc;
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
    let face = Arc::clone(&d.rig.face);
    let mut once = true;
    d.rig.fakes.pause.during(move || {
        if std::mem::take(&mut once) {
            let action = FaceAction::Tap {
                id: KeyId::new(id),
                back: false,
            };
            face.act(action).unwrap();
        }
    });
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
fn a_dead_key_clicked_during_a_pause_keeps_the_text_and_still_waits() {
    let d = Desk::new(None);
    d.rig.switch("fr-fr");
    d.pieces();
    tap_during(&d, CIRCUMFLEX);
    d.word(1, "ab");
    let strokes = d.rig.fakes.injector.strokes();
    assert_eq!(strokes.first(), Some(&bksp()));
    assert_eq!(strokes.last(), Some(&unicode("ab")));
    let told = [Piece::Erase, Piece::Break, text("ab")];
    assert_eq!(d.pieces(), told, "the ^ breaks the word where it went");
    assert_eq!(d.modes().last().map(|m| m.dead), Some(true), "the ^ waits");
    d.rig.tap(E).unwrap();
    assert_eq!(d.modes().last().map(|m| m.dead), Some(false), "ê ends it");
}
