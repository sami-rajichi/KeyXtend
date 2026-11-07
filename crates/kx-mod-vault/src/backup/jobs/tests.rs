//! The jobs over the fake boxes and places: one seal per write, missing and full folders skipped, the cap, a failed
//! write keeping the old box, and the look at a picked folder; `bring` tests the opens, checks and finds, and `thread`
//! the job thread.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use kx_module_api::ports::{
    BoxConfig, BoxError, Boxes, MB, Padlock, PlaceError, PlaceKind, box_file,
};
use kx_module_api::{Redacted, Secret, Zeroizing};
use kx_platform_fake::{BoxCall, FakeBoxes, FakePlaces, PlaceCall};

use super::*;
use crate::backup::mark::mark;

mod bring;
mod keepass;

/// The vault job's config in these tests.
pub(super) const CFG: BoxConfig = BoxConfig {
    deadline_ms: 20_000,
    idle_s: 30,
    restart_ms: 1000,
    restart_max_ms: 30_000,
};
/// The test's backup password.
pub(super) const PASSWORD: &str = "correct horse";
/// How long a test waits for the job thread.
pub(super) const WITHIN: Duration = Duration::from_secs(10);
/// The test folders, with slashes, which every OS reads: a cloud folder, a stick and a drive.
pub(super) const CLOUD: &str = "drive-c/OneDrive";
pub(super) const STICK: &str = "usb-e";
pub(super) const DRIVE: &str = "drive-d";

/// `text` as a secret.
pub(super) fn secret(text: &str) -> Secret {
    Redacted::new(Zeroizing::new(text.to_owned()))
}

/// The fakes, with three folders: a cloud folder, a stick and a drive.
pub(super) fn fakes() -> (FakeBoxes, FakePlaces) {
    let places = FakePlaces::new();
    places.add(PlaceKind::Cloud, "OneDrive", Path::new(CLOUD));
    places.add(PlaceKind::Removable, "E:", Path::new(STICK));
    places.add(PlaceKind::Drive, "D:", Path::new(DRIVE));
    (FakeBoxes::new(), places)
}

/// A runner over the fakes.
pub(super) fn runner(boxes: &FakeBoxes, places: &FakePlaces) -> Runner {
    Runner::new(boxes.start(CFG), Arc::new(places.clone()))
}

/// A padlock the fake made.
pub(super) fn padlock(runner: &mut Runner) -> Padlock {
    let job = Job::Padlock {
        password: secret(PASSWORD),
        work: 18,
    };
    match runner.answer(job) {
        Answer::Padlock(Ok(padlock)) => padlock,
        other => panic!("{other:?}"),
    }
}

/// A write of `notebook` to `folders`, capped at 16 MB, which a folder takes with one byte free.
fn write(padlock: Padlock, notebook: &[u8], folders: &[&str]) -> Job {
    Job::Write(Write {
        padlock,
        notebook: Redacted::new(Zeroizing::new(notebook.to_vec())),
        folders: folders
            .iter()
            .map(|f| Redacted::new(PathBuf::from(f)))
            .collect(),
        cap: 16 * MB,
        need: 1,
    })
}

/// The box `places` keeps in `folder`.
fn box_in(places: &FakePlaces, folder: &str) -> Option<Vec<u8>> {
    places.file(&Path::new(folder).join(box_file()))
}

#[test]
fn a_write_seals_once_and_writes_every_folder_that_can_take_it() {
    let ((boxes, places), all) = (fakes(), [CLOUD, STICK, DRIVE]);
    places.vanish(Path::new(STICK));
    places.set_free(Path::new(DRIVE), 0);
    let mut runner = runner(&boxes, &places);
    let lock = padlock(&mut runner);
    let Answer::Wrote(Ok(wrote)) = runner.answer(write(lock, b"notes", &all)) else {
        panic!("not written");
    };
    let missing = [Ok(()), Err(PlaceError::Missing), Err(PlaceError::Full)];
    assert_eq!(wrote.reached, missing);
    assert_eq!(boxes.calls(BoxCall::Seal), 1);
    assert_eq!(places.calls(PlaceCall::Write), 1, "only the ready folder");
    let written = box_in(&places, CLOUD).unwrap();
    let len = u64::try_from(written.len()).unwrap();
    assert_eq!(wrote.sealed, Some((mark(&written), len)));
    assert_eq!(box_in(&places, STICK), None);
}

#[test]
fn no_folder_ready_seals_nothing() {
    let (boxes, places) = fakes();
    places.vanish(Path::new(STICK));
    let mut runner = runner(&boxes, &places);
    let lock = padlock(&mut runner);
    let answer = runner.answer(write(lock, b"notes", &[STICK]));
    let none = Wrote {
        sealed: None,
        reached: vec![Err(PlaceError::Missing)],
    };
    assert_eq!(answer, Answer::Wrote(Ok(none)));
    assert_eq!(boxes.calls(BoxCall::Seal), 0);
}

