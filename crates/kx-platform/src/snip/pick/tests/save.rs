//! A Save answer on a real temp folder: a picture written there, a video of the unsaved part copied there, and every
//! other write refused as the port says.

use super::*;
use crate::SNIP_DIR;
use crate::snip::pick::save::{PART_TRIES, part_of, put, put_by};
use kx_module_api::Redacted;
use kx_module_api::ports::SaveError;
use kx_test_support::tempdir::TempDir;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::AtomicU32;

/// A temp folder holding the Snips folder and its unsaved part, and the fake folder that names them.
struct Bed {
    dir: TempDir,
    snips: PathBuf,
    folder: Arc<dyn SnipFolder>,
}

impl Bed {
    fn new() -> Self {
        let dir = TempDir::new("save").unwrap();
        let snips = dir.path().join(SNIP_DIR);
        let unsaved = snips.join(UNSAVED_DIR);
        fs::create_dir_all(&unsaved).unwrap();
        let clock: Arc<dyn Clock> = Arc::new(Hands::default());
        let (at, part) = (FolderPath::new(snips.clone()), FolderPath::new(unsaved));
        let folder = FakeSnipFolder::new(clock, at.unwrap(), part.unwrap());
        Self {
            dir,
            snips,
            folder: Arc::new(folder),
        }
    }

    /// The dialog's answer `name` of `kind` in the temp folder.
    fn answer(&self, name: &str, kind: FileKind) -> DiskSaveTo {
        let file = SnipFile::new(self.dir.path().join(name), kind).unwrap();
        DiskSaveTo::new(file, Arc::clone(&self.folder))
    }

    /// A video `name` holding `bytes` in `folder`, under the Snips folder.
    fn video(&self, folder: &str, name: &str, bytes: &[u8]) -> SnipFile {
        let path = self.snips.join(folder).join(name);
        fs::write(&path, bytes).unwrap();
        SnipFile::new(path, FileKind::Mp4).unwrap()
    }
}

/// A picture's bytes, as the port hands them.
fn png(bytes: &[u8]) -> Redacted<Vec<u8>> {
    Redacted::new(bytes.to_vec())
}

#[test]
fn a_picture_is_written_where_the_dialog_said_replacing_what_was_there() {
    let bed = Bed::new();
    let to = bed.answer("a.png", FileKind::Png);
    fs::write(to.file().expose(), b"old").unwrap();
    assert_eq!(to.png(&png(b"new")), Ok(()));
    assert_eq!(fs::read(to.file().expose()).unwrap(), b"new");
    let video = bed.answer("a.mp4", FileKind::Mp4);
    assert_eq!(video.png(&png(b"new")), Err(SaveError::BadPath));
    let lost = bed.answer("gone/b.png", FileKind::Png);
    assert_eq!(lost.png(&png(b"new")), Err(SaveError::Missing));
}

