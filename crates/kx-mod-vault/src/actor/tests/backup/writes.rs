//! The writes over the actor and the fake clock: `box_delay_ms` after the last change, one seal for every place, a
//! place gone or full waiting with the footer and catching up, a failed write, a box too large (`large` has what comes
//! after) and a failed worker.

use std::path::Path;
use std::time::Duration;

use kx_module_api::ports::{BoxError, FillButton, FillNote, PlaceError, PlaceKind};
use kx_platform_fake::{BoxCall, PlaceCall};

use super::super::boxes::{CLOUD, STICK};
use super::super::rig::Rig;
use super::super::rig::sample;
use super::{
    CLOUD_ROW, KEY, box_in, idle, kept, opens, press, rows, save_note, set_up, spot, two_places,
    waits,
};
use crate::actor::Msg;

/// A millisecond, the least the fake clock moves.
const MS: Duration = Duration::from_millis(1);

/// Saves a note and lets the write go, with the stick gone.
fn write_without_the_stick(rig: &mut Rig) {
    rig.fakes.places.vanish(Path::new(STICK));
    save_note(rig, "Shed");
    rig.advance(waits().0);
    rig.jobs();
}

#[test]
fn a_change_writes_after_box_delay_ms_and_each_change_restarts_the_wait() {
    let (mut rig, (delay, _)) = (Rig::new(), waits());
    set_up(&mut rig, CLOUD_ROW);
    let first = box_in(&rig, CLOUD).unwrap();
    save_note(&mut rig, "Shed");
    assert_eq!(rig.actor.wait(), Some(delay));
    rig.advance(delay.saturating_sub(MS));
    save_note(&mut rig, "Barn");
    rig.advance(delay.saturating_sub(MS));
    assert_eq!(
        rig.fakes.queued(),
        0,
        "the second change restarted the wait"
    );
    rig.advance(MS);
    rig.jobs();
    assert_eq!(
        rig.fakes.boxes.calls(BoxCall::Seal),
        2,
        "one seal for the burst"
    );
    let second = box_in(&rig, CLOUD).unwrap();
    assert!(second != first && opens(&second, KEY));
    assert_eq!(
        rig.actor.wait(),
        idle(&rig),
        "nothing more to do but the next check"
    );
}

#[test]
fn one_seal_goes_to_every_place() {
    let mut rig = two_places();
    let seals = rig.fakes.boxes.calls(BoxCall::Seal);
    save_note(&mut rig, "Shed");
    rig.advance(waits().0);
    rig.jobs();
    assert_eq!(rig.fakes.boxes.calls(BoxCall::Seal), seals + 1);
    assert_eq!(box_in(&rig, CLOUD), box_in(&rig, STICK));
}

#[test]
fn a_place_gone_waits_and_the_footer_says_so_on_every_page() {
    let mut rig = two_places();
    let old = box_in(&rig, STICK);
    write_without_the_stick(&mut rig);
    let stick = spot(PlaceKind::Removable, "E: USB");
    assert_eq!(
        rig.page().waiting,
        Some(stick.clone()),
        "the entry page's footer"
    );
    press(&mut rig, FillButton::Backup);
    assert_eq!(
        rows(&rig),
        [("OneDrive".to_owned(), true), ("E: USB".to_owned(), false)]
    );
    assert_eq!(rig.page().waiting, Some(stick));
    assert_eq!(box_in(&rig, STICK), old, "its old box stays");
}

#[test]
fn a_full_place_waits_like_a_gone_one() {
    let mut rig = two_places();
    rig.fakes.places.set_free(Path::new(STICK), 0);
    save_note(&mut rig, "Shed");
    rig.advance(waits().0);
    rig.jobs();
    press(&mut rig, FillButton::Backup);
    assert_eq!(rows(&rig)[1], ("E: USB".to_owned(), false));
}

#[test]
fn a_read_only_place_waits_says_so_and_catches_up_once_it_takes_writes() {
    let mut rig = two_places();
    rig.fakes.places.set_read_only(Path::new(STICK), true);
    save_note(&mut rig, "Shed");
    rig.advance(waits().0);
    rig.jobs();
    assert_eq!(rig.note(), Some(FillNote::ReadOnly));
    press(&mut rig, FillButton::Backup);
    assert_eq!(rows(&rig)[1], ("E: USB".to_owned(), false));
    rig.fakes.places.set_read_only(Path::new(STICK), false);
    rig.advance(waits().1);
    rig.jobs();
    assert_eq!(rows(&rig)[1], ("E: USB".to_owned(), true));
    assert!(opens(&box_in(&rig, STICK).unwrap(), KEY));
}

