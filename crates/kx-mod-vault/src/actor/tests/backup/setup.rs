//! Setup and Add place over the actor: a suggested place or a folder from the list, the new password, the padlock on
//! the job thread and the first box; the data folder, a box from another computer, and answers no longer waited for.

use std::path::Path;
use std::sync::Arc;

use kx_module_api::ports::{
    BoxError, BoxPart, Boxes, Calendar, FillButton, FillNote, NOTEBOOK_PART, PanelKey, PasswordAsk,
    Pick, PlaceKind, Sheet, box_file,
};
use kx_module_api::{Redacted, Zeroizing};
use kx_platform_fake::{BoxCall, FakeBoxes, PlaceCall};

use super::super::boxes::{CLOUD, FOLDER, STICK};
use super::super::logs;
use super::super::rig::Rig;
use super::{
    CLOUD_ROW, KEY, STICK_ROW, add, box_in, opens, pick, press, rows, set_up, spot, type_password,
};
use crate::actor::Msg;
use crate::backup::BackupStore;
use crate::entry::Stamp;
use crate::settings::VaultSettings;

/// The names on the last view's folder list, and the folder it shows.
fn folders(rig: &Rig) -> (Option<String>, Vec<String>) {
    let Sheet::Folders(list) = rig.page().sheet else {
        panic!("not the folder list: {:?}", rig.page().sheet);
    };
    assert_eq!(list.pick, Pick::Folder);
    let names = list.rows.iter().map(|s| s.name.expose().clone()).collect();
    (list.folder.map(|f| f.expose().clone()), names)
}

/// A rig with setup open and the cloud folder picked, every job run, so its password page shows.
fn at_password() -> Rig {
    let mut rig = Rig::new();
    rig.open();
    press(&mut rig, FillButton::Backup);
    rig.jobs();
    pick(&mut rig, CLOUD_ROW);
    rig.jobs();
    rig
}

#[test]
fn setup_in_a_suggested_place_makes_the_padlock_on_the_job_thread_and_writes_the_first_box() {
    let mut rig = Rig::new();
    rig.open();
    press(&mut rig, FillButton::Backup);
    rig.jobs();
    let cloud = spot(PlaceKind::Cloud, "OneDrive");
    let stick = spot(PlaceKind::Removable, "E: USB");
    assert_eq!(rig.page().sheet, Sheet::Places(vec![cloud, stick]));
    pick(&mut rig, CLOUD_ROW);
    rig.jobs();
    let Sheet::Password(asked) = rig.page().sheet else {
        panic!("no password page");
    };
    let settings = VaultSettings::from_defaults().unwrap();
    let min = u8::try_from(settings.box_min_chars).unwrap();
    assert_eq!(asked.ask, PasswordAsk::New { min });
    rig.text(KEY);
    let Sheet::Password(typed) = rig.page().sheet else {
        panic!("no password page");
    };
    assert_eq!(typed.text.expose().as_str(), KEY, "shown while typed");
    assert!(!format!("{:?}", rig.page()).contains(KEY));
    press(&mut rig, FillButton::Done);
    let made = (rig.fakes.boxes.calls(BoxCall::Padlock), rig.fakes.queued());
    assert_eq!(made, (0, 1), "the padlock waits for the job thread");
    rig.jobs();
    assert_eq!(rows(&rig), [("OneDrive".to_owned(), true)]);
    assert!(opens(&box_in(&rig, CLOUD).unwrap(), KEY));
    assert_eq!(rig.fakes.boxes.work(), Some(settings.box_work));
    let kept = BackupStore::new(Arc::new(rig.sealed.clone()))
        .load()
        .unwrap()
        .kept;
    assert!(kept.padlock.is_some() && kept.sites[0].mark.is_some());
}

#[test]
fn another_folder_browses_the_drives_and_uses_the_folder_opened() {
    let mut rig = Rig::new();
    rig.open();
    press(&mut rig, FillButton::Backup);
    rig.jobs();
    press(&mut rig, FillButton::OtherFolder);
    rig.jobs();
    assert_eq!(
        folders(&rig),
        (None, ["E: USB", "D:", "C:"].map(str::to_owned).to_vec())
    );
    pick(&mut rig, 1);
    assert_eq!(
        folders(&rig),
        (Some("D:".to_owned()), Vec::new()),
        "shown at once"
    );
    rig.jobs();
    assert_eq!(
        folders(&rig),
        (Some("D:".to_owned()), vec!["Backups".to_owned()])
    );
    pick(&mut rig, 0);
    rig.jobs();
    press(&mut rig, FillButton::Back);
    rig.jobs();
    assert_eq!(
        folders(&rig).0.as_deref(),
        Some("D:"),
        "Back opens the folder above"
    );
    pick(&mut rig, 0);
    rig.jobs();
    press(&mut rig, FillButton::Here);
    rig.jobs();
    type_password(&mut rig, KEY);
    rig.jobs();
    assert_eq!(rows(&rig), [("Backups".to_owned(), true)]);
    assert!(opens(&box_in(&rig, FOLDER).unwrap(), KEY));
}

#[test]
fn a_password_under_box_min_chars_is_refused_and_nothing_goes_to_the_worker() {
    let mut rig = at_password();
    type_password(&mut rig, "short");
    assert_eq!(rig.note(), Some(FillNote::TooShort));
    assert!(matches!(rig.page().sheet, Sheet::Password(_)));
    assert_eq!(rig.fakes.queued(), 0);
}

