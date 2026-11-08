//! Copy, Cut and Paste: the box is checked again before any key, the keyboard sends the edit, its yes shows the tick
//! and its no tells the user, and an act or answer for another round does nothing (plan edge cases 6 and 12).

use kx_module_api::keys;
use kx_module_api::ports::{
    ChipAct, ChipsView, Edit, EditAsk, EditDone, SelChip, SelKind, SelSpot,
};

use super::super::Msg;
use super::rig::Rig;
use super::spots::{caret, sel};
use SelChip::{Copy, Cut, Paste};

/// True when the last view ticks `chip` on the strip of `round`.
fn ticked(rig: &Rig, round: u32, chip: SelChip) -> bool {
    matches!(rig.view(), ChipsView::Shown { round: r, acted: Some(c), .. } if r == round && c == chip)
}

#[test]
fn copy_checks_the_box_again_then_asks_the_keyboard_and_ticks_on_its_yes() {
    let mut rig = Rig::new();
    rig.focus(sel(SelKind::Editable, 40));
    let round = rig.round();
    rig.act(Copy);
    assert_eq!(rig.watch.calls().stills, 1);
    let ask = EditAsk {
        round,
        edit: Edit::Copy,
        terminal: false,
    };
    assert_eq!(rig.edits(), [ask]);
    rig.answer(true);
    assert!(ticked(&rig, round, Copy), "{:?}", rig.heard());
    assert!(rig.notices().is_empty());
}

#[test]
fn a_cut_whose_selection_went_hides_and_sends_nothing() {
    let mut rig = Rig::new();
    rig.focus(sel(SelKind::Editable, 40));
    rig.watch.set_still(Some(false));
    rig.act(Cut);
    assert_eq!(rig.view(), ChipsView::Hidden);
    assert!(rig.edits().is_empty() && rig.notices().is_empty());
}

#[test]
fn a_terminal_takes_its_own_keys_and_one_whose_selection_cleared_gets_none() {
    let mut rig = Rig::new();
    rig.focus(sel(SelKind::Terminal, 40));
    rig.act(Copy);
    assert!(rig.edits()[0].terminal, "keys that never stop a program");
    rig.focus(sel(SelKind::Terminal, 60));
    rig.watch.set_still(Some(false));
    rig.act(Copy);
    assert_eq!(rig.edits().len(), 1, "no Ctrl+C reaches the program");
    assert_eq!(rig.view(), ChipsView::Hidden);
}

#[test]
fn paste_looks_again_and_needs_the_same_editable_or_terminal_box() {
    let mut rig = Rig::new();
    rig.focus(sel(SelKind::Editable, 40));
    let looks = rig.watch.calls().looks;
    rig.act(Paste);
    assert_eq!(rig.watch.calls().looks, looks + 1);
    assert_eq!(rig.edits()[0].edit, Edit::Paste);
    rig.focus(sel(SelKind::Terminal, 60));
    rig.watch.change_unheard(SelSpot {
        kind: SelKind::ReadOnly,
        ..sel(SelKind::Terminal, 60)
    });
    rig.act(Paste);
    assert_eq!(rig.edits().len(), 1, "a box that can take no paste");
    assert_eq!(rig.view(), ChipsView::Hidden);
}

/// A box with no id (0) cannot be told from another, so its fresh look never confirms it.
#[test]
fn paste_on_a_box_with_no_id_sends_nothing() {
    let mut rig = Rig::new();
    rig.change(sel(SelKind::Editable, 40));
    assert_eq!(rig.watch.spot().box_id, 0, "no focus gave it an id");
    assert_ne!(rig.view(), ChipsView::Hidden, "its strip shows");
    rig.act(Paste);
    assert!(rig.edits().is_empty());
    assert_eq!(rig.view(), ChipsView::Hidden);
}

#[test]
fn paste_on_a_box_focus_left_sends_nothing() {
    let mut rig = Rig::new();
    rig.focus(sel(SelKind::Editable, 40));
    let round = rig.round();
    rig.watch.focus(sel(SelKind::Editable, 40));
    rig.send(Msg::Act(ChipAct { round, chip: Paste }));
    assert!(rig.edits().is_empty());
}

