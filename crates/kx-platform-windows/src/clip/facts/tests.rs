//! The file facts over fake filesystem calls: a path off a drive is left unchecked with no call at all, a drive path
//! is looked at once its root answers, and a link on the way or at the end, a real junction too, is never followed.

use std::cell::RefCell;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use kx_module_api::ports::FileFact;
use kx_test_support::tempdir::TempDir;

use super::{Found, facts_by, seen, told};

/// What the fake filesystem says of each path: a link at each of `links`, else a 1-byte file.
fn answer(links: &[&str], path: &Path) -> Found {
    let link = links.iter().any(|l| Path::new(l) == path);
    if link { Found::Link } else { Found::File(1) }
}

/// The facts of `paths` with every root answering, and each path the fake filesystem was asked, roots first.
fn probed(paths: &[&str], links: &[&str]) -> (Vec<FileFact>, Vec<PathBuf>) {
    let asked = RefCell::new(Vec::new());
    let reach = |root: &Path| {
        asked.borrow_mut().push(root.to_path_buf());
        true
    };
    let look = |path: &Path| {
        asked.borrow_mut().push(path.to_path_buf());
        Ok(answer(links, path))
    };
    let paths: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();
    let found = facts_by(&paths, reach, look);
    (found, asked.into_inner())
}

#[test]
fn a_path_off_a_drive_is_unchecked_with_no_filesystem_call() {
    let far = [
        r"\\host\share\a.txt",
        r"\\?\C:\a.txt",
        r"\\?\UNC\host\share\a.txt",
        r"\\.\PhysicalDrive0",
        r"\??\UNC\host\share\a.txt",
        "//host/share/b.txt",
        "a.txt",
        "C:a.txt",
    ];
    let (found, asked) = probed(&far, &[]);
    assert_eq!(found, [FileFact::Unchecked; 8]);
    assert!(asked.is_empty(), "{asked:?}");
}

#[test]
fn a_drive_path_is_looked_at_once_its_root_answers_and_each_path_once() {
    let (found, asked) = probed(&["C:/d/a.txt", "C:/d/b.txt"], &[]);
    assert_eq!(found, [FileFact::Size(1); 2]);
    for (n, path) in asked.iter().enumerate() {
        assert!(!asked[..n].contains(path), "{path:?} asked twice");
    }
    assert!(
        asked.iter().any(|p| p == Path::new("C:/d")),
        "its folder is looked at"
    );
}

#[test]
fn a_link_on_the_way_or_at_the_end_is_unchecked_and_never_followed() {
    let paths = ["C:/link/a.txt", "C:/d/linked.txt", "C:/d/b.txt"];
    let (found, asked) = probed(&paths, &["C:/link", "C:/d/linked.txt"]);
    let want = [FileFact::Unchecked, FileFact::Unchecked, FileFact::Size(1)];
    assert_eq!(found, want);
    assert!(
        !asked.iter().any(|p| p == Path::new(paths[0])),
        "never through a link"
    );
}

#[test]
fn a_link_reads_unchecked_a_file_its_size_and_a_missing_path_gone() {
    assert_eq!(told(Ok(Found::Link)), FileFact::Unchecked);
    assert_eq!(told(Ok(Found::File(7))), FileFact::Size(7));
    assert_eq!(told(Ok(Found::Other)), FileFact::Size(0));
    assert_eq!(told(Err(ErrorKind::NotFound.into())), FileFact::Gone);
    assert_eq!(
        told(Err(ErrorKind::PermissionDenied.into())),
        FileFact::Size(0)
    );
}

#[test]
fn a_real_file_is_seen_as_a_file_of_its_size() {
    let dir = TempDir::new("clip-facts").unwrap();
    let file = dir.path().join("a.txt");
    std::fs::write(&file, b"abc").unwrap();
    assert_eq!(seen(&file).unwrap(), Found::File(3));
    assert_eq!(seen(dir.path()).unwrap(), Found::Other);
}

#[cfg(unix)]
#[test]
fn a_real_symlink_is_seen_as_a_link_and_not_followed() {
    let dir = TempDir::new("clip-link").unwrap();
    let file = dir.path().join("a.txt");
    std::fs::write(&file, b"abc").unwrap();
    let link = dir.path().join("link.txt");
    std::os::unix::fs::symlink(&file, &link).unwrap();
    assert_eq!(seen(&link).unwrap(), Found::Link);
}

#[cfg(windows)]
#[test]
fn a_real_junction_on_the_way_is_unchecked_and_never_followed() {
    use super::facts;
    use kx_module_api::Redacted;
    let dir = TempDir::new("clip-junction").unwrap();
    let real = dir.path().join("real");
    std::fs::create_dir(&real).unwrap();
    std::fs::write(real.join("a.txt"), b"abc").unwrap();
    let link = dir.path().join("link");
    let made = std::process::Command::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(&link)
        .arg(&real)
        .output()
        .unwrap();
    assert!(made.status.success(), "mklink /J failed: {made:?}");
    assert_eq!(seen(&link).unwrap(), Found::Link, "the junction itself");
    let paths = Redacted::new(vec![link.join("a.txt"), real.join("a.txt")]);
    let want = [FileFact::Unchecked, FileFact::Size(3)];
    assert_eq!(facts(&paths), want, "through it, then the real file");
}
