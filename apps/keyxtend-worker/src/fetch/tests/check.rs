//! The final check (edge 20): a bad fingerprint or a size other than the row's deletes the part and names nothing; a
//! body cut short keeps it. Progress is paced, and a folder that cannot take the part is a disk fault.

use super::super::get::{BUF, MIN_STEP, STEPS};
use super::*;
use std::path::MAIN_SEPARATOR_STR;

#[test]
fn a_bad_fingerprint_deletes_the_part_and_names_nothing() {
    let (dir, server) = (Dir::new(), Server::new([whole(FILE)]));
    let (mut job, mut out) = (job(&server), Vec::new());
    let mut ask = get(1, &dir);
    if let FetchAsk::Get { sha256, .. } = &mut ask {
        *sha256 = print(b"another file");
    }
    take(&mut job, ask, &mut out);
    drive(&mut job, &mut out);
    assert_eq!(
        out,
        [
            progress(1, 0),
            progress(1, 40),
            fault(1, FetchFault::BadHash)
        ]
    );
    assert_eq!(dir.files(), (None, None));
}

#[test]
fn a_complete_part_with_a_bad_fingerprint_is_deleted_without_asking() {
    let dir = Dir::new();
    fs::write(dir.part(), FILE.to_ascii_uppercase()).unwrap();
    let server = Server::new([]);
    let (mut job, mut out) = (job(&server), Vec::new());
    take(&mut job, get(1, &dir), &mut out);
    drive(&mut job, &mut out);
    assert_eq!(out, [progress(1, 40), fault(1, FetchFault::BadHash)]);
    assert!(server.seen().is_empty());
    assert_eq!(dir.files(), (None, None));
}

#[test]
fn a_server_size_other_than_the_rows_deletes_the_part() {
    let (dir, server) = (Dir::new(), Server::new([whole(&[FILE, b"x"].concat())]));
    let (mut job, mut out) = (job(&server), Vec::new());
    take(&mut job, get(1, &dir), &mut out);
    drive(&mut job, &mut out);
    assert_eq!(out, [progress(1, 0), fault(1, FetchFault::BadSize)]);
    assert_eq!(dir.files(), (None, None));
    fs::write(dir.part(), &FILE[..20]).unwrap();
    server.0.push(Act::part(&FILE[20..], 20, 41));
    take(&mut job, get(2, &dir), &mut out);
    drive(&mut job, &mut out);
    assert_eq!(out.last(), Some(&fault(2, FetchFault::BadSize)));
    assert_eq!(dir.files(), (None, None), "a resumed part goes too");
}

#[test]
fn a_body_that_ends_short_is_a_cut_that_keeps_the_part() {
    let short = Act::Answer(Head::whole(FILE.len()), reads(&FILE[..20]));
    let (dir, server) = (Dir::new(), Server::new([short]));
    let (mut job, mut out) = (job(&server), Vec::new());
    take(&mut job, get(1, &dir), &mut out);
    drive(&mut job, &mut out);
    assert_eq!(out, [progress(1, 0), fault(1, FetchFault::Net)]);
    assert_eq!(dir.files(), (None, Some(FILE[..20].to_vec())));
}

#[test]
fn progress_is_paced_not_told_per_read() {
    let big: Vec<u8> = (0..4 * MIN_STEP + 7)
        .map(|i| u8::try_from(i % 251).unwrap())
        .collect();
    let chunks = big.chunks(BUF).map(|c| Step::Bytes(c.to_vec())).collect();
    let server = Server::new([Act::Answer(Head::whole(big.len()), chunks)]);
    let (dir, mut out) = (Dir::new(), Vec::new());
    let mut job = job(&server);
    take(&mut job, get_of(1, &dir, &[URL], &big), &mut out);
    drive(&mut job, &mut out);
    let total = big.len() as u64;
    let gots: Vec<u64> = out
        .iter()
        .filter_map(|n| match n {
            FetchNote::Progress { got, total: t, .. } if *t == total => Some(*got),
            _ => None,
        })
        .collect();
    let most = usize::try_from(total / MIN_STEP.max(total / STEPS)).unwrap() + 2;
    assert!(
        gots.len() <= most,
        "{} notes for {} reads",
        gots.len(),
        big.len() / BUF
    );
    assert!(gots.is_sorted() && gots.first() == Some(&0) && gots.last() == Some(&total));
    assert_eq!(out.last(), Some(&done(1)));
    assert_eq!(fs::read(dir.file()).unwrap(), big);
}

