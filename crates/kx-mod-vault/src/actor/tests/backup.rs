//! The backup over the actor (design "The backup box"): setup (`setup`), its lists (`lists`), the writes and their
//! waits (`writes`, `large`), the Backup page with the first offer (`page`), the hold (`held`), the memory checks
//! (`checks`), Bring back (`bring`, `beside`), a box from another computer (`foreign`), a `KeePass` import (`import`) and
//! any run of steps (`props`), over the fake boxes and places, each job run when the test says.

mod beside;
mod bring;
mod caret;
mod checks;
mod foreign;
mod from;
mod held;
mod import;
mod import_back;
mod import_notes;
mod large;
mod lists;
mod offer;
mod page;
mod props;
mod room;
mod setup;
mod writes;

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use kx_module_api::ports::{
    BackupAsk, BoxPart, BoxState, Boxes, Calendar, FillButton, FillClick, NOTEBOOK_PART, Padlock,
    PlaceKind, RowButton, Sheet, Spot, box_file,
};
use kx_module_api::{Redacted, Zeroizing};
use kx_platform_fake::{FakeBoxes, FakeRandom};

use super::rig::Rig;
use crate::backup::{BackupStore, Kept};
use crate::book::Notebook;
use crate::entry::{Draft, Stamp};
use crate::fixture::LIMITS;
use crate::settings::VaultSettings;
use crate::store::Store;

/// The backup password these tests set up, another one, and the one of a box another computer made.
pub(super) const KEY: &str = "zebra-quartz-9";
pub(super) const NEW_KEY: &str = "otter-maple-42";
pub(super) const OLD_KEY: &str = "heron-cobalt-7";
/// A day, and a minute.
pub(super) const DAY: Duration = Duration::from_hours(24);
pub(super) const MINUTE: Duration = Duration::from_secs(60);
/// The suggested places' rows: the cloud folder, then the stick.
pub(super) const CLOUD_ROW: u8 = 0;
pub(super) const STICK_ROW: u8 = 1;

/// Clicks `button` on the last view.
pub(super) fn press(rig: &mut Rig, button: FillButton) {
    rig.click(FillClick::Button(button));
}

/// Clicks row `row` of the last view.
pub(super) fn pick(rig: &mut Rig, row: u8) {
    rig.click(FillClick::Row {
        row,
        button: RowButton::Open,
    });
}

/// Types `password` on the password page and presses Done.
pub(super) fn type_password(rig: &mut Rig, password: &str) {
    rig.text(password);
    press(rig, FillButton::Done);
}

/// Opens the panel and sets the backup up in the suggested place `row` with `KEY`, every job run.
pub(super) fn set_up(rig: &mut Rig, row: u8) {
    rig.open();
    press(rig, FillButton::Backup);
    rig.jobs();
    pick(rig, row);
    rig.jobs();
    type_password(rig, KEY);
    rig.jobs();
}

/// Adds the suggested place `row` from the Backup page, every job run.
pub(super) fn add(rig: &mut Rig, row: u8) {
    press(rig, FillButton::AddPlace);
    rig.jobs();
    pick(rig, row);
    rig.jobs();
}

/// The box in `folder`, as the places keep it.
pub(super) fn box_in(rig: &Rig, folder: &str) -> Option<Vec<u8>> {
    rig.fakes.places.file(&Path::new(folder).join(box_file()))
}

/// True when `password` opens the box `bytes`.
pub(super) fn opens(bytes: &[u8], password: &str) -> bool {
    let cfg = VaultSettings::from_defaults().unwrap().box_config();
    let password = Redacted::new(Zeroizing::new(password.to_owned()));
    FakeBoxes::new().start(cfg).open(bytes, &password).is_ok()
}

