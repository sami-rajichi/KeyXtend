//! A merge past the notebook's limits brings nothing in and writes nothing, so no entry is lost from the box it came
//! from; one exactly at them all comes in; and a merge saves the notebook once.

use std::path::Path;
use std::sync::Arc;

use kx_module_api::ports::{FillButton, FillClick, FillNote, box_file};
use kx_platform_fake::{BoxCall, FakeSealedStore, PlaceCall};

use super::super::boxes::{CLOUD, STICK};
use super::super::rig::Rig;
use super::bring::{bring_top, new_computer, plant_sample, shed};
use super::{
    CLOUD_ROW, KEY, OLD_KEY, STICK_ROW, add, box_in, kept, opens, pick, plant, press, set_up,
    their_box, type_password,
};
use crate::store::Store;

/// Plants a box another computer wrote in `folder`, holding Shed; gives its bytes.
fn plant_shed(rig: &Rig, folder: &str) -> Vec<u8> {
    let (bytes, _) = their_box(vec![shed("Shed")], OLD_KEY);
    plant(rig, folder, &box_file(), bytes.clone(), 1);
    bytes
}

/// Gives the actor a notebook store of its own, whose saves count the notebook's alone.
fn count_saves(rig: &mut Rig) -> FakeSealedStore {
    let notebook = FakeSealedStore::new();
    rig.actor.store = Store::new(Arc::new(notebook.clone()));
    notebook
}

/// The boxes sealed and the writes so far.
fn writes(rig: &Rig) -> (usize, usize) {
    let boxes = rig.fakes.boxes.calls(BoxCall::Seal);
    (boxes, rig.fakes.places.calls(PlaceCall::Write))
}

/// A rig set up on the stick with the sample's three entries, Shed's box in the cloud folder, its Bring back begun.
fn bringing() -> (Rig, Vec<u8>) {
    let mut rig = Rig::new();
    set_up(&mut rig, STICK_ROW);
    let theirs = plant_shed(&rig, CLOUD);
    press(&mut rig, FillButton::BringBack);
    rig.jobs();
    pick(&mut rig, 0);
    rig.jobs();
    (rig, theirs)
}

/// A rig set up in the cloud folder with the sample's three entries, Shed's box on the stick, its question's Yes given.
fn asked_yes() -> (Rig, Vec<u8>) {
    let mut rig = Rig::new();
    set_up(&mut rig, CLOUD_ROW);
    let theirs = plant_shed(&rig, STICK);
    add(&mut rig, STICK_ROW);
    rig.click(FillClick::Answer(true));
    (rig, theirs)
}

#[test]
fn bring_back_past_the_cap_changes_nothing_and_writes_nothing() {
    let (mut rig, theirs) = bringing();
    rig.actor.limits.max_entries = 3;
    let (before, sent, saves) = (kept(&rig), writes(&rig), count_saves(&mut rig));
    type_password(&mut rig, OLD_KEY);
    rig.jobs();
    assert_eq!(rig.note(), Some(FillNote::Full));
    assert_eq!(rig.actor.book.len(), 3);
    assert_eq!(kept(&rig), before, "no place, padlock or check changes");
    assert_eq!(writes(&rig), sent);
    assert_eq!(
        box_in(&rig, CLOUD),
        Some(theirs),
        "the entries stay in their box"
    );
    assert_eq!(saves.saves(), 0);
}

#[test]
fn bring_back_exactly_at_the_cap_all_comes_in() {
    let (mut rig, _) = bringing();
    rig.actor.limits.max_entries = 4;
    type_password(&mut rig, OLD_KEY);
    rig.jobs();
    assert_eq!(rig.note(), Some(FillNote::Imported));
    assert_eq!(rig.actor.book.len(), 4);
    assert!(opens(&box_in(&rig, CLOUD).unwrap(), KEY));
}

#[test]
fn a_foreign_yes_past_the_cap_changes_nothing_and_writes_nothing() {
    let (mut rig, theirs) = asked_yes();
    rig.actor.limits.max_entries = 3;
    let (before, sent) = (kept(&rig), writes(&rig));
    type_password(&mut rig, OLD_KEY);
    rig.jobs();
    assert_eq!(rig.note(), Some(FillNote::Full));
    assert_eq!(rig.actor.book.len(), 3);
    assert_eq!(kept(&rig), before);
    assert_eq!(writes(&rig), sent);
    assert_eq!(box_in(&rig, STICK), Some(theirs));
}

#[test]
fn a_foreign_yes_exactly_at_the_cap_all_comes_in() {
    let (mut rig, _) = asked_yes();
    rig.actor.limits.max_entries = 4;
    type_password(&mut rig, OLD_KEY);
    rig.jobs();
    assert_eq!(rig.note(), Some(FillNote::Imported));
    assert_eq!(rig.actor.book.len(), 4);
    assert!(opens(&box_in(&rig, STICK).unwrap(), KEY));
}

#[test]
fn an_empty_notebook_under_a_lowered_cap_keeps_no_padlock_and_no_place() {
    let mut rig = new_computer();
    plant_sample(&rig, 1);
    rig.actor.limits.max_entries = 2;
    let before = kept(&rig);
    bring_top(&mut rig, OLD_KEY);
    assert_eq!(rig.note(), Some(FillNote::Full));
    assert!(rig.actor.book.is_empty());
    assert_eq!(kept(&rig), before);
    assert_eq!(writes(&rig), (0, 0));
}

#[test]
fn past_the_cap_a_held_notebook_stays_held() {
    let mut rig = Rig::new();
    set_up(&mut rig, STICK_ROW);
    let ours = box_in(&rig, STICK).unwrap();
    let mut next = Rig::held(&kept(&rig));
    let file = Path::new(STICK).join(box_file());
    next.fakes.places.plant(&file, ours.clone(), 1);
    next.actor.limits.max_entries = 2;
    bring_top(&mut next, KEY);
    assert_eq!(next.note(), Some(FillNote::Full));
    assert!(kept(&next).held);
    assert_eq!(writes(&next), (0, 0));
    assert_eq!(box_in(&next, STICK), Some(ours));
}

#[test]
fn bring_back_and_a_foreign_yes_each_save_the_notebook_once() {
    let mut brought = new_computer();
    plant_sample(&brought, 1);
    let saves = count_saves(&mut brought);
    bring_top(&mut brought, OLD_KEY);
    assert_eq!(brought.actor.book.len(), 3);
    assert_eq!(saves.saves(), 1, "Bring back");
    let (mut asked, _) = asked_yes();
    let saves = count_saves(&mut asked);
    type_password(&mut asked, OLD_KEY);
    asked.jobs();
    assert_eq!(asked.actor.book.len(), 4);
    assert_eq!(saves.saves(), 1, "a foreign Yes");
}
