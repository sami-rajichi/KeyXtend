//! Brings the Qt Quick tests' layout up to date from a fresh one laid out in a folder of its own: only changed files are
//! rewritten and strays go, so a `qmltestrunner` run reading the layout meanwhile never finds it gone.

use kx_test_support::tempdir::TempDir;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Ends the name a changed file is written under before it takes its own.
const PART: &str = ".part";

/// Makes `to` hold exactly the files of `from`, writing only those whose bytes differ, each under a part name first.
pub fn sync(from: &Path, to: &Path) {
    let fresh = files(from);
    for rel in &fresh {
        let (src, dst) = (from.join(rel), to.join(rel));
        let bytes = std::fs::read(&src).expect("a laid-out file reads");
        if std::fs::read(&dst).ok().as_deref() == Some(bytes.as_slice()) {
            continue;
        }
        std::fs::create_dir_all(dst.parent().expect("a folder")).expect("the folder is made");
        let mut part = dst.clone().into_os_string();
        part.push(PART);
        std::fs::write(&part, &bytes).expect("the file is written");
        std::fs::rename(&part, &dst).expect("the file takes its name");
    }
    for rel in files(to).difference(&fresh) {
        std::fs::remove_file(to.join(rel)).expect("a stray file goes");
    }
}

/// Every file under `dir`, by its path from `dir`; none when `dir` is missing.
fn files(dir: &Path) -> BTreeSet<PathBuf> {
    let (mut found, mut todo) = (BTreeSet::new(), vec![dir.to_path_buf()]);
    while let Some(d) = todo.pop() {
        for path in std::fs::read_dir(&d)
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
        {
            if path.is_dir() {
                todo.push(path);
            } else {
                found.insert(
                    path.strip_prefix(dir)
                        .expect("under its folder")
                        .to_path_buf(),
                );
            }
        }
    }
    found
}

#[test]
fn only_changed_files_are_rewritten_and_strays_go() {
    let (from, to) = (
        TempDir::new("sync-from").expect("the test folder"),
        TempDir::new("sync-to").expect("the test folder"),
    );
    let (from, to) = (from.path(), to.path());
    std::fs::create_dir_all(from.join("icons")).expect("the test folder");
    std::fs::write(from.join("icons").join("a.svg"), "new").expect("the test folder");
    std::fs::write(from.join("same.ttf"), "same").expect("the test folder");
    std::fs::create_dir_all(to.join("icons")).expect("the test folder");
    std::fs::write(to.join("icons").join("a.svg"), "old").expect("the test folder");
    std::fs::write(to.join("same.ttf"), "same").expect("the test folder");
    std::fs::write(to.join("stray.txt"), "x").expect("the test folder");
    let old = SystemTime::UNIX_EPOCH + Duration::from_secs(1);
    let same = std::fs::File::options()
        .write(true)
        .open(to.join("same.ttf"))
        .expect("the test folder");
    same.set_modified(old).expect("the test folder");
    drop(same);
    sync(from, to);
    assert_eq!(
        std::fs::read_to_string(to.join("icons").join("a.svg")).expect("the test folder"),
        "new"
    );
    let kept = std::fs::metadata(to.join("same.ttf"))
        .expect("the test folder")
        .modified()
        .expect("the test folder");
    assert_eq!(kept, old, "an unchanged file is never rewritten");
    assert_eq!(files(to), files(from), "strays and part files are gone");
}
