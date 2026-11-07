//! After a box too large over the actor and the fake clock: its place waits, but looks and panel opens stop until a
//! change or Back up now, each failed try says so once, and a restart may try once; a failed worker is looked at still.

use kx_module_api::ports::{BoxError, FillButton, FillNote, PlaceKind};
use kx_platform_fake::BoxCall;

use super::super::rig::{Rig, sample};
use super::{CLOUD_ROW, idle, kept, press, rows, save_note, set_up, spot, waits};
use crate::actor::Msg;

/// A rig set up in the cloud folder whose write after a change sealed a box too large.
fn too_large() -> Rig {
    let mut rig = Rig::new();
    set_up(&mut rig, CLOUD_ROW);
    rig.fakes.boxes.fail(BoxCall::Seal, BoxError::TooLarge);
    save_note(&mut rig, "Shed");
    rig.advance(waits().0);
    rig.jobs();
    rig
}

#[test]
fn a_box_too_large_stops_the_looks_and_the_panels_opens_while_its_place_waits() {
    let mut rig = too_large();
    let seals = rig.fakes.boxes.calls(BoxCall::Seal);
    assert_eq!(rig.note(), Some(FillNote::TooLarge));
    assert!(kept(&rig).sites[0].waiting);
    assert_eq!(rig.actor.wait(), idle(&rig), "no wake for a look");
    rig.advance(waits().1);
    assert_eq!(rig.fakes.queued(), 0, "no look");
    rig.send(Msg::Open);
    rig.open();
    assert_eq!(rig.fakes.queued(), 0, "the open tries nothing");
    assert_eq!(rig.fakes.boxes.calls(BoxCall::Seal), seals);
    assert_eq!(rig.note(), None, "once per failed try");
    assert_eq!(
        rig.page().waiting,
        Some(spot(PlaceKind::Cloud, "OneDrive")),
        "the footer still names it"
    );
}

#[test]
fn a_change_tries_a_box_too_large_again_and_says_so_again() {
    let mut rig = too_large();
    let seals = rig.fakes.boxes.calls(BoxCall::Seal);
    rig.send(Msg::Open);
    rig.open();
    rig.fakes.boxes.fail(BoxCall::Seal, BoxError::TooLarge);
    save_note(&mut rig, "Barn");
    rig.advance(waits().0);
    rig.jobs();
    assert_eq!(rig.fakes.boxes.calls(BoxCall::Seal), seals + 1);
    assert_eq!(rig.note(), Some(FillNote::TooLarge));
    assert_eq!(rig.actor.wait(), idle(&rig), "and stops again");
    save_note(&mut rig, "Shop");
    rig.advance(waits().0);
    rig.jobs();
    assert_eq!(rig.page().waiting, None, "a box that fits is written");
    assert!(!kept(&rig).sites[0].waiting);
}

#[test]
fn back_up_now_tries_a_box_too_large_again() {
    let mut rig = too_large();
    let seals = rig.fakes.boxes.calls(BoxCall::Seal);
    press(&mut rig, FillButton::Backup);
    rig.fakes.boxes.fail(BoxCall::Seal, BoxError::TooLarge);
    press(&mut rig, FillButton::BackUpNow);
    assert_eq!(rig.fakes.queued(), 1);
    rig.jobs();
    assert_eq!(rig.fakes.boxes.calls(BoxCall::Seal), seals + 1);
    assert_eq!(rig.note(), Some(FillNote::TooLarge));
    press(&mut rig, FillButton::BackUpNow);
    rig.jobs();
    assert_eq!(rows(&rig), [("OneDrive".to_owned(), true)]);
}

#[test]
fn a_restart_may_try_a_box_too_large_once() {
    let every = waits().1;
    let mut next = Rig::kept(sample(), &kept(&too_large()));
    next.fakes.boxes.fail(BoxCall::Seal, BoxError::TooLarge);
    next.advance(every);
    next.jobs();
    assert_eq!(next.fakes.boxes.calls(BoxCall::Seal), 1);
    next.advance(every);
    assert_eq!(next.fakes.queued(), 0, "no second look");
}

#[test]
fn a_failed_worker_is_looked_at_again_as_it_may_come_back() {
    let mut rig = Rig::new();
    set_up(&mut rig, CLOUD_ROW);
    rig.fakes.boxes.fail(BoxCall::Seal, BoxError::Failed);
    save_note(&mut rig, "Shed");
    rig.advance(waits().0);
    rig.jobs();
    assert_eq!(rig.actor.wait(), Some(waits().1));
    rig.advance(waits().1);
    rig.jobs();
    assert_eq!(rig.page().waiting, None);
}
