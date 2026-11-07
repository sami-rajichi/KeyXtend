//! Bring back over the actor: an empty notebook offers the boxes found, newest first and paged; none found opens the
//! folder list for a backup file; the right password brings the entries back under the box's padlock or this
//! computer's own; a wrong password tries again; a box too big, damaged or newer says so; and the hold ends.

use std::path::Path;
use std::sync::Arc;

use kx_module_api::ports::{
    BOX_EXT, BOX_MARK, BOX_NAME, BOX_VERSION, BoxPart, FillButton, FillClick, FillNote,
    FolderSheet, NOTEBOOK_PART, NoteCounts, PageNote, PasswordAsk, Pick, PlaceKind, Sheet,
    box_file,
};
use kx_module_api::{Redacted, Zeroizing};
use kx_platform_fake::BoxCall;

use super::super::boxes::{CLOUD, DRIVE, STICK};
use super::super::logs;
use super::super::rig::{Rig, sample};
use super::{
    KEY, OLD_KEY, STICK_ROW, box_in, kept, opens, pick, plant, press, rows, seal, set_up,
    their_box, type_password,
};
use crate::backup::Kept;
use crate::fixture::{draft, field};
use crate::settings::VaultSettings;
use crate::store::Store;

/// A rig with no entry and no backup yet, as on a new computer.
pub(super) fn new_computer() -> Rig {
    let fresh = Kept {
        offered: true,
        ..Kept::default()
    };
    Rig::kept(Vec::new(), &fresh)
}

/// A note named `name`, as another computer saved it.
pub(super) fn shed(name: &str) -> crate::entry::Draft {
    draft(
        name,
        kx_module_api::ports::EntryKind::Note,
        vec![field("text", "42", false)],
    )
}

/// Plants the sample's box, sealed with `OLD_KEY`, in the cloud folder, written `days` ago; gives its padlock.
pub(super) fn plant_sample(rig: &Rig, days: u32) -> kx_module_api::ports::Padlock {
    let (bytes, lock) = their_box(sample(), OLD_KEY);
    plant(rig, CLOUD, &box_file(), bytes, days);
    lock
}

/// Opens the panel on the offer, picks its top box and types `password`, every job run.
pub(super) fn bring_top(rig: &mut Rig, password: &str) {
    rig.open();
    rig.jobs();
    pick(rig, 0);
    rig.jobs();
    type_password(rig, password);
    rig.jobs();
}

/// The days ago of each found box on the last view.
pub(super) fn found(rig: &Rig) -> Vec<u32> {
    let Sheet::Found(rows) = rig.page().sheet else {
        panic!("not the found boxes: {:?}", rig.page().sheet);
    };
    rows.iter().map(|r| r.when.days_ago).collect()
}

/// The folder list on the last view.
fn folders(rig: &Rig) -> FolderSheet {
    let Sheet::Folders(list) = rig.page().sheet else {
        panic!("no folder list: {:?}", rig.page().sheet);
    };
    list
}

#[test]
fn an_empty_notebook_offers_the_boxes_found_newest_first_a_page_at_a_time() {
    let mut rig = new_computer();
    let (bytes, _) = their_box(sample(), OLD_KEY);
    for (n, days) in [4, 1, 6, 2, 5, 3].into_iter().enumerate() {
        let folder = format!("{CLOUD}/old-{n}");
        let name = format!("{BOX_NAME} ({n}).{BOX_EXT}");
        plant(&rig, &folder, &name, bytes.clone(), days);
    }
    rig.open();
    rig.jobs();
    assert_eq!(found(&rig), [1, 2, 3, 4, 5], "newest first");
    rig.click(FillClick::Page { back: false });
    assert_eq!(found(&rig), [6]);
}

#[test]
fn a_boxs_password_needs_a_character_and_loses_its_marks_as_the_new_one_did() {
    let mut rig = new_computer();
    plant_sample(&rig, 1);
    rig.open();
    rig.jobs();
    pick(&mut rig, 0);
    rig.jobs();
    press(&mut rig, FillButton::Done);
    assert_eq!(rig.note(), Some(FillNote::TooShort));
    assert_eq!(rig.fakes.queued(), 0, "nothing goes to the worker");
    type_password(&mut rig, &format!("\u{200f}{OLD_KEY}\u{200e}"));
    rig.jobs();
    assert_eq!(rig.actor.book.len(), sample().len());
}

#[test]
fn with_none_found_bring_back_opens_the_folder_list_for_a_backup_file() {
    let mut rig = new_computer();
    let (bytes, _) = their_box(sample(), OLD_KEY);
    plant(&rig, DRIVE, "laptop.kxbox", bytes, 1);
    rig.open();
    rig.jobs();
    assert!(
        matches!(rig.page().sheet, Sheet::List(_)),
        "nothing found, nothing offered"
    );
    press(&mut rig, FillButton::Backup);
    rig.jobs();
    press(&mut rig, FillButton::BringBack);
    rig.jobs();
    let drives = folders(&rig);
    assert_eq!((drives.pick, drives.folder), (Pick::Backup, None));
    pick(&mut rig, 1);
    rig.jobs();
    let kinds: Vec<_> = folders(&rig).rows.iter().map(|r| r.kind).collect();
    assert_eq!(
        kinds,
        [PlaceKind::Folder, PlaceKind::File],
        "its folders, then its backups"
    );
    pick(&mut rig, 1);
    rig.jobs();
    let Sheet::Password(page) = rig.page().sheet else {
        panic!("no password page");
    };
    assert_eq!(page.ask, PasswordAsk::Open { own: false });
    type_password(&mut rig, OLD_KEY);
    rig.jobs();
    assert_eq!(
        rows(&rig),
        [("D:".to_owned(), true)],
        "its folder is the first place"
    );
}