/// The Backup page's places on the last view, each with true once its box is updated.
pub(super) fn rows(rig: &Rig) -> Vec<(String, bool)> {
    let Sheet::Backup(page) = rig.page().sheet else {
        panic!("not the Backup page: {:?}", rig.page().sheet);
    };
    let row = |r: &kx_module_api::ports::PlaceRow| {
        let updated = matches!(r.state, BoxState::Updated(_));
        (r.place.name.expose().clone(), updated)
    };
    page.places.iter().map(row).collect()
}

/// The place a page names, as a `Spot`.
pub(super) fn spot(kind: PlaceKind, name: &str) -> Spot {
    Spot {
        kind,
        name: Redacted::new(name.to_owned()),
    }
}

/// Saves a new note named `name`, a change the backup writes after its wait.
pub(super) fn save_note(rig: &mut Rig, name: &str) {
    press(rig, FillButton::New);
    rig.click(FillClick::Kind(kx_module_api::ports::EntryKind::Note));
    rig.text(name);
    press(rig, FillButton::Save);
}

/// A rig set up in the cloud folder and the stick, both boxes written, on the Backup page.
pub(super) fn two_places() -> Rig {
    let mut rig = Rig::new();
    set_up(&mut rig, CLOUD_ROW);
    add(&mut rig, STICK_ROW);
    assert_eq!(
        rows(&rig),
        [("OneDrive".to_owned(), true), ("E: USB".to_owned(), true)]
    );
    rig
}

/// The wait after a change, and between looks at a waiting place.
pub(super) fn waits() -> (Duration, Duration) {
    let settings = VaultSettings::from_defaults().unwrap();
    (settings.box_delay(), settings.place_check())
}

/// What the backup's blob in `rig` keeps now.
pub(super) fn kept(rig: &Rig) -> Kept {
    BackupStore::new(Arc::new(rig.sealed.clone()))
        .load()
        .unwrap()
        .kept
}

/// A box another computer sealed with `password` over a notebook of `drafts`, and its padlock.
pub(super) fn their_box(drafts: Vec<Draft>, password: &str) -> (Vec<u8>, Padlock) {
    let (random, mut book) = (FakeRandom::new(99), Notebook::default());
    for d in drafts {
        book.add(d, Stamp::from_unix_ms(1), &random, &LIMITS)
            .unwrap();
    }
    let part = BoxPart {
        name: NOTEBOOK_PART.to_owned(),
        bytes: Redacted::new(Store::part(&book)),
    };
    seal(&[part], password)
}

/// A box another computer sealed with `password` over `parts`, and its padlock.
pub(super) fn seal(parts: &[BoxPart], password: &str) -> (Vec<u8>, Padlock) {
    let settings = VaultSettings::from_defaults().unwrap();
    let mut run = FakeBoxes::new().start(settings.box_config());
    let password = Redacted::new(Zeroizing::new(password.to_owned()));
    let lock = run.padlock(&password, settings.box_work).unwrap();
    (run.seal(&lock, parts).unwrap().expose().clone(), lock)
}

/// Keeps `bytes` as the file `name` in `folder`, last written `days` before the rig's wall time.
pub(super) fn plant(rig: &Rig, folder: &str, name: &str, bytes: Vec<u8>, days: u32) {
    let now = Stamp::of(&rig.calendar.now()).unix_ms();
    let ago = i64::try_from((DAY * days).as_millis()).unwrap();
    let file = Path::new(folder).join(name);
    rig.fakes.places.plant(&file, bytes, now - ago);
}

/// The Backup page's question and checks switch on the last view.
pub(super) fn asked(rig: &Rig) -> (Option<BackupAsk>, bool) {
    let Sheet::Backup(page) = rig.page().sheet else {
        panic!("not the Backup page: {:?}", rig.page().sheet);
    };
    (page.ask, page.checks)
}

/// What the loop waits for once the backup has nothing to do: the next memory check.
pub(super) fn idle(rig: &Rig) -> Option<Duration> {
    let wait = rig.actor.backup.check_wait(Stamp::of(&rig.calendar.now()));
    assert!(wait.is_some(), "a memory check is always ahead");
    wait
}
