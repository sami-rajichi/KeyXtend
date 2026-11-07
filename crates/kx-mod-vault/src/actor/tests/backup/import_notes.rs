//! What a `KeePass` import says: a password goes as typed, a wrong one tries again and logs nothing of the file, each
//! refusal has its note, and what the rules, the duplicates, the caps and the worker left out all count.

use std::path::Path;

use kx_module_api::Redacted;
use kx_module_api::ports::{
    BoxError, Dropped, FillButton, FillNote, KeepassEntry, KeepassField, KeepassImport,
    MAX_KEEPASS_BYTES, PlaceError, Sheet,
};
use kx_platform_fake::{BoxCall, PlaceCall};

use super::super::boxes::DRIVE;
use super::super::logs;
use super::super::rig::{Rig, sample};
use super::import::{
    KDBX, VALUE, asks_password, count_saves, file, import, imported, login, on_invitation,
    pick_file, plant_bytes, plant_file,
};
use super::{CLOUD_ROW, KEY, OLD_KEY, press, set_up, type_password, waits};
use crate::fixture::secret;

#[test]
fn a_wrong_password_tries_again_and_logs_nothing_of_the_file() {
    let mut rig = on_invitation();
    plant_file(&rig, vec![login("Bank", "sam-user")]);
    press(&mut rig, FillButton::Import);
    pick_file(&mut rig);
    let lines = logs::lines_of(|| {
        type_password(&mut rig, KEY);
        rig.jobs();
    });
    assert_eq!(rig.note(), Some(FillNote::FileLocked));
    assert!(asks_password(&rig), "it tries again");
    let Sheet::Password(page) = rig.page().sheet else {
        panic!("no password page");
    };
    assert!(page.text.expose().is_empty(), "the line is wiped");
    let leaks = [KEY, "sam", "Bank", VALUE, DRIVE, "old"];
    let clean = |l: &String| leaks.iter().all(|k| !l.contains(k));
    assert!(lines.iter().all(clean), "{lines:?}");
    let lines = logs::lines_of(|| {
        type_password(&mut rig, OLD_KEY);
        rig.jobs();
    });
    assert_eq!(rig.page().note, Some(imported(1, 0)));
    assert!(lines.iter().all(clean), "{lines:?}");
}

#[test]
fn a_files_password_goes_exactly_as_typed_marks_included() {
    let mut rig = on_invitation();
    let marked = format!("\u{200f}{OLD_KEY}\u{200e}");
    rig.fakes
        .boxes
        .set_keepass(KDBX, &marked, vec![login("Bank", "sam")]);
    plant_bytes(&rig, KDBX.to_vec());
    import(&mut rig, &marked);
    assert_eq!(rig.page().note, Some(imported(1, 0)));
}

#[test]
fn a_file_locked_with_an_empty_password_opens_with_none_typed() {
    let mut rig = on_invitation();
    rig.fakes
        .boxes
        .set_keepass(KDBX, "", vec![login("Bank", "sam")]);
    plant_bytes(&rig, KDBX.to_vec());
    press(&mut rig, FillButton::Import);
    pick_file(&mut rig);
    press(&mut rig, FillButton::Done);
    rig.jobs();
    assert_eq!(rig.page().note, Some(imported(1, 0)));
}

#[test]
fn a_file_that_also_needs_a_key_file_answers_as_a_wrong_password_does() {
    let mut rig = on_invitation();
    rig.fakes.boxes.set_key_file(KDBX);
    plant_bytes(&rig, KDBX.to_vec());
    import(&mut rig, OLD_KEY);
    assert_eq!(rig.note(), Some(FillNote::FileLocked));
    assert!(asks_password(&rig));
    assert!(rig.actor.book.is_empty());
}

#[test]
fn a_file_past_the_cap_says_so_and_never_reaches_the_worker() {
    let mut rig = on_invitation();
    plant_bytes(&rig, vec![0; MAX_KEEPASS_BYTES + 1]);
    import(&mut rig, OLD_KEY);
    assert_eq!(rig.note(), Some(FillNote::FileTooLarge));
    assert_eq!(rig.fakes.boxes.calls(BoxCall::Keepass), 0);
    assert!(rig.actor.book.is_empty());
}

