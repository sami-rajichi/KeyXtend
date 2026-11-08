//! Clicks on the strip: a pick names the live strip's box, an act shows its tick on the same round and ends the strip,
//! and a click on an old round does nothing.

use super::*;

#[test]
fn a_pick_names_the_box_and_its_kind_on_the_live_round() {
    let mut rig = Rig::new();
    let round = rig.show(look(sel(7, SelKind::Terminal, 40)));
    let pick = Pick {
        chip: Copy,
        box_id: 7,
        kind: SelKind::Terminal,
    };
    assert_eq!(rig.strip.pick(round, Copy), Some(pick));
    assert_eq!(
        rig.strip.pick(round, Cut),
        None,
        "not on a terminal's strip"
    );
    assert_eq!(rig.strip.pick(round.wrapping_sub(1), Copy), None);
    rig.step(Input::Esc);
    assert_eq!(rig.strip.pick(round, Copy), None, "hidden");
}

#[test]
fn a_caret_paste_pick_names_its_editable_box() {
    let mut rig = Rig::new();
    let round = rig.show(click(caret(3, 50)));
    let want = Pick {
        chip: Paste,
        box_id: 3,
        kind: SelKind::Editable,
    };
    assert_eq!(rig.strip.pick(round, Paste), Some(want));
    assert_eq!(rig.strip.pick(round, Copy), None);
}

#[test]
fn acting_shows_the_tick_on_the_same_round_then_ends_the_strip() {
    let mut rig = Rig::new();
    let round = rig.show(look(sel(1, SelKind::Editable, 40)));
    let ticked = rig.step(Input::Acted { round, chip: Cut });
    let want = ChipsView::Shown {
        round,
        chips: vec![Copy, Cut, Paste, Translate],
        spot: on_line(40),
        acted: Some(Cut),
    };
    assert_eq!(ticked, Some(want));
    assert_eq!(rig.strip.view(), ChipsView::Hidden);
    assert_eq!(rig.strip.pick(round, Copy), None);
    assert_eq!(
        rig.step(look(sel(1, SelKind::Editable, 40))),
        None,
        "the same selection"
    );
    assert_eq!(
        rig.step(look(caret(1, 50))),
        None,
        "the tick fades by itself"
    );
    assert!(rig.show(look(sel(1, SelKind::Editable, 40))) > round);
}

#[test]
fn an_act_on_an_old_round_or_a_chip_not_shown_does_nothing() {
    let mut rig = Rig::new();
    let old = rig.show(look(sel(1, SelKind::ReadOnly, 40)));
    let live = rig.show(look(sel(1, SelKind::ReadOnly, 60)));
    assert_eq!(
        rig.step(Input::Acted {
            round: old,
            chip: Copy
        }),
        None
    );
    assert_eq!(
        rig.step(Input::Acted {
            round: live,
            chip: Cut
        }),
        None
    );
    assert!(matches!(rig.strip.view(), ChipsView::Shown { round, .. } if round == live));
}

#[test]
fn an_acted_caret_paste_ends_and_its_timer_with_it() {
    let mut rig = Rig::new();
    let round = rig.show(click(caret(1, 50)));
    let ticked = rig.step(Input::Acted { round, chip: Paste });
    assert!(matches!(
        ticked,
        Some(ChipsView::Shown {
            acted: Some(Paste),
            ..
        })
    ));
    assert_eq!(rig.strip.ends(), None);
    assert_eq!(rig.wait(CARET), None);
    assert_eq!(rig.step(look(caret(1, 50))), None);
}

#[test]
fn an_act_while_hidden_does_nothing() {
    let mut rig = Rig::new();
    let round = rig.show(look(sel(1, SelKind::Editable, 40)));
    rig.step(Input::Aimed(true));
    assert_eq!(rig.step(Input::Acted { round, chip: Copy }), None);
}
