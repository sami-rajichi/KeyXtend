//! A notebook that did not read holds the backup: nothing overwrites a box without the user's yes. The
//! hold shows as a note on every page; Add place and Change password write nothing; Back up now, and Bring back while
//! another place would be overwritten, ask first; a wrong password or leaving Bring back midway keeps it.

use kx_module_api::ports::{
    BackupAsk, Boxes, FillButton, FillClick, FillNote, PanelKey, Place, PlaceKind, SealedError,
    Sheet, box_file,
};
use kx_module_api::{Redacted, Zeroizing};
use kx_platform_fake::{BoxCall, FakeBoxes, PlaceCall};

use super::super::boxes::{CLOUD, STICK};
use super::super::rig::{Rig, sample};
use super::bring::plant_sample;
use super::{
    CLOUD_ROW, KEY, NEW_KEY, OLD_KEY, STICK_ROW, add, asked, box_in, kept, opens, pick, plant,
    press, rows, save_note, set_up, their_box, type_password, waits,
};
use crate::actor::Msg;
use crate::backup::{Kept, Site};
use crate::settings::VaultSettings;

/// A backup an earlier run set up in the cloud folder with `KEY`, its box not yet written.
pub(super) fn set_up_before() -> Kept {
    let settings = VaultSettings::from_defaults().unwrap();
    let mut run = FakeBoxes::new().start(settings.box_config());
    let password = Redacted::new(Zeroizing::new(KEY.to_owned()));
    let place = Place {
        kind: PlaceKind::Cloud,
        name: Redacted::new("OneDrive".to_owned()),
        path: Redacted::new(CLOUD.into()),
    };
    Kept {
        sites: vec![Site {
            place,
            written: None,
            mark: None,
            len: 0,
            waiting: false,
        }],
        padlock: Some(run.padlock(&password, settings.box_work).unwrap()),
        offered: true,
        ..Kept::default()
    }
}

/// A rig whose notebook did not read, over `set_up_before`, open, its look for a box to bring back done.
fn held() -> Rig {
    let mut rig = Rig::held(&set_up_before());
    rig.open();
    rig.jobs();
    rig
}

#[test]
fn the_hold_shows_on_every_page_and_a_change_writes_nothing() {
    let mut rig = held();
    assert_eq!(rig.note(), Some(FillNote::Paused), "the list");
    save_note(&mut rig, "Shed");
    rig.advance(waits().0);
    assert_eq!(
        rig.fakes.queued(),
        0,
        "the empty start never replaces a good box"
    );
    press(&mut rig, FillButton::Backup);
    assert_eq!(rig.note(), Some(FillNote::Paused), "the Backup page");
}

#[test]
fn a_failed_save_goes_ahead_of_the_hold_which_shows_again_once_a_save_works() {
    let mut rig = held();
    rig.sealed.fail_save(SealedError::Io);
    save_note(&mut rig, "Shed");
    assert_eq!(rig.note(), Some(FillNote::SaveFailed));
    save_note(&mut rig, "Barn");
    assert_eq!(rig.note(), Some(FillNote::Paused));
}

#[test]
fn while_held_add_place_and_change_password_write_nothing_and_every_place_waits() {
    let mut rig = held();
    press(&mut rig, FillButton::Backup);
    add(&mut rig, STICK_ROW);
    let waiting = [("OneDrive".to_owned(), false), ("E: USB".to_owned(), false)];
    assert_eq!(rows(&rig), waiting);
    press(&mut rig, FillButton::ChangePassword);
    type_password(&mut rig, NEW_KEY);
    rig.jobs();
    assert_eq!(rows(&rig), waiting);
    assert_eq!(
        rig.fakes.boxes.calls(BoxCall::Padlock),
        1,
        "the padlock is made"
    );
    assert_eq!(rig.fakes.boxes.calls(BoxCall::Seal), 0);
    assert_eq!(rig.fakes.places.calls(PlaceCall::Write), 0);
    assert!(
        kept(&rig).sites.iter().all(|s| s.waiting),
        "the blob keeps them waiting"
    );
}