#[test]
fn the_data_folder_is_refused_at_the_pick_before_any_password() {
    let mut rig = Rig::new();
    rig.fakes.places.set_data_dir(Path::new(FOLDER));
    rig.open();
    press(&mut rig, FillButton::Backup);
    rig.jobs();
    press(&mut rig, FillButton::OtherFolder);
    rig.jobs();
    pick(&mut rig, 1);
    rig.jobs();
    pick(&mut rig, 0);
    rig.jobs();
    press(&mut rig, FillButton::Here);
    rig.jobs();
    assert!(
        matches!(rig.page().sheet, Sheet::Places(_)),
        "back to the places"
    );
    assert_eq!(rig.note(), Some(FillNote::DataFolder));
    assert_eq!(rig.fakes.boxes.calls(BoxCall::Padlock), 0);
}

/// Plants a box another computer sealed in `folder`, written `days` before the rig's wall time.
fn plant_foreign(rig: &Rig, folder: &str, days: i64) -> Vec<u8> {
    let settings = VaultSettings::from_defaults().unwrap();
    let mut run = FakeBoxes::new().start(settings.box_config());
    let password = Redacted::new(Zeroizing::new("another-computer".to_owned()));
    let lock = run.padlock(&password, settings.box_work).unwrap();
    let part = BoxPart {
        name: NOTEBOOK_PART.to_owned(),
        bytes: Redacted::new(Zeroizing::new(b"theirs".to_vec())),
    };
    let sealed = run.seal(&lock, &[part]).unwrap().expose().clone();
    let now = Stamp::of(&rig.calendar.now()).unix_ms();
    let file = Path::new(folder).join(box_file());
    rig.fakes
        .places
        .plant(&file, sealed.clone(), now - days * 86_400_000);
    sealed
}

#[test]
fn a_box_another_computer_wrote_is_never_written_over_and_its_question_shows() {
    let mut rig = Rig::new();
    set_up(&mut rig, CLOUD_ROW);
    let theirs = plant_foreign(&rig, STICK, 3);
    let writes = rig.fakes.places.calls(PlaceCall::Write);
    add(&mut rig, STICK_ROW);
    let Sheet::Foreign(found) = rig.page().sheet else {
        panic!("no question: {:?}", rig.page().sheet);
    };
    assert_eq!(found.place, spot(PlaceKind::Removable, "E: USB"));
    assert_eq!(
        (found.when.days_ago, found.when.hour, found.when.minute),
        (3, 9, 30)
    );
    assert_eq!(box_in(&rig, STICK), Some(theirs));
    assert_eq!(rig.fakes.places.calls(PlaceCall::Write), writes);
    rig.send(Msg::Key(PanelKey::Esc));
    assert!(
        matches!(rig.page().sheet, Sheet::Places(_)),
        "No goes back to the places"
    );
}

#[test]
fn an_answer_no_longer_waited_for_is_dropped() {
    let mut rig = Rig::new();
    plant_foreign(&rig, CLOUD, 1);
    rig.open();
    press(&mut rig, FillButton::Backup);
    rig.jobs();
    pick(&mut rig, CLOUD_ROW);
    press(&mut rig, FillButton::Back);
    press(&mut rig, FillButton::Backup);
    rig.jobs();
    pick(&mut rig, STICK_ROW);
    rig.jobs();
    let late = "the cloud's late look asks nothing about the stick";
    assert!(matches!(rig.page().sheet, Sheet::Password(_)), "{late}");
    rig.text(KEY);
    press(&mut rig, FillButton::Done);
    press(&mut rig, FillButton::Back);
    rig.jobs();
    assert_eq!(rig.fakes.boxes.calls(BoxCall::Padlock), 1);
    assert!(
        matches!(rig.page().sheet, Sheet::Places(_)),
        "the late padlock shows nothing"
    );
    press(&mut rig, FillButton::Back);
    press(&mut rig, FillButton::Backup);
    assert!(
        matches!(rig.page().sheet, Sheet::Places(_)),
        "and was not kept"
    );
}

#[test]
fn a_failed_padlock_says_so_and_logs_no_password_or_path() {
    let mut rig = at_password();
    rig.fakes.boxes.fail(BoxCall::Padlock, BoxError::TimedOut);
    let lines = logs::lines_of(|| {
        type_password(&mut rig, KEY);
        rig.jobs();
    });
    assert_eq!(rig.note(), Some(FillNote::BoxFailed));
    assert_eq!(lines.len(), 1, "{lines:?}");
    let leaks = [KEY, "sam", "OneDrive"];
    assert!(
        lines.iter().all(|l| leaks.iter().all(|k| !l.contains(k))),
        "{lines:?}"
    );
}

#[test]
fn back_from_setups_password_page_goes_to_the_places() {
    let mut rig = at_password();
    press(&mut rig, FillButton::Back);
    rig.jobs();
    let sheet = rig.page().sheet;
    assert!(
        matches!(&sheet, Sheet::Places(s) if s.len() == 2),
        "{sheet:?}"
    );
}

#[cfg(windows)]
#[test]
fn a_folder_whose_path_is_not_unicode_cannot_be_written() {
    use super::super::boxes::DRIVE;
    use std::os::windows::ffi::OsStringExt;
    let mut rig = Rig::new();
    let odd: Vec<u16> = "Odd".encode_utf16().chain([0xD800]).collect();
    let odd = Path::new(DRIVE).join(std::ffi::OsString::from_wide(&odd));
    rig.fakes.places.add_folder(&odd);
    rig.open();
    for button in [FillButton::Backup, FillButton::OtherFolder] {
        press(&mut rig, button);
        rig.jobs();
    }
    for row in [1, 1] {
        pick(&mut rig, row);
        rig.jobs();
    }
    press(&mut rig, FillButton::Here);
    assert_eq!(rig.note(), Some(FillNote::ReadOnly));
    assert_eq!(rig.fakes.queued(), 0, "nothing is looked at");
}
