//! The jobs Bring back and the memory checks use: the boxes found, a folder's backups after its folders, a place's
//! state, a box read within the cap and opened, and a password checked against the padlock.

use super::*;
use kx_module_api::ports::{BOX_EXT, BoxPart, NOTEBOOK_PART, PlaceState};

/// A box sealed over `notebook` to a new padlock that `PASSWORD` locks.
fn sealed(runner: &mut Runner, notebook: &[u8]) -> (Vec<u8>, Padlock) {
    let lock = padlock(runner);
    let part = BoxPart {
        name: NOTEBOOK_PART.to_owned(),
        bytes: Redacted::new(Zeroizing::new(notebook.to_vec())),
    };
    let mut run = FakeBoxes::new().start(CFG);
    (run.seal(&lock, &[part]).unwrap().expose().clone(), lock)
}

/// An open of the box in `folder` with `password`, capped at `cap`.
fn open(folder: &str, password: &str, cap: u64) -> Job {
    Job::Open {
        file: Redacted::new(Path::new(folder).join(box_file())),
        cap,
        password: secret(password),
    }
}

#[test]
fn an_open_reads_the_box_within_the_cap_and_opens_it() {
    let (boxes, places) = fakes();
    let mut runner = runner(&boxes, &places);
    let (bytes, lock) = sealed(&mut runner, b"notes");
    places.plant(&Path::new(CLOUD).join(box_file()), bytes.clone(), 7);
    let Answer::Opened(Ok(unboxed)) = runner.answer(open(CLOUD, PASSWORD, MB)) else {
        panic!("not opened");
    };
    assert_eq!(unboxed.opened.padlock, lock);
    assert_eq!(unboxed.opened.parts[0].bytes.expose().as_slice(), b"notes");
    let len = u64::try_from(bytes.len()).unwrap();
    assert_eq!((unboxed.mark, unboxed.len), (mark(&bytes), len));
    let wrong = runner.answer(open(CLOUD, "wrong", MB));
    assert_eq!(
        wrong,
        Answer::Opened(Err(Shut::Open(BoxError::WrongPassword)))
    );
    let big = runner.answer(open(CLOUD, PASSWORD, 1));
    assert_eq!(big, Answer::Opened(Err(Shut::Read(PlaceError::TooLarge))));
    let gone = runner.answer(open(STICK, PASSWORD, MB));
    assert_eq!(gone, Answer::Opened(Err(Shut::Read(PlaceError::Missing))));
}

#[test]
fn a_password_is_checked_against_the_padlock() {
    let (boxes, places) = fakes();
    let mut runner = runner(&boxes, &places);
    let lock = padlock(&mut runner);
    let check = |r: &mut Runner, password| {
        let (padlock, password) = (lock.clone(), secret(password));
        r.answer(Job::Check { padlock, password })
    };
    assert_eq!(check(&mut runner, PASSWORD), Answer::Checked(Ok(true)));
    assert_eq!(check(&mut runner, "wrong"), Answer::Checked(Ok(false)));
    assert_eq!(boxes.calls(BoxCall::Check), 2);
}

#[test]
fn a_backup_list_has_the_folders_then_the_backups_and_the_drives_none() {
    let (boxes, places) = fakes();
    places.add_folder(&Path::new(DRIVE).join("Old"));
    places.plant(
        &Path::new(DRIVE).join(format!("laptop.{BOX_EXT}")),
        b"x".to_vec(),
        1,
    );
    places.plant(&Path::new(DRIVE).join("notes.txt"), b"x".to_vec(), 1);
    let mut runner = runner(&boxes, &places);
    let folder = Some(Redacted::new(PathBuf::from(DRIVE)));
    let Answer::Folders(Ok(rows)) = runner.answer(Job::Folders {
        folder,
        files: Some(BOX_EXT),
    }) else {
        panic!("no list");
    };
    let kinds: Vec<_> = rows.iter().map(|r| r.kind).collect();
    assert_eq!(kinds, [PlaceKind::Folder, PlaceKind::File]);
    let drives = runner.answer(Job::Folders {
        folder: None,
        files: Some(BOX_EXT),
    });
    assert!(
        matches!(drives, Answer::Folders(Ok(d)) if d.iter().all(|p| p.kind != PlaceKind::File))
    );
}

#[test]
fn the_boxes_found_and_a_places_state_are_jobs() {
    let (boxes, places) = fakes();
    let held = [kx_module_api::ports::BOX_MARK.as_slice(), b"x"].concat();
    places.plant(&Path::new(CLOUD).join(box_file()), held, 3);
    places.set_data_dir(Path::new(DRIVE));
    let mut runner = runner(&boxes, &places);
    let Answer::Found(found) = runner.answer(Job::Find) else {
        panic!("nothing found");
    };
    assert_eq!(found.len(), 1);
    let state = |r: &mut Runner, f: &str| {
        let folder = Redacted::new(PathBuf::from(f));
        r.answer(Job::State { folder })
    };
    assert_eq!(state(&mut runner, CLOUD), Answer::State(PlaceState::Ready));
    assert_eq!(
        state(&mut runner, DRIVE),
        Answer::State(PlaceState::Refused)
    );
}

#[test]
fn an_open_or_a_check_never_shows_its_password_in_debug() {
    let (padlock, password) = (
        Padlock {
            recipient: Redacted::new("r".to_owned()),
            locked: Redacted::new(Vec::new()),
        },
        secret(PASSWORD),
    );
    let check = format!("{:?}", Job::Check { padlock, password });
    let open = format!("{:?}", open(CLOUD, PASSWORD, MB));
    assert!(!check.contains(PASSWORD) && !open.contains(PASSWORD));
    assert!(!open.contains("OneDrive"), "nor its path");
}

#[test]
fn a_look_dates_a_box_where_no_find_reaches_and_runs_no_find() {
    let (boxes, places) = fakes();
    let mut runner = runner(&boxes, &places);
    let folder = Path::new(DRIVE).join("deep").join("deeper");
    places.plant(&folder.join(box_file()), b"a box".to_vec(), 42);
    let look = runner.answer(Job::Look {
        folder: Redacted::new(folder),
        cap: MB,
    });
    let Answer::Look(Look::Box { modified_ms, .. }) = look else {
        panic!("no box: {look:?}");
    };
    assert_eq!(modified_ms, Some(42));
    assert_eq!(places.calls(PlaceCall::Find), 0);
}