#[test]
fn each_refusal_of_the_worker_has_its_note_and_changes_nothing() {
    let cases = [
        (BoxError::TooLarge, FillNote::FileTooLarge),
        (BoxError::Damaged, FillNote::Damaged),
        (BoxError::Unsupported, FillNote::Unsupported),
        (BoxError::TimedOut, FillNote::FileUnread),
        (BoxError::Failed, FillNote::FileUnread),
    ];
    for (error, note) in cases {
        let mut rig = on_invitation();
        plant_file(&rig, vec![login("Bank", "sam")]);
        rig.fakes.boxes.fail(BoxCall::Keepass, error);
        let saves = count_saves(&mut rig);
        import(&mut rig, OLD_KEY);
        assert_eq!(rig.note(), Some(note), "{error:?}");
        assert!(asks_password(&rig), "{error:?} stays on the page");
        assert!(rig.actor.book.is_empty() && saves.saves() == 0);
    }
}

#[test]
fn a_file_that_does_not_read_says_so() {
    let mut rig = on_invitation();
    plant_file(&rig, vec![login("Bank", "sam")]);
    press(&mut rig, FillButton::Import);
    pick_file(&mut rig);
    rig.fakes.places.fail(PlaceCall::Read, PlaceError::Io);
    type_password(&mut rig, OLD_KEY);
    rig.jobs();
    assert_eq!(rig.note(), Some(FillNote::FileUnread));
    rig.fakes
        .places
        .delete_by_hand(&Path::new(DRIVE).join(file()));
    type_password(&mut rig, OLD_KEY);
    rig.jobs();
    assert_eq!(
        rig.note(),
        Some(FillNote::FileUnread),
        "nor when it is gone"
    );
    assert!(rig.actor.book.is_empty());
}

#[test]
fn what_the_rules_the_duplicates_and_the_worker_left_out_all_count() {
    let mut rig = Rig::with(sample());
    set_up(&mut rig, CLOUD_ROW);
    let import_ = KeepassImport {
        entries: vec![
            login("BANK", "Sam-User"),
            login("Shed", "alex"),
            KeepassEntry {
                custom: vec![KeepassField {
                    label: Redacted::new("Codes".to_owned()),
                    value: secret("one\ntwo"),
                }],
                ..login("Barn", "alex")
            },
            login("Barn", "ALEX"),
        ],
        dropped: Dropped {
            entries: 2,
            fields: 1,
        },
    };
    rig.fakes.boxes.set_import(KDBX, OLD_KEY, import_);
    plant_bytes(&rig, KDBX.to_vec());
    import(&mut rig, OLD_KEY);
    assert_eq!(rig.page().note, Some(imported(2, 6)));
    assert_eq!(rig.actor.book.len(), 5);
}

#[test]
fn the_entry_cap_holds_and_counts_the_rest() {
    let mut rig = on_invitation();
    rig.actor.limits.max_entries = 2;
    let many = vec![
        login("A", ""),
        login("B", ""),
        login("C", ""),
        login("D", ""),
    ];
    plant_file(&rig, many);
    import(&mut rig, OLD_KEY);
    assert_eq!(rig.page().note, Some(imported(2, 2)));
    assert_eq!(rig.actor.book.len(), 2);
}

#[test]
fn a_file_of_only_duplicates_saves_nothing_and_writes_nothing() {
    let mut rig = Rig::new();
    set_up(&mut rig, CLOUD_ROW);
    rig.advance(waits().0);
    rig.jobs();
    plant_file(&rig, vec![login("Bank", "sam-user")]);
    let sealed = rig.fakes.boxes.calls(BoxCall::Seal);
    let saves = count_saves(&mut rig);
    import(&mut rig, OLD_KEY);
    assert_eq!(rig.page().note, Some(imported(0, 1)));
    assert_eq!(saves.saves(), 0);
    rig.advance(waits().0);
    rig.jobs();
    assert_eq!(rig.fakes.boxes.calls(BoxCall::Seal), sealed);
}

#[test]
fn a_file_with_no_entry_says_none_came_in() {
    let mut rig = on_invitation();
    plant_file(&rig, Vec::new());
    import(&mut rig, OLD_KEY);
    assert_eq!(rig.page().note, Some(imported(0, 0)));
    assert!(rig.actor.book.is_empty());
    assert!(matches!(rig.page().sheet, Sheet::List(_)));
}
