//! A copy of a box brought back with the box's own file beside it: a write to that folder replaces that file, so the
//! copy's entries come in and nothing there changes without the user's yes.

use std::path::Path;

use kx_module_api::ports::{
    BOX_EXT, BOX_NAME, BackupAsk, FillButton, FillClick, FillNote, Sheet, box_file,
};
use kx_platform_fake::{BoxCall, PlaceCall};

use super::super::boxes::{CLOUD, STICK};
use super::super::rig::{Rig, sample};
use super::bring::{bring_top, new_computer, shed};
use super::held::set_up_before;
use super::{
    CLOUD_ROW, KEY, OLD_KEY, add, asked, box_in, kept, opens, pick, plant, press, rows, set_up,
    their_box, type_password, waits,
};

/// The name a cloud gives a conflict copy of the box.
fn copy_name() -> String {
    format!("{BOX_NAME} (1).{BOX_EXT}")
}

/// Plants in `folder` the box's own file and, written later, a copy of another name; gives the own file.
fn plant_both(rig: &Rig, folder: &str) -> Vec<u8> {
    let (own, _) = their_box(sample(), OLD_KEY);
    plant(rig, folder, &box_file(), own.clone(), 30);
    let (copy, _) = their_box(sample(), OLD_KEY);
    plant(rig, folder, &copy_name(), copy, 1);
    own
}

#[test]
fn a_new_computer_gets_the_copys_entries_and_its_folder_stays_no_place() {
    let mut rig = new_computer();
    let own = plant_both(&rig, CLOUD);
    bring_top(&mut rig, OLD_KEY);
    assert_eq!(rig.note(), Some(FillNote::Imported));
    assert_eq!(rig.actor.book.len(), sample().len());
    assert!(rows(&rig).is_empty(), "no place, so nothing is written");
    rig.advance(waits().0 + waits().1);
    rig.jobs();
    assert_eq!(rig.fakes.places.calls(PlaceCall::Write), 0);
    assert_eq!(box_in(&rig, CLOUD), Some(own), "the box beside it stays");
}

#[test]
fn adding_that_folder_after_asks_about_the_box_there() {
    let mut rig = new_computer();
    plant_both(&rig, CLOUD);
    bring_top(&mut rig, OLD_KEY);
    add(&mut rig, CLOUD_ROW);
    assert!(matches!(rig.page().sheet, Sheet::Foreign(_)));
}

#[test]
fn while_held_a_copy_in_the_only_place_asks_before_the_box_beside_it_goes() {
    let mut rig = Rig::held(&set_up_before());
    let own = plant_both(&rig, CLOUD);
    bring_top(&mut rig, OLD_KEY);
    assert_eq!(asked(&rig).0, Some(BackupAsk::Replace));
    rig.advance(waits().0 + waits().1);
    rig.jobs();
    assert!(kept(&rig).held, "the hold stands");
    assert_eq!(box_in(&rig, CLOUD), Some(own), "until the yes");
    rig.click(FillClick::Answer(true));
    rig.jobs();
    assert!(!kept(&rig).held);
    assert!(opens(&box_in(&rig, CLOUD).unwrap(), KEY));
}

#[test]
fn while_held_a_copy_in_a_folder_no_backup_uses_stays_out_of_it_after_the_yes_too() {
    let mut rig = Rig::held(&set_up_before());
    let own = plant_both(&rig, STICK);
    bring_top(&mut rig, OLD_KEY);
    assert_eq!(asked(&rig).0, Some(BackupAsk::Replace));
    assert_eq!(rows(&rig), [("OneDrive".to_owned(), false)]);
    rig.click(FillClick::Answer(true));
    rig.jobs();
    assert_eq!(rig.fakes.boxes.calls(BoxCall::Seal), 1);
    assert!(opens(&box_in(&rig, CLOUD).unwrap(), KEY));
    assert_eq!(box_in(&rig, STICK), Some(own));
}

#[test]
fn a_copy_beside_this_computers_own_box_rewrites_that_box_as_any_change_does() {
    let mut rig = Rig::new();
    set_up(&mut rig, CLOUD_ROW);
    let (copy, _) = their_box(vec![shed("Shed")], OLD_KEY);
    plant(&rig, CLOUD, &copy_name(), copy.clone(), 0);
    press(&mut rig, FillButton::BringBack);
    rig.jobs();
    pick(&mut rig, 0);
    rig.jobs();
    type_password(&mut rig, OLD_KEY);
    rig.jobs();
    assert_eq!(asked(&rig).0, None, "no hold, no question");
    assert_eq!(rows(&rig), [("OneDrive".to_owned(), true)]);
    assert!(opens(&box_in(&rig, CLOUD).unwrap(), KEY));
    let kept_copy = rig.fakes.places.file(&Path::new(CLOUD).join(copy_name()));
    assert_eq!(kept_copy, Some(copy), "the copy itself is never written");
}
