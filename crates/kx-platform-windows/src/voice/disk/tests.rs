//! The Windows disk on this machine: the system drive is listed with room, a file's SHA-256 matches its known value
//! and never goes through a link, a hidden file is hidden, and a guarded folder holds only the user's and the
//! system's protected rights while one already there keeps its own.

use super::*;
use std::fs::OpenOptions;
use std::os::windows::fs::OpenOptionsExt;
use windows::Win32::Foundation::{HLOCAL, LocalFree};
use windows::Win32::Security::Authorization::{
    ConvertSecurityDescriptorToStringSecurityDescriptorW, GetNamedSecurityInfoW, SDDL_REVISION_1,
    SE_FILE_OBJECT,
};
use windows::Win32::Security::{DACL_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR};
use windows::Win32::Storage::FileSystem::FILE_ATTRIBUTE_HIDDEN;
use windows::core::{PCWSTR, PWSTR};

/// SHA-256 of `abc`, from FIPS 180-2's first example.
const ABC: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

/// A fresh folder for test `name` under the system's temporary folder, gone when dropped.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("kx-disk-{name}-{}", std::process::id()));
        drop(fs::remove_dir_all(&dir));
        fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        drop(fs::remove_dir_all(&self.0));
    }
}

#[test]
fn the_system_drive_is_listed_with_room() {
    let system = std::env::var("SystemDrive").unwrap();
    let drives = WinDisk.drives();
    let listed = drives.iter().find(|d| d.root.expose().starts_with(&system));
    assert!(listed.is_some_and(|d| d.free > 0), "{system} with room");
}

#[test]
fn a_files_sha256_matches_its_known_value_and_a_missing_file_is_not_found() {
    let dir = Scratch::new("print");
    let file = dir.0.join("abc.bin");
    fs::write(&file, b"abc").unwrap();
    assert_eq!(Fingerprint::new(WinDisk.sha256(&file).unwrap()).hex(), ABC);
    let gone = WinDisk.sha256(&dir.0.join("gone.bin")).unwrap_err();
    assert_eq!(gone.kind(), io::ErrorKind::NotFound);
    let writer = OpenOptions::new().append(true).open(&file).unwrap();
    assert!(
        WinDisk.sha256(&file).is_err(),
        "never while another writes it"
    );
    drop(writer);
}

#[test]
fn a_hidden_file_is_hidden_and_a_plain_one_is_not() {
    let dir = Scratch::new("hidden");
    let (hidden, plain) = (dir.0.join("h.gguf"), dir.0.join("p.gguf"));
    let mut open = OpenOptions::new();
    open.write(true)
        .create_new(true)
        .attributes(FILE_ATTRIBUTE_HIDDEN.0);
    drop(open.open(&hidden).unwrap());
    fs::write(&plain, b"").unwrap();
    assert!(WinDisk.hidden(&hidden));
    assert!(!WinDisk.hidden(&plain));
}

#[test]
fn a_guarded_folder_holds_only_the_users_and_the_systems_protected_rights() {
    let dir = Scratch::new("guard");
    let made = dir.0.join("models");
    WinDisk.make_guarded(&made).unwrap();
    let rights = rights(&made);
    assert!(rights.starts_with("D:P"), "protected: {rights}");
    assert_eq!(rights.matches("(A;").count(), 2, "{rights}");
    assert!(rights.contains(";;;SY)"), "{rights}");
}

#[test]
fn a_folder_already_there_keeps_its_rights_and_is_told() {
    let dir = Scratch::new("there");
    let before = rights(&dir.0);
    let again = WinDisk.make_guarded(&dir.0).unwrap_err();
    assert_eq!(again.kind(), io::ErrorKind::AlreadyExists);
    assert_eq!(rights(&dir.0), before, "never changed");
    let lost = WinDisk.make_guarded(&dir.0.join("gone").join("models"));
    assert_eq!(lost.unwrap_err().kind(), io::ErrorKind::NotFound);
}

#[test]
fn a_fingerprint_never_follows_a_link_or_reads_a_folder() {
    let dir = Scratch::new("link");
    let (target, link) = (dir.0.join("target"), dir.0.join("link"));
    fs::create_dir_all(&target).unwrap();
    let shell = PathBuf::from(std::env::var_os("SystemRoot").unwrap()).join(r"System32\cmd.exe");
    let made = std::process::Command::new(shell)
        .args(["/d", "/c", "mklink", "/J"])
        .arg(&link)
        .arg(&target)
        .output()
        .unwrap();
    assert!(made.status.success(), "a junction to target");
    for path in [&link, &target] {
        assert!(WinDisk.sha256(path).is_err(), "no hash of a folder or link");
    }
}

/// The folder's list of rights in Windows' own notation.
#[allow(unsafe_code, reason = "Reads a test folder's rights, then frees them.")]
fn rights(folder: &Path) -> String {
    use std::os::windows::ffi::OsStrExt;
    let name: Vec<u16> = folder.as_os_str().encode_wide().chain([0]).collect();
    let mut sd = PSECURITY_DESCRIPTOR::default();
    let what = DACL_SECURITY_INFORMATION;
    // SAFETY: `name` is NUL-ended; Windows allocates the descriptor, freed below.
    let read = unsafe {
        GetNamedSecurityInfoW(
            PCWSTR(name.as_ptr()),
            SE_FILE_OBJECT,
            what,
            None,
            None,
            None,
            None,
            &raw mut sd,
        )
    };
    assert_eq!(read.0, 0);
    let mut text = PWSTR::null();
    // SAFETY: a live descriptor; Windows allocates the text, freed below.
    unsafe {
        ConvertSecurityDescriptorToStringSecurityDescriptorW(
            sd,
            SDDL_REVISION_1,
            what,
            &raw mut text,
            None,
        )
    }
    .unwrap();
    // SAFETY: a NUL-ended string Windows made; both blocks are freed once, after the read.
    unsafe {
        let rights = text.to_string().unwrap();
        let _ = LocalFree(Some(HLOCAL(text.0.cast())));
        let _ = LocalFree(Some(HLOCAL(sd.0)));
        rights
    }
}