#[test]
fn the_right_password_brings_the_entries_back_and_keeps_the_boxs_padlock() {
    let mut rig = new_computer();
    let theirs = plant_sample(&rig, 1);
    bring_top(&mut rig, OLD_KEY);
    let note = PageNote {
        note: FillNote::Imported,
        counts: NoteCounts {
            added: 3,
            skipped: 0,
        },
    };
    assert_eq!(rig.page().note, Some(note));
    assert_eq!(rows(&rig), [("OneDrive".to_owned(), true)]);
    let kept = kept(&rig);
    assert_eq!(kept.padlock, Some(theirs), "the same password works on");
    assert!(kept.checks.due.is_some(), "and its memory checks begin");
    assert!(opens(&box_in(&rig, CLOUD).unwrap(), OLD_KEY));
    press(&mut rig, FillButton::Back);
    assert_eq!(rig.names(), ["Alex", "Bank", "Gate"]);
    let saved = Store::new(Arc::new(rig.sealed.clone())).load().unwrap();
    assert_eq!(saved.book.len(), 3, "the notebook is saved");
    press(&mut rig, FillButton::Backup);
    press(&mut rig, FillButton::AddPlace);
    rig.jobs();
    pick(&mut rig, 0);
    rig.jobs();
    assert!(
        matches!(rig.page().sheet, Sheet::Backup(_)),
        "no question for its place"
    );
}

#[test]
fn a_computer_with_its_own_padlock_rewrites_the_box_brought_back_under_it() {
    let mut rig = Rig::new();
    set_up(&mut rig, STICK_ROW);
    let (bytes, _) = their_box(vec![shed("Shed")], OLD_KEY);
    plant(&rig, CLOUD, &box_file(), bytes, 1);
    press(&mut rig, FillButton::BringBack);
    rig.jobs();
    let Sheet::Found(boxes) = rig.page().sheet else {
        panic!("no found boxes: {:?}", rig.page().sheet);
    };
    let cloud = boxes
        .iter()
        .position(|b| b.place.name.expose() == "OneDrive")
        .unwrap();
    pick(&mut rig, u8::try_from(cloud).unwrap());
    rig.jobs();
    let Sheet::Password(page) = rig.page().sheet else {
        panic!("no password page");
    };
    assert_eq!(
        page.ask,
        PasswordAsk::Open { own: true },
        "the page says so"
    );
    type_password(&mut rig, OLD_KEY);
    rig.jobs();
    let rewritten = box_in(&rig, CLOUD).unwrap();
    assert!(opens(&rewritten, KEY) && !opens(&rewritten, OLD_KEY));
    assert_eq!(
        rows(&rig),
        [("OneDrive".to_owned(), true), ("E: USB".to_owned(), true)]
    );
    press(&mut rig, FillButton::Back);
    assert_eq!(rig.names(), ["Alex", "Bank", "Gate", "Shed"]);
}

#[test]
fn a_wrong_password_tries_again_and_logs_neither_password_nor_place() {
    let mut rig = new_computer();
    plant_sample(&rig, 1);
    let lines = logs::lines_of(|| bring_top(&mut rig, KEY));
    assert_eq!(rig.note(), Some(FillNote::WrongPassword));
    assert!(
        matches!(rig.page().sheet, Sheet::Password(_)),
        "it tries again"
    );
    let leaks = [KEY, "sam", "OneDrive"];
    assert!(
        lines.iter().all(|l| leaks.iter().all(|k| !l.contains(k))),
        "{lines:?}"
    );
    type_password(&mut rig, OLD_KEY);
    rig.jobs();
    assert_eq!(rig.note(), Some(FillNote::Imported));
}

#[test]
fn a_box_too_big_damaged_or_from_a_newer_version_says_so() {
    let cap = usize::try_from(VaultSettings::from_defaults().unwrap().box_cap()).unwrap();
    let head = [&BOX_MARK[..], &[BOX_VERSION]].concat();
    let big = [&head[..], &vec![0; cap]].concat();
    let newer = [&BOX_MARK[..], &[BOX_VERSION + 1], b"later"].concat();
    let junk = [&head[..], b"not a box"].concat();
    let part = BoxPart {
        name: NOTEBOOK_PART.to_owned(),
        bytes: Redacted::new(Zeroizing::new(b"not a notebook".to_vec())),
    };
    let unread = seal(&[part], OLD_KEY).0;
    let cases = [
        (big, FillNote::TooLarge),
        (newer, FillNote::Unsupported),
        (junk, FillNote::Damaged),
        (unread, FillNote::Damaged),
    ];
    for (bytes, note) in cases {
        let mut rig = new_computer();
        plant(&rig, CLOUD, &box_file(), bytes, 1);
        bring_top(&mut rig, OLD_KEY);
        assert_eq!(rig.note(), Some(note));
        let nothing = rig.actor.book.is_empty() && kept(&rig).padlock.is_none();
        assert!(nothing, "{note:?} changes nothing");
    }
}

#[test]
fn bring_back_ends_the_hold_of_a_notebook_that_did_not_read() {
    let mut rig = Rig::new();
    set_up(&mut rig, STICK_ROW);
    let before = kept(&rig);
    let ours = box_in(&rig, STICK).unwrap();
    let mut next = Rig::held(&before);
    next.fakes
        .places
        .plant(&Path::new(STICK).join(box_file()), ours, 1);
    bring_top(&mut next, KEY);
    assert_eq!(next.note(), Some(FillNote::Imported));
    assert_eq!(next.actor.book.len(), 3, "the entries are back");
    assert!(!kept(&next).held, "the hold is gone");
    assert_eq!(
        next.fakes.boxes.calls(BoxCall::Seal),
        1,
        "and the box is written"
    );
}
