//! A `KeePass` import over the actor: the invitation's and the Backup page's Import, the folder list and the password,
//! one save, a rewrite of the boxes and the hold; `import_notes` tests what it says, and `import_back` its way back.

use std::path::Path;
use std::sync::Arc;

use kx_module_api::keys;
use kx_module_api::ports::{
    FillAct, FillButton, FillClick, FillNote, KeepassEntry, NoteCounts, PageNote, PasswordAsk,
    Pick, PlaceKind, Sheet, box_file,
};
use kx_platform_fake::{BoxCall, FakeSealedStore, PlaceCall};

use super::super::super::Msg;
use super::super::boxes::{CLOUD, DRIVE};
use super::super::rig::{ROWS, Rig};
use super::bring::new_computer;
use super::{
    CLOUD_ROW, KEY, OLD_KEY, box_in, kept, opens, press, rows, set_up, type_password, waits,
};
use crate::backup::Kept;
use crate::fixture::secret;
use crate::import::keepass_ext;
use crate::store::Store;

/// The `KeePass` file's bytes, and a value no log may hold.
pub(super) const KDBX: &[u8] = b"a KeePass file";
pub(super) const VALUE: &str = "zebra-quartz";

/// The file's name.
pub(super) fn file() -> String {
    format!("old.{}", keepass_ext())
}

/// A login entry of the file.
pub(super) fn login(title: &str, user: &str) -> KeepassEntry {
    KeepassEntry {
        title: secret(title),
        user: secret(user),
        password: secret(VALUE),
        address: secret(""),
        notes: secret(""),
        custom: Vec::new(),
    }
}

/// The file in `DRIVE`, which `OLD_KEY` opens.
pub(super) fn plant_file(rig: &Rig, entries: Vec<KeepassEntry>) {
    rig.fakes.boxes.set_keepass(KDBX, OLD_KEY, entries);
    plant_bytes(rig, KDBX.to_vec());
}

/// `bytes` as the file in `DRIVE`.
pub(super) fn plant_bytes(rig: &Rig, bytes: Vec<u8>) {
    let path = Path::new(DRIVE).join(file());
    rig.fakes.places.plant(&path, bytes, 1);
}

/// The names the last page's folder list shows.
pub(super) fn listed(rig: &Rig) -> Vec<String> {
    let Sheet::Folders(list) = rig.page().sheet else {
        panic!("no folder list: {:?}", rig.page().sheet);
    };
    assert_eq!(list.pick, Pick::Keepass);
    list.rows.iter().map(|r| r.name.expose().clone()).collect()
}

/// Clicks the folder list's row named `name`.
pub(super) fn open_row(rig: &mut Rig, name: &str) {
    let at = listed(rig).iter().position(|n| n == name).unwrap();
    super::pick(rig, u8::try_from(at).unwrap());
}

/// From the drives, opens the drive and picks the file, every job run.
pub(super) fn pick_file(rig: &mut Rig) {
    rig.jobs();
    open_row(rig, "D:");
    rig.jobs();
    open_row(rig, &file());
}

/// Presses Import and brings the file in with `password`, every job run.
pub(super) fn import(rig: &mut Rig, password: &str) {
    press(rig, FillButton::Import);
    pick_file(rig);
    type_password(rig, password);
    rig.jobs();
}

/// `Imported` with its counts.
pub(super) fn imported(added: u32, skipped: u32) -> PageNote {
    PageNote {
        note: FillNote::Imported,
        counts: NoteCounts { added, skipped },
    }
}

/// Gives the actor a notebook store of its own, whose saves count the notebook's alone.
pub(super) fn count_saves(rig: &mut Rig) -> FakeSealedStore {
    let notebook = FakeSealedStore::new();
    rig.actor.store = Store::new(Arc::new(notebook.clone()));
    notebook
}

/// A new computer with the panel open, past the offer's find.
pub(super) fn on_invitation() -> Rig {
    let mut rig = new_computer();
    rig.open();
    rig.jobs();
    rig
}

/// True when the last page asks a `KeePass` file's password.
pub(super) fn asks_password(rig: &Rig) -> bool {
    matches!(
        rig.page().sheet,
        Sheet::Password(p) if p.ask == PasswordAsk::Keepass
    )
}

#[test]
fn the_invitations_import_lists_the_keepass_files_asks_the_password_and_brings_them_in() {
    let mut rig = on_invitation();
    plant_file(&rig, vec![login("Bank", "sam"), login("Shed", "alex")]);
    rig.fakes
        .places
        .plant(&Path::new(DRIVE).join("notes.txt"), b"x".to_vec(), 1);
    press(&mut rig, FillButton::Import);
    rig.jobs();
    assert!(!listed(&rig).is_empty(), "the drives, for a KeePass file");
    open_row(&mut rig, "D:");
    rig.jobs();
    assert_eq!(
        listed(&rig),
        ["Backups".to_owned(), file()],
        "no other file"
    );
    open_row(&mut rig, &file());
    assert!(asks_password(&rig));
    let saves = count_saves(&mut rig);
    type_password(&mut rig, OLD_KEY);
    assert_eq!(
        rig.fakes.boxes.calls(BoxCall::Keepass),
        0,
        "only on the job"
    );
    rig.jobs();
    assert_eq!(rig.fakes.boxes.calls(BoxCall::Keepass), 1);
    assert_eq!(rig.page().note, Some(imported(2, 0)));
    assert_eq!(rig.names(), ["Bank", "Shed"], "the list shows them");
    assert_eq!(saves.saves(), 1, "the notebook saves once");
}

