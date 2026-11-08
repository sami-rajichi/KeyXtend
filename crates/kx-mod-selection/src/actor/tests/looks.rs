//! Looks: a selection tells the strip, the watch's tell only wakes the actor, a failed look hides, a closed box shows
//! nothing, and an aimed line, the tray or the bubble block the strip while the looks go on (plan edge cases 7, 8, 13).

use kx_module_api::ports::{Board, ChipsView, SelChip, SelError, SelKind};
use toml::Value;

use super::super::Msg;
use super::rig::Rig;
use super::spots::{line, sel};
use SelChip::{Copy, Cut, Paste, Translate};

#[test]
fn a_selection_tells_the_chips_view_and_the_same_look_again_tells_nothing() {
    let mut rig = Rig::new();
    rig.focus(sel(SelKind::Editable, 40));
    let (round, chips) = rig.shown();
    assert_eq!(chips, [Copy, Cut, Paste, Translate]);
    rig.change(sel(SelKind::Editable, 40));
    assert_eq!((rig.views().len(), rig.round()), (1, round));
}

#[test]
fn the_watchs_tell_only_wakes_and_the_look_runs_on_the_actors_turn() {
    let mut rig = Rig::new();
    rig.watch.focus(sel(SelKind::ReadOnly, 40));
    assert_eq!(rig.watch.calls().looks, 0, "the tell looked at nothing");
    assert!(rig.views().is_empty());
    rig.pump();
    assert_eq!(rig.watch.calls().looks, 1);
    assert_eq!(rig.shown().1, [Copy, Translate]);
}

#[test]
fn a_terminal_never_shows_cut_and_a_closed_box_shows_nothing_and_is_never_read() {
    let mut rig = Rig::new();
    rig.focus(sel(SelKind::Terminal, 40));
    assert_eq!(rig.shown().1, [Copy, Paste, Translate]);
    rig.focus(sel(SelKind::Closed, 40));
    assert_eq!(
        rig.view(),
        ChipsView::Hidden,
        "a password box or our own window"
    );
    let calls = rig.watch.calls();
    assert_eq!((calls.reads, calls.stills, calls.clips), (0, 0, 0));
}

#[test]
fn a_failed_look_hides_until_the_app_answers_again() {
    let mut rig = Rig::new();
    rig.focus(sel(SelKind::Editable, 40));
    rig.watch.fail_looks(SelError::TimedOut);
    rig.change(sel(SelKind::Editable, 60));
    assert_eq!(rig.view(), ChipsView::Hidden, "a hung app");
    rig.watch.heal();
    rig.change(sel(SelKind::Editable, 60));
    rig.shown();
}

#[test]
fn an_aimed_panel_line_hides_and_letting_go_shows_the_last_look_with_no_new_one() {
    let mut rig = Rig::new();
    rig.focus(sel(SelKind::Editable, 40));
    rig.send(Msg::Aimed(true));
    assert_eq!(rig.view(), ChipsView::Hidden);
    rig.change(sel(SelKind::Editable, 80));
    assert_eq!(rig.view(), ChipsView::Hidden, "the looks go on, hidden");
    let looks = rig.watch.calls().looks;
    rig.send(Msg::Aimed(false));
    assert_eq!(rig.watch.calls().looks, looks, "no fresh look");
    let ChipsView::Shown { spot, .. } = rig.view() else {
        panic!("the strip is back: {:?}", rig.heard());
    };
    assert_eq!(spot.first, line(80), "where the last look saw it");
}

#[test]
fn the_tray_hides_and_the_bubble_follows_minimised() {
    let mut rig = Rig::new();
    rig.focus(sel(SelKind::Editable, 40));
    let round = rig.round();
    rig.send(Msg::Board(Board::Minimised));
    assert_eq!(rig.shown().0, round, "minimised = true keeps it");
    rig.send(Msg::Board(Board::Tray));
    assert_eq!(rig.view(), ChipsView::Hidden);
    rig.send(Msg::Board(Board::Open));
    rig.shown();
    let mut rig = Rig::build(|h| h.put("minimised", Value::Boolean(false)));
    rig.focus(sel(SelKind::Editable, 40));
    rig.send(Msg::Board(Board::Minimised));
    assert_eq!(rig.view(), ChipsView::Hidden);
}

#[test]
fn a_keycap_switched_off_does_not_show() {
    let mut rig = Rig::build(|h| h.put("cut", Value::Boolean(false)));
    rig.focus(sel(SelKind::Editable, 40));
    assert_eq!(rig.shown().1, [Copy, Paste, Translate]);
}
