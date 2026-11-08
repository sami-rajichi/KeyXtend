//! What hides the strip and what brings it back: a press, Esc, a lost look, an aimed panel line, the tray, the
//! bubble by `minimised`, and the settings (plan edge cases 7 and 13).

use super::*;

#[test]
fn a_press_hides_and_the_same_selection_stays_hidden() {
    let mut rig = Rig::new();
    let first = rig.show(look(sel(1, SelKind::Editable, 40)));
    assert_eq!(rig.step(Input::Press), Some(ChipsView::Hidden));
    assert_eq!(rig.step(click(sel(1, SelKind::Editable, 40))), None);
    let new = rig.show(look(sel(1, SelKind::Editable, 60)));
    assert!(new > first, "a new selection shows");
}

#[test]
fn a_dismissed_selection_shows_again_once_it_went_and_came_back() {
    let mut rig = Rig::new();
    rig.show(look(sel(1, SelKind::Editable, 40)));
    rig.step(Input::Esc);
    assert_eq!(rig.step(look(caret(1, 50))), None);
    rig.show(look(sel(1, SelKind::Editable, 40)));
}

#[test]
fn esc_hides_the_strip() {
    let mut rig = Rig::new();
    rig.show(look(sel(1, SelKind::ReadOnly, 40)));
    assert_eq!(rig.step(Input::Esc), Some(ChipsView::Hidden));
    assert_eq!(rig.step(look(sel(1, SelKind::ReadOnly, 40))), None);
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

#[test]
fn the_helper_off_shows_nothing_and_turning_it_off_hides() {
    let off = StripConfig {
        rules: Rules { on: false, ..RULES },
        ..cfg()
    };
    let mut rig = Rig::with(off);
    assert_eq!(rig.step(look(sel(1, SelKind::Editable, 40))), None);
    assert_eq!(rig.step(click(caret(1, 50))), None);
    let mut rig = Rig::new();
    rig.show(look(sel(1, SelKind::Editable, 40)));
    assert_eq!(rig.strip.configure(off, rig.now), Some(ChipsView::Hidden));
    let back = rig.strip.configure(cfg(), rig.now);
    assert_eq!(shown(back).2, on_line(40));
}

#[test]
fn a_new_setting_redraws_the_keycaps() {
    let mut rig = Rig::new();
    let first = rig.show(look(sel(1, SelKind::Editable, 40)));
    let no_cut = StripConfig {
        rules: Rules {
            cut: false,
            ..RULES
        },
        ..cfg()
    };
    let (round, chips, _) = shown(rig.strip.configure(no_cut, rig.now));
    assert!(round > first);
    assert_eq!(chips, [Copy, Paste, Translate]);
}
