//! The jobs a `KeePass` import uses: a folder's `KeePass` files after its folders, and a file read within the cap and
//! opened only in the worker.

use super::*;
use crate::fixture::secret as text;
use crate::import::keepass_ext;
use kx_module_api::ports::{BOX_EXT, KeepassEntry, KeepassImport, MAX_KEEPASS_BYTES};
use kx_platform_fake::PlaceCall;

/// A `KeePass` file's bytes.
const KDBX: &[u8] = b"a KeePass file";
/// A value no `Debug` may show.
const VALUE: &str = "zebra-quartz";

/// A read of the file `name` in `DRIVE` with `password`.
fn read(name: &str, password: &str) -> Job {
    Job::Keepass {
        file: Redacted::new(Path::new(DRIVE).join(name)),
        password: secret(password),
    }
}

/// The file's one entry.
fn entries() -> Vec<KeepassEntry> {
    vec![KeepassEntry {
        title: text("Bank"),
        user: text("sam"),
        password: text(VALUE),
        address: text(""),
        notes: text(""),
        custom: Vec::new(),
    }]
}

/// The fake drive holding `bytes` as `old.kdbx`.
fn planted(places: &FakePlaces, bytes: Vec<u8>) -> String {
    let name = format!("old.{}", keepass_ext());
    places.plant(&Path::new(DRIVE).join(&name), bytes, 7);
    name
}

#[test]
fn a_keepass_list_has_the_folders_then_the_keepass_files_alone() {
    let (boxes, places) = fakes();
    places.add_folder(&Path::new(DRIVE).join("Old"));
    planted(&places, KDBX.to_vec());
    let backup = Path::new(DRIVE).join(format!("laptop.{BOX_EXT}"));
    places.plant(&backup, b"x".to_vec(), 1);
    places.plant(&Path::new(DRIVE).join("notes.txt"), b"x".to_vec(), 1);
    let mut runner = runner(&boxes, &places);
    let Answer::Folders(Ok(rows)) = runner.answer(Job::Folders {
        folder: Some(Redacted::new(PathBuf::from(DRIVE))),
        files: Some(keepass_ext()),
    }) else {
        panic!("no list");
    };
    let names: Vec<_> = rows
        .iter()
        .map(|r| (r.kind, r.name.expose().clone()))
        .collect();
    let file = format!("old.{}", keepass_ext());
    let want = [
        (PlaceKind::Folder, "Old".to_owned()),
        (PlaceKind::File, file),
    ];
    assert_eq!(names, want);
    assert_eq!(places.calls(PlaceCall::Files), 1);
}

#[test]
fn a_keepass_read_opens_the_file_in_the_worker_with_its_password() {
    let (boxes, places) = fakes();
    let name = planted(&places, KDBX.to_vec());
    boxes.set_keepass(KDBX, PASSWORD, entries());
    let mut runner = runner(&boxes, &places);
    let opened = runner.answer(read(&name, PASSWORD));
    assert_eq!(opened, Answer::Keepass(Ok(KeepassImport::from(entries()))));
    let wrong = runner.answer(read(&name, "wrong"));
    let refused = Answer::Keepass(Err(Shut::Open(BoxError::WrongPassword)));
    assert_eq!(wrong, refused);
    assert_eq!(boxes.calls(BoxCall::Keepass), 2);
}

#[test]
fn a_keepass_file_past_the_cap_never_reaches_the_worker() {
    let (boxes, places) = fakes();
    let name = planted(&places, vec![0; MAX_KEEPASS_BYTES + 1]);
    let mut runner = runner(&boxes, &places);
    let big = runner.answer(read(&name, PASSWORD));
    assert_eq!(big, Answer::Keepass(Err(Shut::Read(PlaceError::TooLarge))));
    assert_eq!(boxes.calls(BoxCall::Keepass), 0);
    assert_eq!(places.calls(PlaceCall::Read), 1);
}

#[test]
fn a_keepass_file_exactly_at_the_cap_reaches_the_worker() {
    let (boxes, places) = fakes();
    let bytes = vec![0; MAX_KEEPASS_BYTES];
    let name = planted(&places, bytes.clone());
    boxes.set_keepass(&bytes, PASSWORD, entries());
    let mut runner = runner(&boxes, &places);
    let opened = runner.answer(read(&name, PASSWORD));
    assert_eq!(opened, Answer::Keepass(Ok(KeepassImport::from(entries()))));
}

#[test]
fn a_keepass_file_gone_says_so_and_never_reaches_the_worker() {
    let (boxes, places) = fakes();
    let mut runner = runner(&boxes, &places);
    let gone = runner.answer(read("gone.kdbx", PASSWORD));
    assert_eq!(gone, Answer::Keepass(Err(Shut::Read(PlaceError::Missing))));
    assert_eq!(boxes.calls(BoxCall::Keepass), 0);
}

#[test]
fn a_panicking_keepass_job_answers_failed() {
    assert_eq!(
        read("old.kdbx", PASSWORD).failed(),
        Answer::Keepass(Err(Shut::Open(BoxError::Failed)))
    );
}

#[test]
fn a_keepass_job_and_its_answer_never_show_a_value_in_debug() {
    let job = format!("{:?}", read("sam-old.kdbx", VALUE));
    assert!(!job.contains(VALUE) && !job.contains("sam-old"), "{job}");
    let answer = Answer::Keepass(Ok(KeepassImport::from(entries())));
    let shown = format!("{answer:?}");
    assert!(!shown.contains(VALUE) && !shown.contains("Bank"), "{shown}");
}
