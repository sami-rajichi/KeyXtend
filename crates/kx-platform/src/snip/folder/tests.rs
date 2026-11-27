//! The Snips folder on a real temp folder: its pictures and videos in the unsaved part with their ages and copy marks,
//! marks and deletes there only, never through a planted link, the drive's room, and a hidden run with none.

use super::*;
use crate::{SNIP_DIR, UNSAVED_DIR};
use kx_test_support::tempdir::TempDir;
use std::path::PathBuf;
use std::time::Duration;

/// The room the fake drive tells.
const FREE: u64 = 7 << 30;

/// A drive with `FREE` bytes, or none.
struct Drive(Option<u64>);

impl Room for Drive {
    fn free(&self, _: &Path) -> io::Result<u64> {
        self.0.ok_or_else(|| io::ErrorKind::NotFound.into())
    }
}

/// A temp folder holding the Snips folder and its unsaved part, and the adapter over it.
struct Bed {
    _dir: TempDir,
    unsaved: PathBuf,
    snips: DiskSnips,
}

impl Bed {
    fn new() -> Self {
        let dir = TempDir::new("snips").unwrap();
        let at = dir.path().join(SNIP_DIR);
        let unsaved = at.join(UNSAVED_DIR);
        fs::create_dir_all(&unsaved).unwrap();
        let at = FolderPath::new(at).unwrap();
        let snips = DiskSnips::new(Some(at), Arc::new(Drive(Some(FREE))));
        Self {
            _dir: dir,
            unsaved,
            snips,
        }
    }

    /// Writes `name` in the unsaved part.
    fn put(&self, name: &str) -> PathBuf {
        let path = self.unsaved.join(name);
        fs::write(&path, b"x").unwrap();
        path
    }
}

/// A hidden run's adapter.
fn hidden() -> DiskSnips {
    DiskSnips::new(None, Arc::new(Drive(Some(FREE))))
}

#[test]
fn the_unsaved_part_lists_its_pictures_and_videos_alone_with_their_ages() {
    let bed = Bed::new();
    let (png, mp4) = (bed.put("1.png"), bed.put("2.MP4"));
    bed.put("notes.txt");
    fs::create_dir(bed.unsaved.join("3.mp4")).unwrap();
    let mut listed = bed.snips.unsaved().unwrap();
    listed.sort_by(|a, b| a.file.expose().cmp(b.file.expose()));
    let files: Vec<&Path> = listed.iter().map(|u| u.file.expose()).collect();
    assert_eq!(files, [png.as_path(), mp4.as_path()]);
    assert_eq!(listed[1].file.kind(), FileKind::Mp4);
    assert!(listed.iter().all(|u| u.age < Duration::from_secs(3_600)));
}

#[test]
fn a_gone_part_is_missing_and_a_hidden_run_has_no_folder() {
    let bed = Bed::new();
    fs::remove_dir(&bed.unsaved).unwrap();
    assert_eq!(bed.snips.unsaved(), Err(SaveError::Missing));
    assert_eq!(hidden().snips(), None);
    assert_eq!(hidden().unsaved(), Err(SaveError::NoFolder));
    assert_eq!(hidden().free(), Err(SaveError::NoFolder));
}

#[test]
fn a_delete_takes_a_file_of_the_unsaved_part_alone_and_a_gone_one_is_no_error() {
    let bed = Bed::new();
    let file = SnipFile::new(bed.put("1.mp4"), FileKind::Mp4).unwrap();
    assert_eq!(bed.snips.remove(&file), Ok(()));
    assert!(!file.expose().exists());
    assert_eq!(bed.snips.remove(&file), Ok(()), "already gone");
    let kept = bed.unsaved.parent().unwrap().join("2.mp4");
    fs::write(&kept, b"x").unwrap();
    let elsewhere = SnipFile::new(kept.clone(), FileKind::Mp4).unwrap();
    assert_eq!(bed.snips.remove(&elsewhere), Err(SaveError::BadPath));
    assert!(kept.exists());
    assert_eq!(hidden().remove(&file), Err(SaveError::NoFolder));
}

#[test]
fn a_copy_mark_is_a_file_beside_its_video_listed_with_its_age_and_deleted_first() {
    let bed = Bed::new();
    let video = SnipFile::new(bed.put("1.mp4"), FileKind::Mp4).unwrap();
    let plain = SnipFile::new(bed.put("2.mp4"), FileKind::Mp4).unwrap();
    assert_eq!(bed.snips.mark_copied(&video), Ok(()));
    let mark = bed.unsaved.join("1.mp4.copied");
    assert!(mark.is_file(), "beside it, the video kept its name");
    assert!(video.expose().is_file());
    let mut listed = bed.snips.unsaved().unwrap();
    listed.sort_by(|a, b| a.file.expose().cmp(b.file.expose()));
    let marks: Vec<_> = listed.iter().map(|u| u.copied.is_some()).collect();
    assert_eq!(marks, [true, false], "the mark itself is not listed");
    assert!(listed[0].copied < Some(Duration::from_secs(3_600)));
    assert_eq!(bed.snips.remove(&video), Ok(()));
    assert!(!mark.exists() && !video.expose().exists());
    assert_eq!(bed.snips.remove(&plain), Ok(()));
}

