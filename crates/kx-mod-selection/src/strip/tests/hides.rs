//! What hides the strip and what brings it back: a press, our Esc as the actor feeds it, an act's put-away, a lost look,
//! an aimed panel line, the tray, the bubble by `minimised`, and the switches the module starts with (plan edge cases 7
//! and 13).

use super::*;

#[test]
fn a_press_hides_until_the_next_look_decides() {
    let mut rig = Rig::new();
    let first = rig.show(look(sel(1, SelKind::Editable, 40)));
    assert_eq!(rig.step(Input::Press), Some(ChipsView::Hidden));
    assert_eq!(rig.step(Input::Tick), None, "no look yet");
    let again = rig.show(click(sel(1, SelKind::Editable, 40)));
    assert!(again > first, "the same text selected again");
}

/// Our Esc comes as a break, which the actor feeds as a key, then a lost look.
#[test]
fn our_esc_as_a_key_and_a_lost_look_hides_until_the_next_look() {
    let mut rig = Rig::new();
    let first = rig.show(look(sel(1, SelKind::ReadOnly, 40)));
    assert_eq!(rig.step(Input::Key), None, "the next look decides");
    assert_eq!(rig.step(Input::Lost), Some(ChipsView::Hidden));
    let again = rig.show(look(sel(1, SelKind::ReadOnly, 40)));
    assert!(again > first, "the app's own event brings it back");
}

#[test]
fn an_acted_selection_shows_again_once_it_went_and_came_back() {
    let mut rig = Rig::new();
    let round = rig.show(look(sel(1, SelKind::Editable, 40)));
    rig.step(Input::Acted { round, chip: Copy });
    assert_eq!(rig.step(look(caret(1, 50))), None);
    rig.show(look(sel(1, SelKind::Editable, 40)));
}

#[test]
fn an_act_puts_the_selection_away_while_its_box_holds_one() {
    let mut rig = Rig::new();
    let round = rig.show(look(sel(1, SelKind::Editable, 40)));
    rig.step(Input::Acted { round, chip: Copy });
    assert_eq!(
        rig.step(look(sel(1, SelKind::Editable, 90))),
        None,
        "scrolled"
    );
    let off = SelSpot {
        first: None,
        last: None,
        ..sel(1, SelKind::Editable, 40)
    };
    assert_eq!(rig.step(look(off)), None, "scrolled off");
    assert_eq!(
        rig.step(look(sel(1, SelKind::Editable, 60))),
        None,
        "and back"
    );
    assert_eq!(rig.step(Input::Press), None);
    rig.show(click(sel(1, SelKind::Editable, 60)));
}

#[test]
fn another_box_ends_an_acts_put_away() {
    let mut rig = Rig::new();
    let round = rig.show(look(sel(1, SelKind::Editable, 40)));
    rig.step(Input::Acted { round, chip: Cut });
    rig.show(look(sel(2, SelKind::Editable, 40)));
    rig.show(look(sel(1, SelKind::Editable, 40)));
}

#[test]
fn a_lost_look_hides_and_the_next_good_one_shows() {
    let mut rig = Rig::new();
    let first = rig.show(look(sel(1, SelKind::Editable, 40)));
    assert_eq!(rig.step(Input::Lost), Some(ChipsView::Hidden));
    assert_eq!(rig.step(Input::Lost), None);
    assert!(rig.show(look(sel(1, SelKind::Editable, 40))) > first);
}

#[test]
fn an_aimed_panel_line_hides_the_strip_until_let_go() {
    let mut rig = Rig::new();
    let first = rig.show(look(sel(1, SelKind::Editable, 40)));
    assert_eq!(rig.step(Input::Aimed(true)), Some(ChipsView::Hidden));
    assert_eq!(rig.step(look(sel(1, SelKind::Editable, 60))), None);
    let (back, _, spot) = shown(rig.step(Input::Aimed(false)));
    assert!(back > first);
    assert_eq!(spot, on_line(60), "the last look while aimed");
}

#[test]
fn the_tray_hides_and_the_keyboards_return_shows_again() {
    let mut rig = Rig::new();
    rig.show(look(sel(1, SelKind::Editable, 40)));
    assert_eq!(rig.step(Input::Board(Board::Tray)), Some(ChipsView::Hidden));
    assert_eq!(rig.step(look(sel(1, SelKind::Editable, 40))), None);
    rig.show(Input::Board(Board::Open));
}

#[test]
fn minimised_follows_its_setting() {
    let mut rig = Rig::new();
    rig.show(look(sel(1, SelKind::Editable, 40)));
    let min = rig.step(Input::Board(Board::Minimised));
    assert_eq!(min, None, "still shown");
    let hide = StripConfig {
        minimised: false,
        ..cfg()
    };
    let mut rig = Rig::with(hide);
    rig.show(look(sel(1, SelKind::Editable, 40)));
    let min = rig.step(Input::Board(Board::Minimised));
    assert_eq!(min, Some(ChipsView::Hidden));
    rig.show(Input::Board(Board::Open));
}

/// A settings change restarts the module, so the strip only ever follows the switches it started with.
#[test]
fn the_helper_off_shows_nothing() {
    let off = StripConfig {
        rules: Rules { on: false, ..RULES },
        ..cfg()
    };
    let mut rig = Rig::with(off);
    assert_eq!(rig.step(look(sel(1, SelKind::Editable, 40))), None);
    assert_eq!(rig.step(click(caret(1, 50))), None);
}

#[test]
fn a_keycap_switched_off_leaves_the_strip() {
    let no_cut = StripConfig {
        rules: Rules {
            cut: false,
            ..RULES
        },
        ..cfg()
    };
    let mut rig = Rig::with(no_cut);
    let (_, chips, _) = shown(rig.step(look(sel(1, SelKind::Editable, 40))));
    assert_eq!(chips, [Copy, Paste, Translate]);
}
