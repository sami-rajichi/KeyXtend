//! The file facts over fake filesystem calls: a share or a device path is left unchecked with no call at all, and a
//! local path is looked at once its root answers.

use std::cell::Cell;
use std::path::{Path, PathBuf};

use kx_module_api::ports::FileFact;

use super::facts_by;

/// The facts of `paths` with every root answering and every file 1 byte, and how many filesystem calls they made.
fn probed(paths: &[&str]) -> (Vec<FileFact>, usize) {
    let calls = Cell::new(0);
    let reach = |_: &Path| {
        calls.set(calls.get() + 1);
        true
    };
    let look = |_: &Path| {
        calls.set(calls.get() + 1);
        FileFact::Size(1)
    };
    let paths: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();
    (facts_by(&paths, reach, look), calls.get())
}

#[test]
fn a_share_or_a_device_path_is_unchecked_with_no_filesystem_call() {
    let far = [
        r"\\host\share\a.txt",
        r"\\?\C:\a.txt",
        r"\\.\PhysicalDrive0",
        "//host/share/b.txt",
    ];
    assert_eq!(probed(&far), (vec![FileFact::Unchecked; 4], 0));
}

#[test]
fn a_local_path_is_looked_at_once_its_root_answers() {
    let (found, calls) = probed(&["a.txt", "b.txt"]);
    assert_eq!(found, [FileFact::Size(1); 2]);
    assert_eq!(calls, 3, "one root asked once, then each file");
}
