//! A box another computer wrote, in a place picked at setup or Add place: one question, nothing written before its
//! Yes; Yes asks that box's password, brings its entries in and rewrites it under this computer's padlock; No leaves
//! the box and skips the place. Two computers writing one place: the last write wins.

use std::path::Path;

use kx_module_api::ports::{
    EntryKind, FillButton, FillClick, FillNote, PasswordAsk, PlaceError, PlaceKind, Sheet, box_file,
};
use kx_platform_fake::{BoxCall, PlaceCall};

use super::super::boxes::{CLOUD, FOLDER, STICK};
use super::super::rig::Rig;
use super::{
    CLOUD_ROW, KEY, OLD_KEY, STICK_ROW, add, box_in, kept, opens, pick, plant, press, rows,
    save_note, set_up, spot, their_box, type_password, waits,
};
use crate::fixture::{draft, field};

/// Plants a box another computer wrote in `folder`, a note named Shed sealed with `OLD_KEY`; gives its bytes.
fn plant_shed(rig: &Rig, folder: &str) -> Vec<u8> {
    let shed = draft("Shed", EntryKind::Note, vec![field("text", "42", false)]);
    let (bytes, _) = their_box(vec![shed], OLD_KEY);
    plant(rig, folder, &box_file(), bytes.clone(), 2);
    bytes
}

/// A rig set up in the cloud folder with a box from another computer on the stick, its question shown.
fn asked() -> (Rig, Vec<u8>) {
    let mut rig = Rig::new();
    set_up(&mut rig, CLOUD_ROW);
    let theirs = plant_shed(&rig, STICK);
    add(&mut rig, STICK_ROW);
    assert!(
        matches!(rig.page().sheet, Sheet::Foreign(_)),
        "{:?}",
        rig.page().sheet
    );
    (rig, theirs)
}

#[test]
fn yes_brings_its_entries_in_and_rewrites_it_under_this_padlock_after_the_right_password() {
    let (mut rig, theirs) = asked();
    let writes = rig.fakes.places.calls(PlaceCall::Write);
    rig.click(FillClick::Answer(true));
    let Sheet::Password(page) = rig.page().sheet else {
        panic!("no password page");
    };
    assert_eq!(page.ask, PasswordAsk::Foreign);
    type_password(&mut rig, KEY);
    rig.jobs();
    assert_eq!(rig.note(), Some(FillNote::WrongPassword));
    assert_eq!(
        box_in(&rig, STICK),
        Some(theirs),
        "never written before the right password"
    );
    assert_eq!(rig.fakes.places.calls(PlaceCall::Write), writes);
    type_password(&mut rig, OLD_KEY);
    rig.jobs();
    let rewritten = box_in(&rig, STICK).unwrap();
    assert!(opens(&rewritten, KEY) && !opens(&rewritten, OLD_KEY));
    assert!(opens(&box_in(&rig, CLOUD).unwrap(), KEY));
    assert_eq!(rows(&rig).len(), 2);
    press(&mut rig, FillButton::Back);
    assert_eq!(rig.names(), ["Alex", "Bank", "Gate", "Shed"]);
}

#[test]
fn no_leaves_the_box_and_skips_the_place() {
    let (mut rig, theirs) = asked();
    rig.click(FillClick::Answer(false));
    assert!(
        matches!(rig.page().sheet, Sheet::Places(_)),
        "back to the places"
    );
    assert_eq!(box_in(&rig, STICK), Some(theirs));
    assert_eq!(kept(&rig).sites.len(), 1, "the place is skipped");
}

#[test]
fn at_setup_its_yes_keeps_that_boxs_padlock_as_no_padlock_is_made_yet() {
    let mut rig = Rig::new();
    plant_shed(&rig, CLOUD);
    rig.open();
    press(&mut rig, FillButton::Backup);
    rig.jobs();
    pick(&mut rig, CLOUD_ROW);
    rig.jobs();
    rig.click(FillClick::Answer(true));
    type_password(&mut rig, OLD_KEY);
    rig.jobs();
    assert_eq!(rows(&rig), [("OneDrive".to_owned(), true)]);
    assert!(
        opens(&box_in(&rig, CLOUD).unwrap(), OLD_KEY),
        "one password opens it still"
    );
    assert_eq!(rig.actor.book.len(), 4);
}