#[test]
fn back_up_now_while_held_asks_once_and_only_its_yes_writes() {
    let mut rig = held();
    press(&mut rig, FillButton::Backup);
    press(&mut rig, FillButton::BackUpNow);
    assert_eq!(asked(&rig).0, Some(BackupAsk::Replace));
    rig.click(FillClick::Answer(false));
    press(&mut rig, FillButton::BackUpNow);
    rig.send(Msg::Key(PanelKey::Esc));
    assert_eq!(
        (asked(&rig).0, rig.fakes.queued()),
        (None, 0),
        "No and Esc write nothing"
    );
    press(&mut rig, FillButton::BackUpNow);
    rig.click(FillClick::Answer(true));
    rig.jobs();
    assert!(opens(&box_in(&rig, CLOUD).unwrap(), KEY));
    assert_eq!(rig.note(), None, "the hold is gone");
    assert!(!kept(&rig).held, "and the blob forgets it");
    save_note(&mut rig, "Barn");
    rig.advance(waits().0);
    assert_eq!(rig.fakes.queued(), 1, "writes go on after");
}

#[test]
fn with_no_padlock_there_is_no_backup_to_hold_so_setup_writes() {
    let kept = Kept {
        offered: true,
        ..Kept::default()
    };
    let mut rig = Rig::held(&kept);
    set_up(&mut rig, CLOUD_ROW);
    assert_eq!(rig.note(), None, "nothing is paused");
    assert!(opens(&box_in(&rig, CLOUD).unwrap(), KEY));
}

#[test]
fn the_hold_outlasts_a_restart_so_neither_a_change_nor_the_open_writes() {
    let mut rig = held();
    press(&mut rig, FillButton::Backup);
    add(&mut rig, STICK_ROW);
    save_note(&mut rig, "Shed");
    rig.send(Msg::Quit);
    let before = kept(&rig);
    assert!(before.held, "the blob keeps the hold");
    let mut next = Rig::kept(sample(), &before);
    next.open();
    assert_eq!(next.note(), Some(FillNote::Paused));
    assert_eq!(next.fakes.queued(), 0, "the open's catch-up writes nothing");
    save_note(&mut next, "Barn");
    next.advance(waits().0 + waits().1);
    assert_eq!(next.fakes.queued(), 0, "nor does the first change");
}

/// A held rig on the password page of the box `OLD_KEY` sealed, picked from the offer.
fn asked_for_its_box() -> Rig {
    let mut rig = Rig::held(&set_up_before());
    plant_sample(&rig, 1);
    rig.open();
    rig.jobs();
    pick(&mut rig, 0);
    rig.jobs();
    assert!(matches!(rig.page().sheet, Sheet::Password(_)));
    rig
}

/// Checks the hold stands, nothing came in, and no box was sealed or written, also once every wait has passed.
fn still_held(rig: &mut Rig, how: &str) {
    rig.jobs();
    rig.advance(waits().0 + waits().1);
    rig.jobs();
    assert!(kept(rig).held, "{how}: the hold stands");
    assert!(rig.actor.book.is_empty(), "{how}: nothing came in");
    assert_eq!(rig.fakes.boxes.calls(BoxCall::Seal), 0, "{how}");
    assert_eq!(rig.fakes.places.calls(PlaceCall::Write), 0, "{how}");
}

#[test]
fn while_held_a_wrong_password_keeps_the_hold_and_seals_nothing() {
    let mut rig = asked_for_its_box();
    type_password(&mut rig, KEY);
    rig.jobs();
    assert_eq!(rig.note(), Some(FillNote::WrongPassword));
    assert!(matches!(rig.page().sheet, Sheet::Password(_)));
    still_held(&mut rig, "a wrong password");
    type_password(&mut rig, OLD_KEY);
    rig.jobs();
    assert!(!kept(&rig).held, "only the right one ends it");
    assert_eq!(rig.fakes.boxes.calls(BoxCall::Seal), 1);
}

