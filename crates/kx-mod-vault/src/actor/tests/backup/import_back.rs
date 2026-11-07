//! Import's way back: Import shows on setup's and Add place's places as well, each page steps back one, the
//! pick's top returns to where Import was pressed, and so does the result and a list that fails. An abandoned read
//! brings nothing in.

use kx_module_api::keys;
use kx_module_api::ports::{FillButton, FillNote, PlaceError, Sheet};
use kx_platform_fake::{BoxCall, PlaceCall};

use super::super::rig::{Rig, sample};
use super::bring::new_computer;
use super::import::{file, import, imported, listed, login, open_row, pick_file, plant_file};
use super::{CLOUD_ROW, OLD_KEY, add, press, rows, save_note, set_up, type_password};
use crate::backup::Kept;

/// The name each suggested place shows on the last page, or panics when another page shows.
fn places(rig: &Rig) -> Vec<String> {
    let Sheet::Places(spots) = rig.page().sheet else {
        panic!("not the places: {:?}", rig.page().sheet);
    };
    spots.iter().map(|s| s.name.expose().clone()).collect()
}

/// A rig on setup's places: entries saved and no padlock yet, every job run.
fn on_setup() -> Rig {
    let mut rig = Rig::with(sample());
    rig.open();
    press(&mut rig, FillButton::Backup);
    rig.jobs();
    assert!(!places(&rig).is_empty());
    rig
}

/// A rig set up in the cloud folder, on Add place's places, every job run.
fn on_add() -> Rig {
    let mut rig = Rig::new();
    set_up(&mut rig, CLOUD_ROW);
    press(&mut rig, FillButton::AddPlace);
    rig.jobs();
    assert!(!places(&rig).is_empty());
    rig
}

#[test]
fn back_from_the_drives_goes_to_the_list_when_the_invitation_opened_them() {
    let mut rig = new_computer();
    rig.open();
    rig.jobs();
    plant_file(&rig, Vec::new());
    press(&mut rig, FillButton::Import);
    rig.jobs();
    press(&mut rig, FillButton::Back);
    assert!(matches!(rig.page().sheet, Sheet::List(_)));
    assert!(!rig.actor.panel.on_backup());
}

#[test]
fn back_from_the_drives_goes_to_the_backup_page_it_came_from() {
    let mut rig = Rig::new();
    set_up(&mut rig, CLOUD_ROW);
    press(&mut rig, FillButton::Import);
    rig.jobs();
    press(&mut rig, FillButton::Back);
    assert_eq!(rows(&rig).len(), 1, "the Backup page");
}

#[test]
fn back_steps_from_the_password_to_the_folder_to_the_drives() {
    let mut rig = Rig::new();
    set_up(&mut rig, CLOUD_ROW);
    plant_file(&rig, Vec::new());
    press(&mut rig, FillButton::Import);
    pick_file(&mut rig);
    press(&mut rig, FillButton::Back);
    rig.jobs();
    assert!(listed(&rig).contains(&file()), "its folder");
    press(&mut rig, FillButton::Back);
    rig.jobs();
    assert!(listed(&rig).contains(&"D:".to_owned()), "the drives");
    press(&mut rig, FillButton::Back);
    assert_eq!(rows(&rig).len(), 1);
}

#[test]
fn import_on_setups_places_goes_back_to_them_and_setup_goes_on() {
    let mut rig = on_setup();
    let shown = places(&rig);
    press(&mut rig, FillButton::Import);
    rig.jobs();
    assert!(!listed(&rig).is_empty());
    press(&mut rig, FillButton::Back);
    rig.jobs();
    assert_eq!(places(&rig), shown, "back on setup's places");
    plant_file(&rig, vec![login("Shed", "alex")]);
    import(&mut rig, OLD_KEY);
    assert_eq!(rig.page().note, Some(imported(1, 0)));
    rig.jobs();
    assert_eq!(places(&rig), shown, "the result shows there too");
    let note = Some(imported(1, 0));
    assert_eq!(rig.page().note, note, "its list coming in keeps the note");
    assert_eq!(rig.actor.book.len(), 4);
    super::pick(&mut rig, CLOUD_ROW);
    rig.jobs();
    assert!(
        matches!(rig.page().sheet, Sheet::Password(_)),
        "setup goes on to its password"
    );
}

#[test]
fn import_on_add_places_goes_back_to_them_then_to_the_backup_page() {
    let mut rig = on_add();
    let shown = places(&rig);
    press(&mut rig, FillButton::Import);
    rig.jobs();
    press(&mut rig, FillButton::Back);
    rig.jobs();
    assert_eq!(places(&rig), shown, "back on Add place's places");
    plant_file(&rig, vec![login("Shed", "alex")]);
    import(&mut rig, OLD_KEY);
    assert_eq!(rig.page().note, Some(imported(1, 0)));
    rig.jobs();
    assert_eq!(places(&rig), shown);
    press(&mut rig, FillButton::Back);
    assert_eq!(rows(&rig).len(), 1, "Add place's Back goes home, as it did");
}

