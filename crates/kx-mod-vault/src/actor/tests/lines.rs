//! The line the keyboard hears (ADR-0040): each change once, the search first as the panel opens, and again on an ask.

use kx_module_api::Redacted;
use kx_module_api::ports::{EntryKind, FillButton, FillClick, PanelKey, PanelLine};

use super::backup::{press, type_password};
use super::{Msg, Rig};

/// The search line, holding `text`.
fn search(text: &str) -> PanelLine {
    PanelLine::Search(Redacted::new(text.to_owned()))
}

#[test]
fn a_begun_actor_has_told_no_line() {
    let rig = Rig::new();
    assert!(rig.lines().is_empty(), "shut, as the keyboard assumes");
}

#[test]
fn the_panel_opening_tells_the_search_after_its_state_and_closing_tells_no_line() {
    let mut rig = Rig::new();
    rig.send(Msg::Open);
    assert_eq!(rig.lines(), [search("")], "before the ui tells its rows");
    assert_eq!(rig.panels(), [true]);
    rig.send(Msg::Open);
    assert_eq!(rig.lines(), [search(""), PanelLine::None]);
}

#[test]
fn a_search_is_told_as_it_changes_and_only_then() {
    let mut rig = Rig::new();
    rig.open();
    rig.text("b");
    rig.send(Msg::Key(PanelKey::Up));
    rig.text("a");
    assert_eq!(rig.lines(), [search(""), search("b"), search("ba")]);
}

#[test]
fn a_caret_move_tells_the_search_before_it_again_and_gives_one_view_each() {
    let mut rig = Rig::new();
    rig.open();
    rig.text("bank");
    let views = rig.views().len();
    rig.send(Msg::Key(PanelKey::Left));
    assert_eq!(rig.lines().last(), Some(&search("ban")));
    rig.click(FillClick::Caret { at: 1.into() });
    assert_eq!(rig.lines().last(), Some(&search("b")));
    assert_eq!(
        rig.views().len(),
        views + 2,
        "the chips follow, and so does the line"
    );
    let told = rig.lines().len();
    rig.send(Msg::Key(PanelKey::Delete));
    assert_eq!(rig.page().search.expose(), "bnk");
    assert_eq!(rig.lines().len(), told, "nothing before the caret changed");
}

#[test]
fn a_chip_acts_at_the_caret_and_prediction_hears_what_it_left_before_it() {
    let mut rig = Rig::new();
    rig.open();
    rig.text("ba x");
    rig.click(FillClick::Caret { at: 2.into() });
    rig.send(Msg::Key(PanelKey::Word {
        erase: 2,
        text: Redacted::new("bank".to_owned()),
    }));
    assert_eq!(rig.page().search.expose(), "bank x");
    assert_eq!(*rig.page().caret.expose(), 4);
    assert_eq!(rig.lines().last(), Some(&search("bank")));
}

#[test]
fn a_chip_gives_one_view_and_one_line_in_english_and_arabic() {
    for (typed, erase, chip, want) in [
        ("ban", 3, "Bank ", "Bank "),
        ("مرحب", 2, "حبا ", "مرحبا "),
        ("ba", 9, "k", "k"),
    ] {
        let mut rig = Rig::new();
        rig.open();
        rig.text(typed);
        let (views, lines) = (rig.views().len(), rig.lines().len());
        rig.send(Msg::Key(PanelKey::Word {
            erase,
            text: Redacted::new(chip.to_owned()),
        }));
        assert_eq!(rig.views().len(), views + 1, "{want}");
        assert_eq!(rig.lines().len(), lines + 1, "{want}");
        assert_eq!(rig.lines().last(), Some(&search(want)));
    }
}

#[test]
fn the_backup_password_page_ignores_a_chip() {
    let mut rig = Rig::new();
    rig.open();
    press(&mut rig, FillButton::Backup);
    rig.jobs();
    rig.click(FillClick::Row {
        row: 0,
        button: kx_module_api::ports::RowButton::Open,
    });
    rig.jobs();
    rig.text("ab");
    let before = rig.page().sheet;
    rig.send(Msg::Key(PanelKey::Word {
        erase: 1,
        text: Redacted::new("x".to_owned()),
    }));
    assert_eq!(rig.page().sheet, before, "no search here");
}

#[test]
fn a_form_tells_private_once_and_leaving_it_tells_the_search() {
    let mut rig = Rig::new();
    rig.open();
    rig.click(FillClick::Button(FillButton::New));
    assert_eq!(rig.lines().last(), Some(&PanelLine::None), "the kind chips");
    rig.click(FillClick::Kind(EntryKind::Note));
    rig.text("Shed");
    let lines = rig.lines();
    assert_eq!(lines.last(), Some(&PanelLine::Private));
    assert_eq!(
        lines.iter().filter(|l| **l == PanelLine::Private).count(),
        1,
        "the name's letters tell nothing"
    );
    rig.click(FillClick::Button(FillButton::Cancel));
    assert_eq!(rig.lines().last(), Some(&search("")));
}

#[test]
fn a_form_caret_tells_no_new_line_and_no_text() {
    let mut rig = Rig::new();
    rig.open();
    rig.click(FillClick::Button(FillButton::New));
    rig.click(FillClick::Kind(EntryKind::Note));
    rig.text("Shed");
    let told = rig.lines().len();
    rig.send(Msg::Key(PanelKey::Left));
    rig.click(FillClick::Caret { at: 0.into() });
    rig.send(Msg::Key(PanelKey::Delete));
    assert_eq!(rig.lines().len(), told, "still private");
}

#[test]
fn the_backup_password_page_tells_private_and_leaving_it_tells_no_line() {
    let mut rig = Rig::new();
    rig.open();
    press(&mut rig, FillButton::Backup);
    rig.jobs();
    assert_eq!(rig.lines().last(), Some(&PanelLine::None));
    rig.click(FillClick::Row {
        row: 0,
        button: kx_module_api::ports::RowButton::Open,
    });
    rig.jobs();
    assert_eq!(rig.lines().last(), Some(&PanelLine::Private));
    type_password(&mut rig, super::backup::KEY);
    rig.jobs();
    assert_eq!(rig.lines().last(), Some(&PanelLine::None));
}

#[test]
fn an_ask_tells_the_line_again_even_unchanged() {
    let mut rig = Rig::new();
    rig.send(Msg::Ask);
    assert_eq!(rig.lines(), [PanelLine::None], "shut");
    rig.open();
    rig.send(Msg::Ask);
    assert_eq!(rig.lines(), [PanelLine::None, search(""), search("")]);
}

#[test]
fn the_stop_tells_no_line_and_nothing_after_it() {
    let mut rig = Rig::new();
    rig.open();
    rig.send(Msg::Quit);
    rig.send(Msg::Ask);
    assert_eq!(rig.lines(), [search(""), PanelLine::None]);
}
