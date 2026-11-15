//! The real looks, holds and drive check behind the reveals. No test shows a plain folder: that would open a window
//! on the screen of whoever runs the tests.

use super::*;
use std::path::PathBuf;

/// A fresh folder of this test's own under the temporary folder, holding one file.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("kx-folder-reveal-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("model.gguf"), b"x").unwrap();
    dir
}

#[test]
fn a_missing_or_partial_folder_is_missing_and_opens_nothing() {
    let missing = std::env::temp_dir().join("kx-reveal-no-such-folder");
    assert_eq!(WinReveal.show(&missing), Err(RevealError::Missing));
    assert_eq!(WinReveal.show(Path::new(".")), Err(RevealError::Missing));
}

#[test]
fn a_real_folder_is_found_plain_and_a_file_or_nothing_is_other() {
    let dir = scratch("look");
    assert_eq!(Win32.found(&dir), Found::Folder);
    assert_eq!(Win32.found(&dir.join("model.gguf")), Found::Other);
    assert_eq!(Win32.found(&dir.join("none")), Found::Other);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn a_held_folder_and_the_folder_above_it_cannot_be_renamed_until_it_is_let_go() {
    let base = scratch("hold");
    let (dir, aside) = (base.join("above"), base.join("aside"));
    let (inner, moved) = (dir.join("models"), dir.join("moved"));
    std::fs::create_dir_all(&inner).unwrap();
    let held = Win32.hold(&inner).expect("a plain folder is held");
    assert!(std::fs::rename(&inner, &moved).is_err(), "the folder stays");
    assert!(std::fs::rename(&dir, &aside).is_err(), "the one above too");
    drop(held);
    assert!(std::fs::rename(&dir, &aside).is_ok(), "let go");
    let file = Win32.hold(&base.join("model.gguf")).err();
    assert_eq!(file, Some(Found::Other), "a file is never held");
    assert_eq!(
        Win32.hold(&inner).err(),
        Some(Found::Other),
        "nothing there"
    );
    std::fs::remove_dir_all(&base).unwrap();
}

#[test]
fn the_temporary_folder_is_on_a_local_drive_and_an_unused_letter_is_not() {
    assert!(Win32.local(&std::env::temp_dir()), "the tests' own drive");
    let used: Vec<u8> = fixed_drives()
        .iter()
        .filter_map(|(root, _)| root.as_os_str().as_encoded_bytes().first().copied())
        .collect();
    let free = (b'A'..=b'Z')
        .find(|l| !used.contains(l))
        .expect("a letter no fixed drive uses");
    let elsewhere = PathBuf::from(format!(r"{}:\models", char::from(free)));
    assert!(!Win32.local(&elsewhere), "{elsewhere:?}");
}

#[test]
fn the_platforms_folder_reveal_refuses_a_missing_folder_on_another_thread() {
    let reveal = crate::WindowsPlatform::new().unwrap().folder_reveal();
    let gone = FolderPath::new(std::env::temp_dir().join("kx-no-such-models")).unwrap();
    let shown = std::thread::spawn(move || reveal.show(&gone)).join();
    assert_eq!(shown.ok(), Some(Err(FolderError::Missing)));
}

#[test]
fn a_file_or_nothing_is_missing_and_opens_nothing() {
    let dir = scratch("file");
    let file = FolderPath::new(dir.join("model.gguf")).unwrap();
    assert_eq!(WinFolderReveal.show(&file), Err(FolderError::Missing));
    let none = FolderPath::new(dir.join("none")).unwrap();
    assert_eq!(WinFolderReveal.show(&none), Err(FolderError::Missing));
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn a_link_as_the_folder_or_above_it_is_linked_and_opens_nothing() {
    let dir = scratch("link");
    let (link, inner) = (dir.join("link"), dir.join("inner"));
    std::fs::create_dir(&inner).unwrap();
    // Making a link needs Developer Mode or an administrator; without it the rule stays covered by `plain`'s tests.
    if std::os::windows::fs::symlink_dir(&inner, &link).is_ok() {
        assert_eq!(Win32.found(&link), Found::Linked);
        assert_eq!(Win32.hold(&link).err(), Some(Found::Linked));
        let linked = FolderPath::new(link.clone()).unwrap();
        assert_eq!(WinFolderReveal.show(&linked), Err(FolderError::Linked));
        std::fs::create_dir(inner.join("models")).unwrap();
        let below = FolderPath::new(link.join("models")).unwrap();
        assert_eq!(WinFolderReveal.show(&below), Err(FolderError::Linked));
    }
    std::fs::remove_dir_all(&dir).unwrap();
}
