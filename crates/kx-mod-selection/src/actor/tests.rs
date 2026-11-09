//! The actor over the fakes: looks and what hides them (`looks`, `presses`), the keycaps' edits (`acts`), Translate
//! (`translate`) and its consent (`consent`), its logs (`logs`), and properties over any run (`props`); `rig` drives
//! it without a thread on the boxes of `spots`, hearing it through `taps`. Here: the start, the stop, and an inbox that
//! never holds typed text.

#![allow(
    clippy::unwrap_used,
    reason = "these tests and their rig fail the test on a failed setup"
)]

mod acts;
mod consent;
mod deadline;
mod logs;
mod looks;
mod presses;
mod props;
mod rig;
mod spots;
mod taps;
mod translate;

use kx_module_api::ports::{AssistAsk, BoardPlaceAsk, ChipsView, Piece, SelKind, TypingModeAsk};
use kx_module_api::{Event, Redacted};
use rig::Rig;
use spots::sel;
use taps::Heard;

use super::Msg;

#[test]
fn the_start_wants_presses_and_asks_for_the_place_grab_and_the_aim() {
    let rig = Rig::new();
    let want = [
        Heard::Presses(true),
        Heard::Ask(BoardPlaceAsk::NAME),
        Heard::Ask(AssistAsk::NAME),
        Heard::Ask(TypingModeAsk::NAME),
    ];
    assert_eq!(rig.heard(), want);
}

#[test]
fn a_late_mouse_module_hears_the_wish_again() {
    let mut rig = Rig::new();
    rig.send(Msg::PressesAsk);
    assert_eq!(rig.presses(), [true, true]);
}

#[test]
fn the_stop_drops_the_watch_hides_and_lets_presses_go() {
    let mut rig = Rig::new();
    rig.focus(sel(SelKind::Editable, 40));
    rig.send(Msg::Quit);
    assert!(!rig.watch.watching(), "the watch went first");
    let heard = rig.heard();
    let end = &heard[heard.len() - 2..];
    assert_eq!(end, [Heard::View(ChipsView::Hidden), Heard::Presses(false)]);
    let (told, looks) = (rig.count(), rig.watch.calls().looks);
    rig.send(Msg::Wake);
    rig.send(Msg::Quit);
    assert_eq!((rig.count(), rig.watch.calls().looks), (told, looks));
}

#[test]
fn a_typed_piece_reaches_the_inbox_only_as_whether_it_was_a_break() {
    let text = Piece::Text(Redacted::new("zebra-quartz".to_owned()));
    let pieces = [text, Piece::Erase, Piece::Advance, Piece::Break];
    let broke = pieces.map(|p| matches!(Msg::typed(&p), Msg::Typed(true)));
    assert_eq!(broke, [false, false, false, true]);
}