#[test]
fn a_waiting_place_catches_up_on_the_next_change_and_when_the_panel_opens() {
    let mut rig = two_places();
    write_without_the_stick(&mut rig);
    rig.fakes.places.come_back(Path::new(STICK));
    save_note(&mut rig, "Barn");
    rig.advance(waits().0);
    rig.jobs();
    assert_eq!(rig.page().waiting, None, "caught up on the change");
    write_without_the_stick(&mut rig);
    rig.fakes.places.come_back(Path::new(STICK));
    let (writes, old) = (
        rig.fakes.places.calls(PlaceCall::Write),
        box_in(&rig, STICK),
    );
    rig.send(Msg::Open);
    rig.open();
    rig.jobs();
    assert_eq!(
        rig.fakes.places.calls(PlaceCall::Write),
        writes + 1,
        "only the waiting place"
    );
    let caught = box_in(&rig, STICK).unwrap();
    assert!(Some(&caught) != old.as_ref() && opens(&caught, KEY));
    assert_eq!(rig.page().waiting, None);
}

#[test]
fn a_waiting_place_is_looked_at_every_place_check_s() {
    let (mut rig, (_, every)) = (two_places(), waits());
    write_without_the_stick(&mut rig);
    let seals = rig.fakes.boxes.calls(BoxCall::Seal);
    rig.advance(every);
    rig.jobs();
    assert_eq!(
        rig.fakes.boxes.calls(BoxCall::Seal),
        seals,
        "still gone, so nothing sealed"
    );
    rig.fakes.places.come_back(Path::new(STICK));
    rig.advance(every.saturating_sub(MS));
    assert_eq!(rig.fakes.queued(), 0);
    rig.advance(MS);
    rig.jobs();
    assert_eq!(rig.page().waiting, None);
    assert_eq!(rig.actor.wait(), idle(&rig), "no look once nothing waits");
}

#[test]
fn a_failed_write_keeps_the_old_box_and_the_place_waits() {
    let mut rig = Rig::new();
    set_up(&mut rig, CLOUD_ROW);
    let old = box_in(&rig, CLOUD);
    rig.fakes.places.fail(PlaceCall::Write, PlaceError::Io);
    save_note(&mut rig, "Shed");
    rig.advance(waits().0);
    rig.jobs();
    press(&mut rig, FillButton::Backup);
    assert_eq!(rows(&rig), [("OneDrive".to_owned(), false)]);
    assert_eq!(box_in(&rig, CLOUD), old);
}

#[test]
fn a_box_too_large_or_a_failed_worker_writes_nothing_says_so_and_the_place_waits() {
    for (error, note) in [
        (BoxError::TooLarge, FillNote::TooLarge),
        (BoxError::TimedOut, FillNote::BoxFailed),
        (BoxError::Failed, FillNote::BoxFailed),
    ] {
        let mut rig = Rig::new();
        set_up(&mut rig, CLOUD_ROW);
        let old = box_in(&rig, CLOUD);
        rig.fakes.boxes.fail(BoxCall::Seal, error);
        save_note(&mut rig, "Shed");
        rig.advance(waits().0);
        rig.jobs();
        assert_eq!(rig.note(), Some(note), "{error:?} on the panel's page");
        assert_eq!(box_in(&rig, CLOUD), old, "{error:?}");
        press(&mut rig, FillButton::Backup);
        assert_eq!(rows(&rig), [("OneDrive".to_owned(), false)], "{error:?}");
        assert!(kept(&rig).sites[0].waiting, "{error:?} over a restart");
    }
}

#[test]
fn the_last_box_length_is_kept_so_a_full_place_is_passed_over_after_a_start() {
    let mut rig = Rig::new();
    set_up(&mut rig, CLOUD_ROW);
    let before = kept(&rig);
    assert!(before.sites[0].len > 0);
    let mut next = Rig::kept(sample(), &before);
    next.fakes.places.set_free(Path::new(CLOUD), 1);
    next.open();
    save_note(&mut next, "Shed");
    next.advance(waits().0);
    next.jobs();
    assert_eq!(
        next.fakes.boxes.calls(BoxCall::Seal),
        0,
        "no seal for a full place"
    );
}

#[test]
fn a_change_marks_the_backup_behind_in_its_blob_at_once_till_it_is_written() {
    let mut rig = Rig::new();
    set_up(&mut rig, CLOUD_ROW);
    save_note(&mut rig, "Shed");
    assert!(kept(&rig).behind, "saved before any stop");
    rig.advance(waits().0);
    rig.jobs();
    assert!(!kept(&rig).behind);
    save_note(&mut rig, "Barn");
    rig.advance(waits().0);
    rig.send(Msg::Quit);
    assert!(kept(&rig).behind, "a write cut by the stop stays behind");
}

#[test]
fn a_writes_note_waits_while_the_panel_is_shut_and_shows_when_it_opens() {
    let mut rig = Rig::new();
    set_up(&mut rig, CLOUD_ROW);
    save_note(&mut rig, "Shed");
    rig.send(Msg::Open);
    rig.fakes.boxes.fail(BoxCall::Seal, BoxError::TooLarge);
    rig.advance(waits().0);
    rig.jobs();
    rig.open();
    assert_eq!(rig.note(), Some(FillNote::TooLarge));
}