#[test]
fn import_works_with_a_place_added_before_it() {
    let mut rig = Rig::new();
    set_up(&mut rig, CLOUD_ROW);
    add(&mut rig, super::STICK_ROW);
    plant_file(&rig, vec![login("Shed", "alex")]);
    import(&mut rig, OLD_KEY);
    assert_eq!(rows(&rig).len(), 2);
    assert_eq!(rig.page().note, Some(imported(1, 0)));
}

#[test]
fn a_list_that_fails_on_the_invitation_goes_back_to_the_list_with_its_note() {
    let mut rig = new_computer();
    rig.open();
    rig.jobs();
    rig.fakes.places.fail(PlaceCall::Folders, PlaceError::Io);
    press(&mut rig, FillButton::Import);
    rig.jobs();
    assert!(matches!(rig.page().sheet, Sheet::List(_)));
    assert_eq!(rig.note(), Some(FillNote::FileUnread));
}

#[test]
fn a_folder_that_fails_steps_back_to_the_drives_with_its_note() {
    let mut rig = Rig::new();
    set_up(&mut rig, CLOUD_ROW);
    press(&mut rig, FillButton::Import);
    rig.jobs();
    rig.fakes.places.fail(PlaceCall::Files, PlaceError::Io);
    open_row(&mut rig, "D:");
    rig.jobs();
    assert!(listed(&rig).contains(&"D:".to_owned()), "the drives");
    assert_eq!(rig.note(), Some(FillNote::FileUnread));
}

#[test]
fn a_list_that_fails_steps_back_to_its_places_with_the_note_of_what_it_was_for() {
    let cases = [
        (on_setup(), FillButton::OtherFolder, FillNote::ReadOnly),
        (on_setup(), FillButton::BringBack, FillNote::Unread),
        (on_add(), FillButton::OtherFolder, FillNote::ReadOnly),
    ];
    for (mut rig, button, note) in cases {
        let shown = places(&rig);
        rig.fakes.places.fail(PlaceCall::Folders, PlaceError::Io);
        press(&mut rig, button);
        rig.jobs();
        assert_eq!(places(&rig), shown, "{button:?}");
        assert_eq!(rig.note(), Some(note), "{button:?}");
    }
}

#[test]
fn an_import_on_setups_places_makes_no_offer_until_setup_is_left() {
    for button in [FillButton::Back, FillButton::Close] {
        let mut rig = Rig::kept(Vec::new(), &Kept::default());
        rig.open();
        rig.jobs();
        press(&mut rig, FillButton::Backup);
        rig.jobs();
        assert!(!places(&rig).is_empty());
        plant_file(&rig, vec![login("Shed", "alex")]);
        import(&mut rig, OLD_KEY);
        rig.jobs();
        assert_eq!(rig.page().note, Some(imported(1, 0)), "{button:?}");
        let offered = |rig: &Rig| rig.notices().iter().any(|n| n.key == keys::VAULT_OFFER);
        assert!(!offered(&rig), "setup shows already: {button:?}");
        press(&mut rig, button);
        if button == FillButton::Close {
            rig.open();
        }
        save_note(&mut rig, "Barn");
        assert!(offered(&rig), "the next save offers: {button:?}");
    }
}

#[test]
fn a_read_left_by_back_or_by_closing_brings_nothing_in() {
    for close in [false, true] {
        let mut rig = new_computer();
        rig.open();
        rig.jobs();
        plant_file(&rig, vec![login("Bank", "sam")]);
        press(&mut rig, FillButton::Import);
        pick_file(&mut rig);
        type_password(&mut rig, OLD_KEY);
        let button = if close {
            FillButton::Close
        } else {
            FillButton::Back
        };
        press(&mut rig, button);
        rig.jobs();
        assert!(rig.actor.book.is_empty(), "close: {close}");
        assert_eq!(
            rig.fakes.boxes.calls(BoxCall::Keepass),
            1,
            "it ran, unheard"
        );
        assert!(rig.actor.backup.take_import().is_none());
    }
}

#[test]
fn nothing_goes_while_the_read_runs_but_back_and_close() {
    let mut rig = new_computer();
    rig.open();
    rig.jobs();
    plant_file(&rig, vec![login("Bank", "sam")]);
    press(&mut rig, FillButton::Import);
    pick_file(&mut rig);
    type_password(&mut rig, OLD_KEY);
    press(&mut rig, FillButton::Done);
    assert_eq!(rig.fakes.queued(), 1, "one read");
}
