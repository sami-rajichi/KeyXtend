//! The mouse and the keys: a release looks, a press outside hides, Grab's carry is no look, and the caret's Paste
//! shows after a click and ends on a key, a press, a caret move or `caret_s` (plan edge case 14).

use std::time::Duration;

use kx_module_api::ports::{ChipsView, Piece, SelChip, SelKind};

use super::super::Msg;
use super::rig::Rig;
use super::spots::{caret, sel};
use crate::settings::SelSettings;

/// The smallest step of time.
const TICK: Duration = Duration::from_micros(1);

/// How long the caret's Paste stays by default.
fn stay() -> Duration {
    SelSettings::from_defaults().unwrap().caret()
}

#[test]
fn a_release_outside_looks_even_when_the_app_told_nothing() {
    let mut rig = Rig::new();
    rig.watch.change_unheard(sel(SelKind::Editable, 40));
    assert!(rig.views().is_empty());
    rig.click();
    rig.shown();
}

#[test]
fn a_press_outside_hides_and_a_click_on_our_windows_is_no_look() {
    let mut rig = Rig::new();
    rig.focus(sel(SelKind::Editable, 40));
    let looks = rig.watch.calls().looks;
    rig.send(Msg::Pressed(true));
    rig.send(Msg::Released(true));
    assert_eq!(rig.watch.calls().looks, looks, "a click on the keyboard");
    rig.shown();
    rig.send(Msg::Pressed(false));
    assert_eq!(rig.view(), ChipsView::Hidden);
}

#[test]
fn a_drag_select_that_ends_over_our_keyboard_still_looks() {
    let mut rig = Rig::new();
    rig.send(Msg::Pressed(false));
    rig.watch.change_unheard(sel(SelKind::Editable, 40));
    rig.send(Msg::Released(true));
    rig.shown();
}

#[test]
fn releases_are_no_looks_while_grab_carries() {
    let mut rig = Rig::new();
    rig.send(Msg::Carrying(true));
    rig.watch.change_unheard(sel(SelKind::Editable, 40));
    rig.click();
    assert_eq!(rig.watch.calls().looks, 0);
    rig.send(Msg::Carrying(false));
    rig.send(Msg::Released(false));
    rig.shown();
}

#[test]
fn a_selection_later_than_its_release_shows_on_the_apps_own_event() {
    let mut rig = Rig::new();
    rig.click();
    assert!(rig.views().is_empty(), "the app had not selected yet");
    rig.change(sel(SelKind::Editable, 40));
    rig.shown();
}

#[test]
fn a_click_at_a_bare_caret_shows_paste_when_the_clipboard_holds_anything() {
    let mut rig = Rig::new();
    rig.watch.set_clip(true);
    rig.focus(caret(50));
    assert!(rig.views().is_empty(), "no click came before");
    rig.click();
    assert_eq!(rig.shown().1, [SelChip::Paste]);
    assert_eq!(rig.watch.calls().reads, 0, "only the formats are checked");
}

#[test]
fn an_empty_clipboard_shows_no_caret_paste() {
    let mut rig = Rig::new();
    rig.focus(caret(50));
    rig.click();
    assert!(rig.views().is_empty());
    assert_eq!(rig.watch.calls().clips, 1);
}

/// A rig showing the caret's Paste at `x`.
fn pasting(x: i32) -> Rig {
    let mut rig = Rig::new();
    rig.watch.set_clip(true);
    rig.focus(caret(x));
    rig.click();
    rig.shown();
    rig
}

#[test]
fn the_caret_paste_ends_on_a_key_or_our_esc_as_a_break() {
    let mut rig = pasting(50);
    rig.letter();
    assert_eq!(rig.view(), ChipsView::Hidden);
    let mut rig = pasting(50);
    rig.typed(Piece::Break);
    assert_eq!(rig.view(), ChipsView::Hidden, "Esc comes as a break");
}

#[test]
fn the_caret_paste_ends_on_a_press_or_a_caret_move() {
    let mut rig = pasting(50);
    rig.send(Msg::Pressed(false));
    assert_eq!(rig.view(), ChipsView::Hidden);
    let mut rig = pasting(50);
    rig.change(caret(60));
    assert_eq!(rig.view(), ChipsView::Hidden);
}

#[test]
fn the_caret_paste_ends_after_caret_s_on_the_clock() {
    let mut rig = pasting(50);
    assert_eq!(rig.wait(), Some(stay()));
    rig.advance(stay().saturating_sub(TICK));
    rig.shown();
    rig.advance(TICK);
    assert_eq!(rig.view(), ChipsView::Hidden);
    assert_eq!(rig.wait(), None, "nothing left to wait for");
}
