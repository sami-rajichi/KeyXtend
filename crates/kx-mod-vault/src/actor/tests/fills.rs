//! Fills through the actor: a row's button sends one `FillType` with the Tab wait; the answer of its round gives the
//! note, and a typed one moves the entry's last use and saves without moving a row while open; a fill while one waits
//! is refused; nothing of a value is logged.

use kx_module_api::ports::{FillClick, FillNote, FillPart, FillTyped, RowButton};

use super::super::Msg;
use super::logs::lines_of;
use super::rig::{PASSWORD, Rig, USER};
use crate::settings::VaultSettings;

/// Bank's row on the list: Alex, then Bank.
pub(super) const BANK: u8 = 1;

/// Bank's `button`.
fn bank(button: RowButton) -> FillClick {
    FillClick::Row { row: BANK, button }
}

/// The texts and Tabs the last request types.
fn typed(rig: &Rig) -> Vec<String> {
    let part = |p: &FillPart| match p {
        FillPart::Text(t) => t.text().expose().to_string(),
        FillPart::Tab => "<tab>".to_owned(),
    };
    let last = rig.types().pop().unwrap();
    last.parts.iter().map(part).collect()
}

/// The keyboard answers the last request.
fn answer(rig: &mut Rig, ok: bool) {
    let round = rig.types().last().unwrap().round;
    rig.send(Msg::Typed(FillTyped { round, ok }));
}

/// When Bank was last filled.
fn used(rig: &Rig) -> Option<crate::entry::Stamp> {
    let bank = rig
        .actor
        .book
        .entries()
        .iter()
        .find(|e| e.name().expose() == "Bank");
    bank.and_then(crate::entry::Entry::used)
}

#[test]
fn both_sends_one_request_of_the_user_a_tab_and_the_password_and_draws_again() {
    let mut rig = Rig::new();
    rig.open();
    let views = rig.views().len();
    rig.click(bank(RowButton::Both));
    assert_eq!(typed(&rig), [USER, "<tab>", PASSWORD]);
    let settings = VaultSettings::from_defaults().unwrap();
    assert_eq!(rig.types()[0].tab_gap, settings.tab_gap());
    assert_eq!(rig.views().len(), views + 1, "drawn again after a fill");
    assert_eq!(rig.page().highlight, Some(BANK));
}

#[test]
fn a_typed_fill_says_so_moves_the_last_use_and_saves_and_no_row_moves_till_the_next_open() {
    let mut rig = Rig::new();
    rig.open();
    rig.click(bank(RowButton::Password));
    assert_eq!(typed(&rig), [PASSWORD]);
    let saves = rig.sealed.saves();
    answer(&mut rig, true);
    assert_eq!(rig.note(), Some(FillNote::Typed));
    assert!(used(&rig).is_some());
    assert_eq!(rig.sealed.saves(), saves + 1);
    assert_eq!(rig.names(), ["Alex", "Bank", "Gate"], "rows stay still");
    rig.send(Msg::Open);
    rig.send(Msg::Open);
    assert_eq!(rig.names(), ["Bank", "Alex", "Gate"], "last used first");
    assert_eq!(rig.page().note, None);
}

#[test]
fn a_fill_that_did_not_finish_says_so_and_changes_nothing() {
    let mut rig = Rig::new();
    rig.open();
    rig.click(bank(RowButton::Both));
    let saves = rig.sealed.saves();
    answer(&mut rig, false);
    assert_eq!(rig.note(), Some(FillNote::NotTyped));
    assert_eq!((used(&rig), rig.sealed.saves()), (None, saves));
}

#[test]
fn an_older_rounds_answer_is_dropped() {
    let mut rig = Rig::new();
    rig.open();
    rig.click(bank(RowButton::User));
    let round = rig.types()[0].round;
    let views = rig.views().len();
    rig.send(Msg::Typed(FillTyped {
        round: round.wrapping_sub(1),
        ok: true,
    }));
    assert_eq!((rig.views().len(), used(&rig)), (views, None));
}

#[test]
fn a_fill_while_one_waits_is_refused_with_the_note_and_never_queued() {
    let mut rig = Rig::new();
    rig.open();
    rig.click(bank(RowButton::User));
    rig.click(bank(RowButton::Password));
    assert_eq!(rig.types().len(), 1);
    assert_eq!(rig.note(), Some(FillNote::NotTyped));
    answer(&mut rig, true);
    rig.click(bank(RowButton::Password));
    assert_eq!(rig.types().len(), 2, "free again after the answer");
}

#[test]
fn closing_the_panel_lets_a_keyboard_that_never_answered_block_no_later_fill() {
    let mut rig = Rig::new();
    rig.open();
    rig.click(bank(RowButton::User));
    rig.send(Msg::Open);
    rig.send(Msg::Open);
    rig.click(bank(RowButton::User));
    assert_eq!(rig.types().len(), 2);
}

#[test]
fn a_fill_and_its_answer_log_no_value() {
    let mut rig = Rig::new();
    rig.open();
    let lines = lines_of(|| {
        rig.click(bank(RowButton::Both));
        answer(&mut rig, true);
    });
    let leaks = |l: &String| l.contains(USER) || l.contains(PASSWORD);
    assert!(!lines.iter().any(leaks), "{lines:?}");
    let shown = format!("{:?}", rig.types());
    assert!(
        !shown.contains(USER) && !shown.contains(PASSWORD),
        "{shown}"
    );
}