/// A download of `FILE` into `path` in place of `dir`'s file.
fn get_into(id: u32, dir: &Dir, path: String) -> FetchAsk<'static> {
    let mut ask = get(id, dir);
    if let FetchAsk::Get { path: at, .. } = &mut ask {
        *at = Cow::Owned(path);
    }
    ask
}

#[test]
fn a_path_the_part_cannot_sit_beside_is_a_disk_fault_before_any_request() {
    let dir = Dir::new();
    let text = |p: PathBuf| p.to_string_lossy().into_owned();
    let paths = [
        text(dir.0.join("gone").join("m.gguf")),
        "m.gguf".to_owned(),
        text(dir.0.join("gone").join("..").join("m.gguf")),
        format!("{}{MAIN_SEPARATOR_STR}", dir.0.display()),
    ];
    let server = Server::new([whole(FILE)]);
    let (mut job, mut out) = (job(&server), Vec::new());
    for (id, path) in (1..).zip(paths) {
        take(&mut job, get_into(id, &dir, path), &mut out);
        assert!(!job.busy());
        assert_eq!(out.pop(), Some(fault(id, FetchFault::Disk)), "{id}");
    }
    assert!(server.seen().is_empty());
    assert!(!Path::new("m.gguf.part").exists());
    assert_eq!(dir.files(), (None, None), "nothing climbs into the folder");
    assert_eq!(fs::read_dir(&dir.0).unwrap().count(), 0);
}

#[test]
fn a_name_already_taken_is_never_replaced() {
    let (dir, server) = (Dir::new(), Server::new([whole(FILE)]));
    let (mut job, mut out) = (job(&server), Vec::new());
    fs::write(dir.file(), b"the user's own file").unwrap();
    take(&mut job, get(1, &dir), &mut out);
    assert_eq!(out, [fault(1, FetchFault::Disk)], "refused at once");
    assert!(server.seen().is_empty());
    fs::remove_file(dir.file()).unwrap();
    take(&mut job, get(2, &dir), &mut out);
    fs::write(dir.file(), b"made meanwhile").unwrap();
    drive(&mut job, &mut out);
    assert_eq!(out.last(), Some(&fault(2, FetchFault::Disk)));
    let kept = (Some(b"made meanwhile".to_vec()), Some(FILE.to_vec()));
    assert_eq!(dir.files(), kept, "the checked part waits");
    take(&mut job, FetchAsk::Cancel { id: 2 }, &mut out);
    assert_eq!(dir.files().1, None, "a cancel deletes it");
}

#[test]
fn a_folder_or_a_link_at_the_parts_name_is_never_followed() {
    let dir = Dir::new();
    let server = Server::new([whole(FILE)]);
    let (mut job, mut out) = (job(&server), Vec::new());
    fs::create_dir(dir.part()).unwrap();
    take(&mut job, get(1, &dir), &mut out);
    assert_eq!(out, [fault(1, FetchFault::Disk)]);
    fs::remove_dir(dir.part()).unwrap();
    let target = dir.0.join("target");
    fs::write(&target, b"another file").unwrap();
    if link(&target, &dir.part()) {
        take(&mut job, get(2, &dir), &mut out);
        assert_eq!(out.last(), Some(&fault(2, FetchFault::Disk)));
        assert_eq!(fs::read(&target).unwrap(), b"another file");
    }
    assert!(server.seen().is_empty());
}

/// Makes `at` a link to the file `target`; false where the system refuses, as Windows does outside developer mode.
fn link(target: &Path, at: &Path) -> bool {
    #[cfg(windows)]
    let made = std::os::windows::fs::symlink_file(target, at);
    #[cfg(unix)]
    let made = std::os::unix::fs::symlink(target, at);
    made.is_ok()
}
