//! A failed save over the actor: the note line says so until a save works, on every page and after the panel opens
//! again, and another note takes the line meanwhile; the notebook and the backup's blob each count, and each is tried
//! again every `place_check_s`.

use std::sync::Arc;
use std::time::Duration;

use kx_module_api::keys;
use kx_module_api::ports::{
    BlobName, FillButton, FillClick, FillFilter, FillNote, FillTyped, RowButton, SealedError,
};
use kx_platform_fake::FakeSealedStore;

use super::super::Msg;
use super::backup::{CLOUD_ROW, kept, press, save_note, set_up, waits};
use super::fills::BANK;
use super::logs;
use super::rig::Rig;
use crate::settings::VaultSettings;
use crate::store::{BLOB, Store};

/// A millisecond, the least the fake clock moves.
const MS: Duration = Duration::from_millis(1);

/// An open rig whose last notebook save failed.
fn failed() -> Rig {
    let mut rig = Rig::new();
    rig.open();
    rig.sealed.fail_save(SealedError::Io);
    save_note(&mut rig, "Shed");
    rig
}

/// Answers the memory check's banner with Later.
fn later(rig: &mut Rig) {
    rig.send(Msg::Answered {
        key: keys::VAULT_CHECK,
        yes: false,
    });
}

#[test]
fn the_note_stays_over_other_clicks_until_a_save_works() {
    let mut rig = failed();
    assert_eq!(rig.note(), Some(FillNote::SaveFailed));
    press(&mut rig, FillButton::Back);
    rig.click(FillClick::Filter(FillFilter::Notes));
    assert_eq!(
        rig.note(),
        Some(FillNote::SaveFailed),
        "clicks change no save"
    );
    save_note(&mut rig, "Barn");
    assert_eq!(rig.note(), None, "the next save worked");
}

#[test]
fn another_note_takes_the_line_while_a_save_still_fails() {
    let mut rig = failed();
    press(&mut rig, FillButton::Back);
    rig.click(FillClick::Row {
        row: BANK,
        button: RowButton::Password,
    });
    let round = rig.types().last().unwrap().round;
    rig.send(Msg::Typed(FillTyped { round, ok: false }));
    assert_eq!(rig.note(), Some(FillNote::NotTyped));
    rig.click(FillClick::Filter(FillFilter::Notes));
    assert_eq!(rig.note(), Some(FillNote::SaveFailed), "still not saved");
}

#[test]
fn the_note_shows_on_the_backup_pages_and_after_the_panel_opens_again() {
    let mut rig = failed();
    press(&mut rig, FillButton::Backup);
    rig.jobs();
    assert_eq!(rig.note(), Some(FillNote::SaveFailed), "the Places page");
    rig.send(Msg::Open);
    rig.send(Msg::Open);
    assert_eq!(rig.note(), Some(FillNote::SaveFailed), "opened again");
}

#[test]
fn a_failed_blob_save_says_so_until_the_next_change_saves_it() {
    let mut rig = Rig::new();
    set_up(&mut rig, CLOUD_ROW);
    save_note(&mut rig, "Shed");
    rig.advance(waits().0);
    rig.sealed.fail_save(SealedError::Io);
    rig.jobs();
    assert_eq!(rig.note(), Some(FillNote::SaveFailed));
    save_note(&mut rig, "Barn");
    assert_eq!(rig.note(), None, "the change saved the blob too");
}

#[test]
fn a_change_tries_a_failed_blob_save_again_though_no_write_is_wanted() {
    let mut rig = Rig::new();
    set_up(&mut rig, CLOUD_ROW);
    rig.click(FillClick::Row {
        row: 0,
        button: RowButton::Remove,
    });
    rig.sealed.fail_save(SealedError::Io);
    rig.click(FillClick::Answer(true));
    assert_eq!(
        rig.note(),
        Some(FillNote::SaveFailed),
        "the last place left"
    );
    save_note(&mut rig, "Shed");
    assert_eq!(rig.note(), None, "no place, so only the retry saves it");
}