/// The names in `dir`, sorted.
fn names(dir: &Path) -> Vec<String> {
    let entries = fs::read_dir(dir).unwrap();
    let mut names: Vec<String> = entries
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[test]
fn a_write_that_fails_partway_leaves_the_place_as_it_was_and_no_part() {
    let bed = Bed::new();
    let to = bed.dir.path().join("a.png");
    fs::write(&to, b"old").unwrap();
    let broken = put(&to, |f| {
        f.write_all(b"ne")?;
        Err(io::Error::from(io::ErrorKind::StorageFull))
    });
    assert_eq!(
        broken.map_err(|e| e.kind()),
        Err(io::ErrorKind::StorageFull)
    );
    assert_eq!(
        fs::read(&to).unwrap(),
        b"old",
        "the user's file stays whole"
    );
    assert_eq!(names(bed.dir.path()), ["a.png", SNIP_DIR], "no part left");
    let fresh = bed.dir.path().join("b.png");
    assert!(put(&fresh, |f| f.write_all(b"ne")).is_ok());
    assert_eq!(fs::read(&fresh).unwrap(), b"ne");
    assert_eq!(names(bed.dir.path()), ["a.png", "b.png", SNIP_DIR]);
}

#[test]
fn a_stale_part_is_left_untouched_and_the_save_still_succeeds() {
    let bed = Bed::new();
    let to = bed.dir.path().join("c.png");
    let stale = [part_of(&to, 0), part_of(&to, 1)];
    for part in &stale {
        fs::write(part, b"theirs").unwrap();
    }
    let parts = AtomicU32::new(0);
    assert!(put_by(&to, &parts, |f| f.write_all(b"ne")).is_ok());
    assert_eq!(fs::read(&to).unwrap(), b"ne");
    for part in &stale {
        assert_eq!(fs::read(part).unwrap(), b"theirs", "never written over");
    }
    assert_eq!(names(bed.dir.path()).len(), 4, "no new part left");
}

#[test]
fn a_save_whose_every_part_name_is_taken_fails_and_touches_none() {
    let bed = Bed::new();
    let to = bed.dir.path().join("d.png");
    for n in 0..PART_TRIES {
        fs::write(part_of(&to, n), b"theirs").unwrap();
    }
    let parts = AtomicU32::new(0);
    let refused = put_by(&to, &parts, |f| f.write_all(b"ne"));
    assert_eq!(
        refused.map_err(|e| e.kind()),
        Err(io::ErrorKind::AlreadyExists)
    );
    assert!(!to.exists());
    let left = usize::try_from(PART_TRIES).unwrap() + 1;
    assert_eq!(
        names(bed.dir.path()).len(),
        left,
        "only theirs and the Snips folder"
    );
}

#[test]
fn a_video_whose_source_is_gone_leaves_nothing_at_the_place() {
    let bed = Bed::new();
    let to = bed.answer("b.mp4", FileKind::Mp4);
    let gone = SnipFile::new(bed.snips.join(UNSAVED_DIR).join("3.mp4"), FileKind::Mp4);
    assert_eq!(to.copy(&gone.unwrap()), Err(SaveError::Missing));
    assert!(!to.file().expose().exists());
}

#[test]
fn a_video_of_the_unsaved_part_is_copied_there_and_any_other_refused() {
    let bed = Bed::new();
    let kept = bed.video(UNSAVED_DIR, "1.mp4", b"mp4");
    let to = bed.answer("b.mp4", FileKind::Mp4);
    assert_eq!(to.copy(&kept), Ok(()));
    assert_eq!(fs::read(to.file().expose()).unwrap(), b"mp4");
    let elsewhere = bed.video("", "2.mp4", b"mp4");
    assert_eq!(to.copy(&elsewhere), Err(SaveError::BadPath));
    let gone = SnipFile::new(bed.snips.join(UNSAVED_DIR).join("3.mp4"), FileKind::Mp4);
    assert_eq!(to.copy(&gone.unwrap()), Err(SaveError::Missing));
    let picture = bed.answer("c.png", FileKind::Png);
    assert_eq!(picture.copy(&kept), Err(SaveError::BadPath));
}

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

#[test]
fn a_link_or_folder_planted_in_the_unsaved_part_is_never_copied() {
    let bed = Bed::new();
    let unsaved = bed.snips.join(UNSAVED_DIR);
    let outside = bed.snips.join("mine.mp4");
    fs::write(&outside, b"outside").unwrap();
    fs::create_dir(unsaved.join("5.mp4")).unwrap();
    let planted = unsaved.join("4.mp4");
    let linked = symlink(&outside, &planted);
    if !linked {
        // Windows makes a symbolic link only in Developer Mode or as admin, so only the folder is tried here.
        eprintln!("the system refused a symbolic link, so that case is skipped here");
    }
    let names = ["5.mp4"].into_iter().chain(linked.then_some("4.mp4"));
    for name in names {
        let to = bed.answer(&format!("copy {name}"), FileKind::Mp4);
        let source = SnipFile::new(unsaved.join(name), FileKind::Mp4).unwrap();
        assert_eq!(to.copy(&source), Err(SaveError::BadPath), "{name}");
        assert!(!to.file().expose().exists(), "{name}");
    }
}
