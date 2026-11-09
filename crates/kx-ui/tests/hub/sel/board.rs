//! The keyboard's place (ADR-0041): the hub tells it at each change of the mode and answers the ask with the mode now;
//! the module owns the show rules, so the hub tells the place alone.

use super::heard;
use crate::Rig;
use kx_module_api::ports::{Board, BoardPlaceAsk};
use kx_ui::Mode;

#[test]
fn each_mode_names_its_place() {
    let places = [Mode::Shown, Mode::Minimised, Mode::Away].map(Mode::board);
    assert_eq!(places, [Board::Open, Board::Minimised, Board::Tray]);
}

#[test]
fn the_place_goes_out_at_each_change_of_the_mode_only() {
    let rig = Rig::new();
    let (_bus, sent, _subs) = heard(&rig);
    rig.hub.keep_mode(Mode::Shown);
    assert!(sent.places().is_empty(), "shown already");
    let steps = [
        (Mode::Minimised, Board::Minimised),
        (Mode::Away, Board::Tray),
        (Mode::Shown, Board::Open),
    ];
    for (mode, place) in steps {
        rig.hub.keep_mode(mode);
        assert_eq!(sent.places(), [place], "{mode:?}");
        rig.hub.keep_mode(mode);
        assert!(sent.places().is_empty(), "{mode:?} again");
    }
}

#[test]
fn the_ask_hears_the_mode_now() {
    let rig = Rig::new();
    let (bus, sent, _subs) = heard(&rig);
    bus.publish(&BoardPlaceAsk).unwrap();
    assert_eq!(sent.places(), [Board::Open], "open until told");
    rig.hub.keep_mode(Mode::Minimised);
    sent.places();
    bus.publish(&BoardPlaceAsk).unwrap();
    assert_eq!(sent.places(), [Board::Minimised]);
}

#[test]
fn a_mode_kept_before_listen_is_told_on_ask() {
    let rig = Rig::new();
    rig.hub.keep_mode(Mode::Away);
    let (bus, sent, _subs) = heard(&rig);
    assert!(sent.places().is_empty(), "the module asks when it starts");
    bus.publish(&BoardPlaceAsk).unwrap();
    assert_eq!(sent.places(), [Board::Tray]);
}