#[test]
fn import_on_a_list_with_entries_does_nothing() {
    let mut rig = Rig::new();
    rig.open();
    rig.jobs();
    press(&mut rig, FillButton::Import);
    assert_eq!(rig.fakes.queued(), 0);
    assert!(matches!(rig.page().sheet, Sheet::List(_)));
}

#[test]
fn the_first_entries_ever_brought_in_make_the_first_offer() {
    let mut rig = Rig::kept(Vec::new(), &Kept::default());
    rig.open();
    rig.jobs();
    plant_file(&rig, vec![login("Bank", "sam")]);
    assert!(rig.notices().is_empty());
    import(&mut rig, OLD_KEY);
    let offered = rig.notices().iter().any(|n| n.key == keys::VAULT_OFFER);
    assert!(offered, "as a first save does");
}

#[test]
fn the_backup_pages_import_rewrites_the_boxes_after_the_wait() {
    let mut rig = Rig::new();
    set_up(&mut rig, CLOUD_ROW);
    plant_file(&rig, vec![login("Shed", "alex")]);
    let (sealed, before) = (
        rig.fakes.boxes.calls(BoxCall::Seal),
        box_in(&rig, CLOUD).unwrap(),
    );
    let saves = count_saves(&mut rig);
    import(&mut rig, OLD_KEY);
    assert_eq!(rig.page().note, Some(imported(1, 0)));
    assert_eq!(
        rows(&rig),
        [("OneDrive".to_owned(), true)],
        "back on its page"
    );
    assert_eq!(saves.saves(), 1, "the notebook saves once");
    assert_eq!(rig.fakes.boxes.calls(BoxCall::Seal), sealed, "not yet");
    rig.advance(waits().0);
    rig.jobs();
    assert_eq!(
        rig.fakes.boxes.calls(BoxCall::Seal),
        sealed + 1,
        "sealed once"
    );
    let after = box_in(&rig, CLOUD).unwrap();
    assert!(after != before && opens(&after, KEY));
    assert!(!kept(&rig).behind);
}

#[test]
fn import_adds_to_a_held_notebook_and_writes_no_box() {
    let mut rig = Rig::new();
    set_up(&mut rig, CLOUD_ROW);
    let (before, ours) = (kept(&rig), box_in(&rig, CLOUD).unwrap());
    let mut next = Rig::held(&before);
    let held = Path::new(CLOUD).join(box_file());
    next.fakes.places.plant(&held, ours.clone(), 1);
    plant_file(&next, vec![login("Shed", "alex")]);
    next.open();
    next.jobs();
    press(&mut next, FillButton::Back);
    press(&mut next, FillButton::Backup);
    import(&mut next, OLD_KEY);
    assert_eq!(next.page().note, Some(imported(1, 0)));
    assert_eq!(next.actor.book.len(), 1, "its entries came in");
    next.advance(waits().0);
    next.jobs();
    assert!(kept(&next).held, "the hold stays");
    assert_eq!(next.fakes.boxes.calls(BoxCall::Seal), 0);
    assert_eq!(next.fakes.places.calls(PlaceCall::Write), 0);
    assert_eq!(box_in(&next, CLOUD), Some(ours), "the box is as it was");
}

#[test]
fn use_this_folder_picks_no_file_on_imports_list() {
    let mut rig = on_invitation();
    plant_file(&rig, Vec::new());
    press(&mut rig, FillButton::Import);
    rig.jobs();
    open_row(&mut rig, "D:");
    rig.jobs();
    let Sheet::Folders(list) = rig.page().sheet else {
        panic!("no folder list");
    };
    assert_eq!(list.rows.last().unwrap().kind, PlaceKind::File);
    press(&mut rig, FillButton::Here);
    assert!(matches!(rig.page().sheet, Sheet::Folders(_)));
    assert_eq!(rig.fakes.queued(), 0, "nor looks at the folder");
}

#[test]
fn an_import_click_from_an_older_view_is_dropped() {
    let mut rig = on_invitation();
    let round = rig.page().round;
    rig.click(FillClick::Rows(ROWS + 1));
    assert_ne!(rig.page().round, round, "a newer view shows");
    let click = FillClick::Button(FillButton::Import);
    rig.send(Msg::Act(FillAct { round, click }));
    assert_eq!(rig.fakes.queued(), 0);
    assert!(matches!(rig.page().sheet, Sheet::List(_)));
}