/// A way out of Bring back: its name, and how it leaves.
type Way = (&'static str, fn(&mut Rig));

/// Back, Esc and closing the panel.
const WAYS_OUT: [Way; 3] = [
    ("Back", |rig| press(rig, FillButton::Back)),
    ("Esc", |rig| rig.send(Msg::Key(PanelKey::Esc))),
    ("Close", |rig| rig.send(Msg::Close)),
];

#[test]
fn while_held_leaving_midway_keeps_the_hold_and_seals_nothing() {
    for (how, leave) in WAYS_OUT {
        let mut rig = asked_for_its_box();
        leave(&mut rig);
        still_held(&mut rig, how);
        let mut rig = asked_for_its_box();
        type_password(&mut rig, OLD_KEY);
        leave(&mut rig);
        still_held(&mut rig, &format!("{how}, the right password opening"));
    }
}

#[test]
fn replaces_yes_writes_once_though_a_write_was_due_from_the_last_stop() {
    let behind = Kept {
        behind: true,
        ..set_up_before()
    };
    let mut rig = Rig::held(&behind);
    rig.open();
    press(&mut rig, FillButton::Backup);
    press(&mut rig, FillButton::BackUpNow);
    rig.click(FillClick::Answer(true));
    rig.jobs();
    rig.advance(waits().0);
    rig.jobs();
    assert_eq!(rig.fakes.boxes.calls(BoxCall::Seal), 1);
}

/// A held rig that brought back the stick's older box, its Replace question up; gives the cloud's newer box too.
fn brought_older_box() -> (Rig, Vec<u8>) {
    let mut rig = Rig::held(&set_up_before());
    let (newer, _) = their_box(sample(), OLD_KEY);
    plant(&rig, CLOUD, &box_file(), newer.clone(), 1);
    let (older, _) = their_box(sample(), OLD_KEY);
    plant(&rig, STICK, &box_file(), older, 30);
    rig.open();
    rig.jobs();
    pick(&mut rig, 1);
    rig.jobs();
    type_password(&mut rig, OLD_KEY);
    rig.jobs();
    (rig, newer)
}

#[test]
fn an_older_box_brought_back_asks_before_it_touches_a_newer_one() {
    let (mut rig, newer) = brought_older_box();
    assert!(!rig.actor.book.is_empty(), "its entries came in");
    assert_eq!(rig.note(), Some(FillNote::Imported));
    assert_eq!(asked(&rig).0, Some(BackupAsk::Replace));
    rig.advance(waits().0 + waits().1);
    rig.jobs();
    assert!(kept(&rig).held, "the hold stands");
    assert_eq!(rig.fakes.boxes.calls(BoxCall::Seal), 0);
    assert_eq!(box_in(&rig, CLOUD), Some(newer), "the newer box stays");
}

#[test]
fn replaces_no_after_bring_back_keeps_the_hold_over_a_restart_and_asks_again() {
    let (mut rig, _) = brought_older_box();
    rig.click(FillClick::Answer(false));
    assert_eq!(asked(&rig).0, None);
    assert_eq!(rig.note(), Some(FillNote::Paused), "the hold still shows");
    rig.send(Msg::Quit);
    let before = kept(&rig);
    assert_eq!(before.sites.len(), 2, "the stick is kept as a place");
    let mut next = Rig::kept(sample(), &before);
    next.open();
    press(&mut next, FillButton::Backup);
    assert_eq!(next.fakes.boxes.calls(BoxCall::Seal), 0);
    press(&mut next, FillButton::BackUpNow);
    assert_eq!(asked(&next).0, Some(BackupAsk::Replace));
}

#[test]
fn replaces_yes_after_bring_back_writes_every_place_under_this_computers_padlock() {
    let (mut rig, _) = brought_older_box();
    rig.click(FillClick::Answer(true));
    rig.jobs();
    assert!(!kept(&rig).held, "the hold is gone");
    assert_eq!(rig.fakes.boxes.calls(BoxCall::Seal), 1);
    assert!(opens(&box_in(&rig, CLOUD).unwrap(), KEY));
    assert!(opens(&box_in(&rig, STICK).unwrap(), KEY));
}