#[test]
fn a_mark_is_made_only_on_a_video_of_the_unsaved_part_that_is_there() {
    let bed = Bed::new();
    let picture = SnipFile::new(bed.put("1.png"), FileKind::Png).unwrap();
    let gone = SnipFile::new(bed.unsaved.join("2.mp4"), FileKind::Mp4).unwrap();
    let kept = bed.unsaved.parent().unwrap().join("3.mp4");
    fs::write(&kept, b"x").unwrap();
    let elsewhere = SnipFile::new(kept, FileKind::Mp4).unwrap();
    assert_eq!(bed.snips.mark_copied(&picture), Err(SaveError::BadPath));
    assert_eq!(bed.snips.mark_copied(&elsewhere), Err(SaveError::BadPath));
    assert_eq!(bed.snips.mark_copied(&gone), Err(SaveError::Missing));
    assert_eq!(hidden().mark_copied(&gone), Err(SaveError::NoFolder));
    assert_eq!(hidden().prune_marks(), Err(SaveError::NoFolder));
    let held = fs::read_dir(&bed.unsaved).unwrap().count();
    assert_eq!(held, 1, "the picture alone: no mark was written");
}

/// What the file outside the unsaved part holds.
const OUTSIDE: &[u8] = b"outside";

/// Plants a symbolic link to `target` at `at`; false where the system refuses one.
#[cfg(windows)]
fn symlink(target: &Path, at: &Path) -> bool {
    std::os::windows::fs::symlink_file(target, at).is_ok()
}

/// Plants a symbolic link to `target` at `at`; false where the system refuses one.
#[cfg(unix)]
fn symlink(target: &Path, at: &Path) -> bool {
    std::os::unix::fs::symlink(target, at).is_ok()
}

/// A way to plant something at the mark's path that may reach `target`, by name.
type Plant = (&'static str, fn(&Path, &Path) -> bool);

#[test]
fn a_link_or_file_planted_at_the_marks_path_is_replaced_and_what_it_reached_stays() {
    let bed = Bed::new();
    let video = SnipFile::new(bed.put("1.mp4"), FileKind::Mp4).unwrap();
    let mark = bed.unsaved.join("1.mp4.copied");
    let outside = bed.unsaved.parent().unwrap().join("mine.txt");
    fs::write(&outside, OUTSIDE).unwrap();
    let plants: [Plant; 3] = [
        ("a hard link", |target, at| {
            fs::hard_link(target, at).is_ok()
        }),
        ("a symbolic link", symlink),
        ("an old mark", |_, at| fs::write(at, b"old").is_ok()),
    ];
    for (plant, make) in plants {
        if !make(&outside, &mark) {
            // Windows makes a symbolic link only in Developer Mode or as admin; only that case may be skipped.
            assert_eq!(
                plant, "a symbolic link",
                "only a symbolic link may be refused"
            );
            eprintln!("the system refused a symbolic link, so that case is skipped here");
            continue;
        }
        assert_eq!(bed.snips.mark_copied(&video), Ok(()), "{plant}");
        assert!(plain(&mark), "{plant}: the mark is a plain file");
        assert!(
            fs::read(&mark).unwrap().is_empty(),
            "{plant}: a new, empty mark"
        );
        assert_eq!(
            fs::read(&outside).unwrap(),
            OUTSIDE,
            "{plant}: what it reached stays"
        );
        fs::remove_file(&mark).unwrap();
    }
}

#[test]
fn prune_deletes_only_the_marks_whose_video_is_gone() {
    let bed = Bed::new();
    let video = SnipFile::new(bed.put("1.mp4"), FileKind::Mp4).unwrap();
    assert_eq!(bed.snips.mark_copied(&video), Ok(()));
    let orphan = bed.put("2.mp4.copied");
    let other = bed.put("notes.txt");
    assert_eq!(bed.snips.prune_marks(), Ok(()));
    assert!(!orphan.exists(), "its video is gone");
    assert!(bed.unsaved.join("1.mp4.copied").exists() && other.exists());
    fs::remove_dir_all(&bed.unsaved).unwrap();
    assert_eq!(bed.snips.prune_marks(), Err(SaveError::Missing));
}

#[test]
fn the_room_is_the_drives_and_a_gone_drive_is_missing() {
    let bed = Bed::new();
    assert_eq!(bed.snips.free(), Ok(FREE));
    let at = bed.snips.snips();
    let gone = DiskSnips::new(at, Arc::new(Drive(None)));
    assert_eq!(gone.free(), Err(SaveError::Missing));
}