#[test]
fn when_two_computers_write_one_place_the_last_write_wins_and_nothing_asks() {
    let mut rig = Rig::new();
    set_up(&mut rig, CLOUD_ROW);
    plant_shed(&rig, CLOUD);
    save_note(&mut rig, "Barn");
    rig.advance(waits().0);
    rig.jobs();
    assert!(
        opens(&box_in(&rig, CLOUD).unwrap(), KEY),
        "this computer's write is the last"
    );
    assert!(!matches!(rig.page().sheet, Sheet::Foreign(_)));
}

#[test]
fn a_yes_while_held_brings_the_entries_in_and_keeps_the_hold_and_every_box() {
    let mut rig = Rig::new();
    set_up(&mut rig, CLOUD_ROW);
    let ours = box_in(&rig, CLOUD).unwrap();
    let mut next = Rig::held(&kept(&rig));
    let file = Path::new(CLOUD).join(box_file());
    next.fakes.places.plant(&file, ours.clone(), 1);
    let theirs = plant_shed(&next, STICK);
    next.open();
    next.jobs();
    press(&mut next, FillButton::Back);
    press(&mut next, FillButton::Backup);
    add(&mut next, STICK_ROW);
    next.click(FillClick::Answer(true));
    type_password(&mut next, OLD_KEY);
    next.jobs();
    assert_eq!(next.note(), Some(FillNote::Imported));
    assert_eq!(next.actor.book.len(), 1, "the entries come in");
    let kept = kept(&next);
    assert!(kept.held, "the hold stays");
    assert_eq!(
        kept.sites.len(),
        2,
        "and its place waits for the hold to end"
    );
    assert_eq!(next.fakes.boxes.calls(BoxCall::Seal), 0);
    assert_eq!(next.fakes.places.calls(PlaceCall::Write), 0);
    assert_eq!(
        box_in(&next, CLOUD),
        Some(ours),
        "this computer's good box stays"
    );
    assert_eq!(box_in(&next, STICK), Some(theirs));
}

#[test]
fn a_box_in_a_folder_no_find_reaches_shows_its_own_date() {
    let mut rig = Rig::new();
    set_up(&mut rig, CLOUD_ROW);
    plant_shed(&rig, FOLDER);
    press(&mut rig, FillButton::AddPlace);
    rig.jobs();
    press(&mut rig, FillButton::OtherFolder);
    rig.jobs();
    pick(&mut rig, 1);
    rig.jobs();
    pick(&mut rig, 0);
    rig.jobs();
    press(&mut rig, FillButton::Here);
    rig.jobs();
    let Sheet::Foreign(row) = rig.page().sheet else {
        panic!("no dated question: {:?}", rig.page().sheet);
    };
    assert_eq!(row.when.days_ago, 2);
}

#[test]
fn a_box_whose_date_the_system_does_not_tell_asks_without_one() {
    let mut rig = Rig::new();
    set_up(&mut rig, CLOUD_ROW);
    let theirs = plant_shed(&rig, STICK);
    rig.fakes.places.fail(PlaceCall::Modified, PlaceError::Io);
    add(&mut rig, STICK_ROW);
    let Sheet::ForeignUndated(place) = rig.page().sheet else {
        panic!("no undated question: {:?}", rig.page().sheet);
    };
    assert_eq!(place, spot(PlaceKind::Removable, "E: USB"));
    rig.click(FillClick::Answer(false));
    assert!(matches!(rig.page().sheet, Sheet::Places(_)), "No skips it");
    assert_eq!(box_in(&rig, STICK), Some(theirs));
}
