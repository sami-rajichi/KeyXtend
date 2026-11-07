//! Bring back from the places pages: setup's and Add place's both offer it, and Back returns to the page it
//! came from, whether boxes were found or the folder list opened. A hold still ends with the right password.

use std::path::Path;

use kx_module_api::ports::{FillButton, FillNote, Pick, Sheet, box_file};
use kx_platform_fake::BoxCall;

use super::super::boxes::{CLOUD, STICK};
use super::super::rig::Rig;
use super::bring::shed;
use super::{
    CLOUD_ROW, KEY, OLD_KEY, box_in, kept, pick, plant, press, set_up, their_box, type_password,
};

/// Plants a box of another computer on the stick.
fn plant_stick(rig: &Rig) {
    let (bytes, _) = their_box(vec![shed("Shed")], OLD_KEY);
    plant(rig, STICK, &box_file(), bytes, 1);
}

/// A rig on setup's places, over the sample, every job run.
fn on_setup() -> Rig {
    let mut rig = Rig::new();
    rig.open();
    press(&mut rig, FillButton::Backup);
    rig.jobs();
    rig
}

/// A rig on Add place's places, set up in the cloud folder, with no box left there, every job run.
fn on_add() -> Rig {
    let mut rig = Rig::new();
    set_up(&mut rig, CLOUD_ROW);
    let own = Path::new(CLOUD).join(box_file());
    rig.fakes.places.delete_by_hand(&own);
    press(&mut rig, FillButton::AddPlace);
    rig.jobs();
    rig
}

/// Presses `button`, every job run.
fn go(rig: &mut Rig, button: FillButton) {
    press(rig, button);
    rig.jobs();
}

/// Bring back shows its first page by what was found, then Back shows the places again.
fn there_and_back(rig: &mut Rig, boxes: bool) {
    go(rig, FillButton::BringBack);
    let sheet = rig.page().sheet;
    if boxes {
        assert!(matches!(sheet, Sheet::Found(_)), "{sheet:?}");
    } else {
        let picks = matches!(&sheet, Sheet::Folders(list) if list.pick == Pick::Backup);
        assert!(picks, "{sheet:?}");
    }
    go(rig, FillButton::Back);
    let sheet = rig.page().sheet;
    assert!(matches!(sheet, Sheet::Places(_)), "{sheet:?}");
}

#[test]
fn add_places_bring_back_shows_the_boxes_found_and_back_returns_to_add_place() {
    let mut rig = on_add();
    plant_stick(&rig);
    there_and_back(&mut rig, true);
    go(&mut rig, FillButton::Back);
    assert!(matches!(rig.page().sheet, Sheet::Backup(_)), "its own Back");
}

#[test]
fn add_places_bring_back_shows_the_folder_list_when_none_is_found_and_back_returns() {
    let mut rig = on_add();
    there_and_back(&mut rig, false);
    go(&mut rig, FillButton::Back);
    assert!(matches!(rig.page().sheet, Sheet::Backup(_)), "its own Back");
}

#[test]
fn setups_bring_back_returns_to_setups_places_in_both_flows() {
    let mut rig = on_setup();
    there_and_back(&mut rig, false);
    plant_stick(&rig);
    there_and_back(&mut rig, true);
    go(&mut rig, FillButton::Back);
    assert!(matches!(rig.page().sheet, Sheet::List(_)), "its own Back");
}

#[test]
fn the_backup_pages_bring_back_returns_to_the_backup_page() {
    let mut rig = Rig::new();
    set_up(&mut rig, CLOUD_ROW);
    go(&mut rig, FillButton::BringBack);
    assert!(matches!(rig.page().sheet, Sheet::Found(_)));
    go(&mut rig, FillButton::Back);
    assert!(matches!(rig.page().sheet, Sheet::Backup(_)));
}

#[test]
fn a_password_pages_back_comes_home_to_add_place_a_page_at_a_time() {
    let mut rig = on_add();
    plant_stick(&rig);
    go(&mut rig, FillButton::BringBack);
    pick(&mut rig, 0);
    rig.jobs();
    assert!(matches!(rig.page().sheet, Sheet::Password(_)));
    go(&mut rig, FillButton::Back);
    assert!(matches!(rig.page().sheet, Sheet::Found(_)));
    go(&mut rig, FillButton::Back);
    assert!(matches!(rig.page().sheet, Sheet::Places(_)));
    go(&mut rig, FillButton::Back);
    assert!(matches!(rig.page().sheet, Sheet::Backup(_)));
}

#[test]
fn add_places_bring_back_while_held_ends_the_hold_with_the_right_password() {
    let mut rig = Rig::new();
    set_up(&mut rig, CLOUD_ROW);
    let ours = box_in(&rig, CLOUD).unwrap();
    let mut next = Rig::held(&kept(&rig));
    let file = Path::new(CLOUD).join(box_file());
    next.fakes.places.plant(&file, ours, 1);
    next.open();
    next.jobs();
    press(&mut next, FillButton::Back);
    press(&mut next, FillButton::Backup);
    go(&mut next, FillButton::AddPlace);
    go(&mut next, FillButton::BringBack);
    pick(&mut next, 0);
    next.jobs();
    type_password(&mut next, KEY);
    next.jobs();
    assert_eq!(next.note(), Some(FillNote::Imported));
    assert_eq!(next.actor.book.len(), 3);
    assert!(!kept(&next).held, "the hold is gone");
    assert_eq!(next.fakes.boxes.calls(BoxCall::Seal), 1);
}
