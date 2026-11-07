//! The actor over the fakes: fills and their answers (`fills`), Paste (`pastes`); `rig` drives it without a thread.
//! Here: the panel's life, the views held until the ui tells its rows, and the emails told.

#![allow(
    clippy::unwrap_used,
    reason = "these tests and their rig fail the test on a failed setup"
)]

mod backup;
mod boxes;
mod ears;
mod fills;
mod lines;
mod logs;
mod pastes;
mod rig;
mod saves;

use kx_module_api::ports::{
    EntryKind, FieldButton, FillAct, FillButton, FillClick, FillNote, SealedError,
};

use super::Msg;
use crate::find::LabelNames;
use rig::{EMAIL, Rig, sample};

#[test]
fn it_tells_the_emails_as_it_begins_and_the_panel_shut_and_no_emails_as_it_quits() {
    let mut rig = Rig::new();
    assert_eq!(rig.emails(), [vec![EMAIL.to_owned()]]);
    rig.open();
    rig.send(Msg::Quit);
    assert_eq!(rig.panels(), [true, false]);
    assert_eq!(rig.emails().last(), Some(&Vec::new()));
    rig.send(Msg::Open);
    assert_eq!(rig.panels(), [true, false], "nothing after the stop");
}

#[test]
fn it_says_it_runs_as_it_begins_and_on_each_ask_and_that_it_stopped_as_it_quits() {
    let mut rig = Rig::new();
    rig.send(Msg::Ask);
    rig.send(Msg::Quit);
    rig.send(Msg::Ask);
    assert_eq!(rig.lives(), [true, true, false], "nothing after the stop");
}

#[test]
fn no_view_shows_until_the_ui_tells_its_rows() {
    let mut rig = Rig::new();
    rig.send(Msg::Open);
    assert_eq!(rig.panels(), [true]);
    assert!(rig.views().is_empty());
    rig.send(Msg::Act(FillAct {
        round: 0,
        click: FillClick::Rows(rig::ROWS),
    }));
    assert_eq!(rig.views().len(), 1);
    assert_eq!(rig.names(), ["Alex", "Bank", "Gate"]);
}

#[test]
fn fill_ask_answers_the_panel_state_and_the_emails() {
    let mut rig = Rig::new();
    rig.send(Msg::Ask);
    assert_eq!(rig.panels(), [false]);
    assert_eq!(rig.emails().len(), 2, "told again");
    rig.open();
    rig.send(Msg::Ask);
    assert_eq!(rig.panels(), [false, true, true]);
}

#[test]
fn the_toggle_and_the_clipboards_close_shut_the_panel_and_say_so() {
    let mut rig = Rig::new();
    rig.open();
    rig.send(Msg::Open);
    rig.open();
    rig.send(Msg::Close);
    rig.send(Msg::Close);
    assert_eq!(
        rig.panels(),
        [true, false, true, false],
        "a late close does nothing"
    );
}

#[test]
fn a_save_that_alters_the_emails_tells_them_again_and_one_that_does_not_tells_nothing() {
    let mut rig = Rig::with(sample());
    rig.open();
    let saves = rig.sealed.saves();
    rig.click(FillClick::Button(FillButton::New));
    rig.click(FillClick::Kind(EntryKind::Note));
    rig.text("Shed");
    rig.click(FillClick::Button(FillButton::Save));
    assert_eq!(rig.emails().len(), 1, "a note holds no email");
    assert_eq!(rig.sealed.saves(), saves + 1);
    rig.click(FillClick::Button(FillButton::New));
    rig.click(FillClick::Kind(EntryKind::Person));
    rig.text("Sam");
    rig.click(FillClick::Field {
        field: 1,
        button: FieldButton::Value,
    });
    rig.text("sam@example.com");
    rig.click(FillClick::Button(FillButton::Save));
    let want = vec![EMAIL.to_owned(), "sam@example.com".to_owned()];
    assert_eq!(rig.emails().last(), Some(&want));
    assert_eq!(rig.sealed.saves(), saves + 2);
}

#[test]
fn a_failed_save_logs_its_kind_once_says_so_and_is_tried_again_at_the_stop() {
    let mut rig = Rig::new();
    rig.open();
    rig.sealed.fail_save(SealedError::Io);
    rig.click(FillClick::Button(FillButton::New));
    rig.click(FillClick::Kind(EntryKind::Note));
    rig.text("Shed");
    let lines = logs::lines_of(|| rig.click(FillClick::Button(FillButton::Save)));
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert!(lines.iter().all(|l| !l.contains("Shed")), "{lines:?}");
    assert_eq!(rig.note(), Some(FillNote::SaveFailed));
    let saves = rig.sealed.saves();
    rig.send(Msg::Quit);
    assert_eq!(rig.sealed.saves(), saves + 1, "tried again at the stop");
}

#[test]
fn the_label_names_the_ui_gives_reach_the_search() {
    let mut rig = Rig::new();
    rig.open();
    rig.text("courriel");
    assert!(rig.names().is_empty(), "no names heard yet");
    let names = LabelNames::from([("email".to_owned(), "Courriel".to_owned())]);
    rig.send(Msg::Names(names));
    assert_eq!(rig.names(), ["Alex"]);
}