#[test]
fn the_caret_paste_goes_through_the_keyboard() {
    let mut rig = Rig::new();
    rig.watch.set_clip(true);
    rig.focus(caret(50));
    rig.click();
    let round = rig.round();
    rig.act(Paste);
    assert_eq!(rig.edits()[0].edit, Edit::Paste);
    rig.answer(true);
    assert!(ticked(&rig, round, Paste));
}

#[test]
fn a_refused_edit_hides_and_says_the_keys_did_not_go() {
    let mut rig = Rig::new();
    rig.focus(sel(SelKind::Editable, 40));
    rig.act(Copy);
    rig.answer(false);
    assert_eq!(rig.view(), ChipsView::Hidden);
    assert_eq!(rig.keys(), [keys::KEYS_NOT_SENT]);
}

#[test]
fn an_act_or_an_answer_for_another_round_does_nothing() {
    let mut rig = Rig::new();
    rig.focus(sel(SelKind::Terminal, 40));
    let round = rig.round();
    let other = round + 1;
    rig.send(Msg::Act(ChipAct {
        round: other,
        chip: Copy,
    }));
    rig.send(Msg::Act(ChipAct { round, chip: Cut }));
    assert!(
        rig.edits().is_empty(),
        "an old round, or a keycap not shown"
    );
    rig.act(Copy);
    rig.act(Copy);
    assert_eq!(rig.edits().len(), 1, "one edit while it is in flight");
    rig.send(Msg::Done(EditDone {
        round: other,
        ok: true,
    }));
    assert!(rig.notices().is_empty() && !ticked(&rig, round, Copy));
    rig.answer(true);
    assert!(ticked(&rig, round, Copy));
}

#[test]
fn a_failed_edit_for_an_older_strip_still_tells_the_user() {
    let mut rig = Rig::new();
    rig.focus(sel(SelKind::Editable, 40));
    let old = rig.round();
    rig.act(Copy);
    rig.focus(sel(SelKind::Editable, 60));
    rig.act(Copy);
    rig.send(Msg::Done(EditDone {
        round: old,
        ok: false,
    }));
    assert_eq!(rig.keys(), [keys::KEYS_NOT_SENT]);
    let shows = matches!(rig.view(), ChipsView::Shown { acted: None, .. });
    assert!(shows, "the new strip stays: {:?}", rig.heard());
    rig.answer(true);
    assert!(ticked(&rig, rig.round(), Copy));
}

#[test]
fn any_press_ends_the_edits_swallow() {
    let mut rig = Rig::new();
    rig.focus(sel(SelKind::Editable, 40));
    rig.act(Copy);
    rig.answer(true);
    rig.send(Msg::Pressed(true));
    rig.send(Msg::Released(true));
    rig.broke();
    rig.change(sel(SelKind::Editable, 40));
    rig.shown();
}

#[test]
fn the_edits_own_break_leaves_the_act_put_away() {
    let mut rig = Rig::new();
    rig.focus(sel(SelKind::Editable, 40));
    rig.act(Copy);
    rig.answer(true);
    let told = rig.count();
    rig.broke();
    rig.change(sel(SelKind::Editable, 40));
    assert_eq!(
        rig.count(),
        told,
        "the app's own event for Ctrl+C brings nothing back"
    );
    rig.broke();
    rig.change(sel(SelKind::Editable, 40));
    rig.shown();
}

#[test]
fn another_input_before_the_break_ends_the_swallow() {
    let mut rig = Rig::new();
    rig.watch.set_clip(true);
    rig.focus(sel(SelKind::Editable, 40));
    rig.act(Copy);
    rig.answer(true);
    rig.watch.change_unheard(caret(50));
    rig.click();
    assert_eq!(rig.shown().1, [Paste]);
    rig.broke();
    assert_eq!(
        rig.view(),
        ChipsView::Hidden,
        "the break counts as a key now"
    );
}