#[test]
fn a_save_that_fails_or_works_outside_a_click_draws_the_note_line_again() {
    let settings = VaultSettings::from_defaults().unwrap();
    let mut rig = Rig::new();
    set_up(&mut rig, CLOUD_ROW);
    rig.pass(settings.check_waits()[0]);
    rig.sealed.fail_save(SealedError::Io);
    later(&mut rig);
    assert_eq!(rig.note(), Some(FillNote::SaveFailed));
    rig.pass(settings.later());
    later(&mut rig);
    assert_eq!(rig.note(), None);
}

#[test]
fn a_failed_save_is_tried_again_every_place_check_s_and_logged_once() {
    let every = waits().1;
    let mut rig = Rig::new();
    rig.open();
    rig.sealed.fail_save(SealedError::Io);
    let lines = logs::lines_of(|| {
        save_note(&mut rig, "Shed");
        assert_eq!(rig.actor.wait(), Some(every), "it wakes for the retry");
        rig.sealed.fail_save(SealedError::Io);
        rig.advance(every);
    });
    assert_eq!(rig.note(), Some(FillNote::SaveFailed), "that retry failed");
    assert_eq!(lines.len(), 1, "logged once: {lines:?}");
    assert_eq!(rig.actor.wait(), Some(every), "the next retry");
    rig.advance(every.saturating_sub(MS));
    assert_eq!(rig.note(), Some(FillNote::SaveFailed), "not yet");
    assert_eq!(rig.actor.wait(), Some(MS));
    rig.advance(MS);
    assert_eq!(rig.note(), None, "it worked, and the line drew again");
    assert_eq!(rig.actor.wait(), None, "nothing more to wake for");
}

#[test]
fn a_failed_blob_save_is_tried_again_every_place_check_s() {
    let (delay, every) = waits();
    let mut rig = Rig::new();
    set_up(&mut rig, CLOUD_ROW);
    save_note(&mut rig, "Shed");
    rig.advance(delay);
    rig.sealed.fail_save(SealedError::Io);
    rig.jobs();
    assert_eq!(rig.note(), Some(FillNote::SaveFailed));
    assert!(kept(&rig).behind, "the write's answer did not save");
    assert_eq!(rig.actor.wait(), Some(every));
    rig.advance(every);
    assert_eq!(rig.note(), None);
    assert!(!kept(&rig).behind, "the retry saved it");
}

#[test]
fn a_retry_tries_both_the_notebook_and_the_blob() {
    let (delay, every) = waits();
    let mut rig = Rig::new();
    set_up(&mut rig, CLOUD_ROW);
    let notebook = FakeSealedStore::new();
    rig.actor.store = Store::new(Arc::new(notebook.clone()));
    notebook.fail_save(SealedError::Io);
    rig.sealed.fail_save(SealedError::Io);
    save_note(&mut rig, "Shed");
    assert_eq!(rig.note(), Some(FillNote::SaveFailed));
    assert!(!kept(&rig).behind, "the blob's save failed");
    assert_eq!(rig.actor.wait(), Some(delay), "the write comes first");
    rig.advance(every);
    assert_eq!(rig.note(), None);
    assert_eq!(notebook.saves(), 2, "the failed save and the retry");
    assert!(kept(&rig).behind, "the blob's retry saved it");
}

#[test]
fn a_store_holding_an_unread_notebook_refuses_every_save_and_the_note_stays() {
    let mut rig = Rig::new();
    rig.open();
    let blob = BlobName::new(BLOB).unwrap();
    rig.sealed.plant(&blob, b"not a notebook".to_vec());
    rig.sealed.fail_save(SealedError::Io);
    let mut store = Store::new(Arc::new(rig.sealed.clone()));
    assert!(store.load().unwrap().damaged);
    rig.actor.store = store;
    save_note(&mut rig, "Shed");
    save_note(&mut rig, "Barn");
    assert_eq!(rig.note(), Some(FillNote::SaveFailed));
}
