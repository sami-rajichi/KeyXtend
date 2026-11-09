//! The caret's Paste (plan edge case 14): a click into an editable box with something on the clipboard shows it, and a
//! key, a press, a caret move or `caret_s` ends it.

use super::*;

#[test]
fn a_click_into_an_editable_box_shows_paste_by_the_caret() {
    let mut rig = Rig::new();
    let (_, chips, spot) = shown(rig.step(click(caret(1, 50))));
    assert_eq!((chips, spot), (vec![Paste], by_caret(50)));
}

#[test]
fn no_caret_paste_without_a_click_a_clipboard_or_an_editable_box() {
    let mut rig = Rig::new();
    assert_eq!(rig.step(look(caret(1, 50))), None, "no click");
    let empty = Input::Look {
        spot: caret(1, 50),
        click: true,
        clip: false,
    };
    assert_eq!(rig.step(empty), None, "an empty clipboard");
    for kind in [SelKind::ReadOnly, SelKind::Terminal, SelKind::Closed] {
        let spot = SelSpot {
            kind,
            ..caret(1, 50)
        };
        assert_eq!(rig.step(click(spot)), None, "{kind:?}");
    }
    let untold = SelSpot {
        caret: None,
        ..caret(1, 50)
    };
    assert_eq!(rig.step(click(untold)), None, "no caret box");
}

#[test]
fn the_caret_paste_needs_its_switch() {
    let off = StripConfig {
        rules: Rules {
            caret_paste: false,
            ..RULES
        },
        ..cfg()
    };
    assert_eq!(Rig::with(off).step(click(caret(1, 50))), None);
}

#[test]
fn the_caret_paste_ends_at_a_key() {
    let mut rig = Rig::new();
    rig.show(click(caret(1, 50)));
    assert_eq!(rig.step(Input::Key), Some(ChipsView::Hidden));
    assert_eq!(rig.step(look(caret(1, 50))), None, "it does not come back");
}

#[test]
fn the_caret_paste_ends_at_the_next_press_and_a_new_click_brings_a_new_one() {
    let mut rig = Rig::new();
    let first = rig.show(click(caret(1, 50)));
    assert_eq!(rig.step(Input::Press), Some(ChipsView::Hidden));
    let (next, _, spot) = shown(rig.step(click(caret(1, 70))));
    assert!(next > first);
    assert_eq!(spot, by_caret(70));
}

#[test]
fn the_caret_paste_stays_on_the_same_caret_and_ends_when_it_moves() {
    let mut rig = Rig::new();
    rig.show(click(caret(1, 50)));
    assert_eq!(rig.step(look(caret(1, 50))), None, "the same caret");
    assert_eq!(rig.step(look(caret(1, 51))), Some(ChipsView::Hidden));
}

#[test]
fn the_caret_paste_ends_in_another_box_or_at_a_selection() {
    let mut rig = Rig::new();
    rig.show(click(caret(1, 50)));
    assert_eq!(rig.step(look(caret(2, 50))), Some(ChipsView::Hidden));
    rig.show(click(caret(1, 50)));
    let (_, chips, _) = shown(rig.step(look(sel(1, SelKind::Editable, 40))));
    assert_eq!(chips, [Copy, Cut, Paste, Translate]);
}

#[test]
fn the_caret_paste_ends_when_its_box_turns_closed() {
    let mut rig = Rig::new();
    rig.show(click(caret(1, 50)));
    let password = SelSpot {
        kind: SelKind::Closed,
        ..caret(1, 50)
    };
    assert_eq!(rig.step(look(password)), Some(ChipsView::Hidden));
}

#[test]
fn the_caret_paste_ends_after_caret_s() {
    let mut rig = Rig::new();
    let start = rig.now;
    rig.show(click(caret(1, 50)));
    assert_eq!(rig.strip.ends(), Some(start.saturating_add(CARET)));
    assert_eq!(rig.wait(CARET.saturating_sub(TICK)), None);
    assert_eq!(rig.wait(TICK), Some(ChipsView::Hidden));
    assert_eq!(rig.strip.ends(), None);
}

#[test]
fn the_caret_paste_ends_while_hidden_and_never_comes_back() {
    let mut rig = Rig::new();
    rig.show(click(caret(1, 50)));
    assert_eq!(rig.step(Input::Aimed(true)), Some(ChipsView::Hidden));
    assert_eq!(rig.step(Input::Aimed(false)), None);
    rig.show(click(caret(1, 50)));
    rig.step(Input::Board(Board::Tray));
    assert_eq!(rig.step(Input::Board(Board::Open)), None);
}