#[test]
fn a_box_over_the_cap_is_not_written_and_the_old_box_stays() {
    let (boxes, places) = fakes();
    places.plant(&Path::new(DRIVE).join(box_file()), b"old".to_vec(), 1);
    let mut runner = runner(&boxes, &places);
    let lock = padlock(&mut runner);
    let Job::Write(mut job) = write(lock, b"notes", &[DRIVE]) else {
        unreachable!()
    };
    job.cap = 4;
    let answer = runner.answer(Job::Write(job));
    assert_eq!(answer, Answer::Wrote(Err(BoxError::TooLarge)));
    assert_eq!(box_in(&places, DRIVE), Some(b"old".to_vec()));
}

#[test]
fn a_failed_write_keeps_the_old_box() {
    let (boxes, places) = fakes();
    places.plant(&Path::new(DRIVE).join(box_file()), b"old".to_vec(), 1);
    places.fail(PlaceCall::Write, PlaceError::Io);
    let mut runner = runner(&boxes, &places);
    let lock = padlock(&mut runner);
    let Answer::Wrote(Ok(wrote)) = runner.answer(write(lock, b"notes", &[DRIVE])) else {
        panic!("not sealed");
    };
    assert_eq!(wrote.reached, [Err(PlaceError::Io)]);
    assert_eq!(box_in(&places, DRIVE), Some(b"old".to_vec()));
}

#[test]
fn a_failed_seal_says_so_and_writes_nothing() {
    let (boxes, places) = fakes();
    let mut runner = runner(&boxes, &places);
    let lock = padlock(&mut runner);
    boxes.fail(BoxCall::Seal, BoxError::TimedOut);
    let answer = runner.answer(write(lock, b"notes", &[DRIVE]));
    assert_eq!(answer, Answer::Wrote(Err(BoxError::TimedOut)));
    assert_eq!(places.calls(PlaceCall::Write), 0);
}

#[test]
fn a_look_tells_no_box_a_box_by_its_mark_one_too_large_and_an_unread_folder() {
    let (boxes, places) = fakes();
    let mut runner = runner(&boxes, &places);
    let look = |r: &mut Runner, cap| {
        r.answer(Job::Look {
            folder: Redacted::new(PathBuf::from(CLOUD)),
            cap,
        })
    };
    assert_eq!(look(&mut runner, MB), Answer::Look(Look::Empty));
    let file = Path::new(CLOUD).join(box_file());
    let held = [kx_module_api::ports::BOX_MARK.as_slice(), b"x"].concat();
    places.plant(&file, held.clone(), 42);
    let seen = Look::Box {
        mark: Some(mark(&held)),
        modified_ms: Some(42),
    };
    assert_eq!(look(&mut runner, MB), Answer::Look(seen));
    let big = Look::Box {
        mark: None,
        modified_ms: Some(42),
    };
    assert_eq!(look(&mut runner, 1), Answer::Look(big));
    places.fail(PlaceCall::Read, PlaceError::Io);
    let unread = Look::Unread(PlaceError::Io);
    assert_eq!(look(&mut runner, MB), Answer::Look(unread));
}

#[test]
fn a_password_never_shows_in_a_jobs_debug() {
    let ask = Ask {
        round: 1,
        job: Job::Padlock {
            password: secret(PASSWORD),
            work: 18,
        },
    };
    assert!(!format!("{ask:?}").contains(PASSWORD));
}

#[test]
fn a_look_at_the_data_folder_is_refused_before_it_reads() {
    let (boxes, places) = fakes();
    places.set_data_dir(Path::new(DRIVE));
    let mut runner = runner(&boxes, &places);
    let look = Job::Look {
        folder: Redacted::new(PathBuf::from(DRIVE)),
        cap: MB,
    };
    let refused = Answer::Look(Look::Unread(PlaceError::Refused));
    assert_eq!(runner.answer(look), refused);
    assert_eq!(places.calls(PlaceCall::Read), 0);
}

#[test]
fn the_places_and_the_folders_are_listed_as_jobs() {
    let (boxes, places) = fakes();
    let mut runner = runner(&boxes, &places);
    let Answer::Suggested(spots) = runner.answer(Job::Suggested) else {
        panic!("no places");
    };
    assert_eq!(spots.len(), 2, "the cloud folder and the stick");
    let Answer::Folders(Ok(drives)) = runner.answer(Job::Folders {
        folder: None,
        files: None,
    }) else {
        panic!("no drives");
    };
    assert_eq!(drives.len(), 2, "the stick and the drive");
    places.fail(PlaceCall::Folders, PlaceError::Io);
    let folder = Some(Redacted::new(PathBuf::from(DRIVE)));
    let failed = Answer::Folders(Err(PlaceError::Io));
    let files = None;
    assert_eq!(runner.answer(Job::Folders { folder, files }), failed);
}
