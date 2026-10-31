//! The shelf's drive on Windows: the bytes free to us and the volume's root, shown without its last backslash.
//! The name rule is pure and tested everywhere; only `WinDiskRoom` calls Windows.
#![cfg_attr(
    not(windows),
    allow(
        dead_code,
        reason = "Only the Windows disk room runs this rule; tests cover it everywhere."
    )
)]

#[cfg(windows)]
pub use win::WinDiskRoom;

/// What ends a volume's root path.
const SEP: char = '\\';

/// The name to show for the volume whose root is `root`: `D:\` shows as `D:`, a bare `\` stays.
fn shown(root: &str) -> String {
    let trimmed = root.strip_suffix(SEP).filter(|t| !t.is_empty());
    trimmed.unwrap_or(root).to_owned()
}

#[cfg(windows)]
mod win {
    use std::io;
    use std::os::windows::ffi::OsStrExt;
    use std::path::Path;

    use kx_crypto::DiskRoom;
    use windows::Win32::Foundation::MAX_PATH;
    use windows::Win32::Storage::FileSystem::{GetDiskFreeSpaceExW, GetVolumePathNameW};
    use windows::core::PCWSTR;

    use super::shown;

    /// The Windows disk room: free bytes as the user's quota allows, and the volume that holds a folder.
    #[derive(Clone, Copy, Debug, Default)]
    pub struct WinDiskRoom;

    impl DiskRoom for WinDiskRoom {
        #[allow(unsafe_code, reason = "Asks Windows for a folder's free bytes.")]
        fn free(&self, path: &Path) -> io::Result<u64> {
            let name = wide(path);
            let mut free = 0;
            // SAFETY: `name` is NUL-ended and outlives the call; `free` is a local Windows writes once.
            unsafe { GetDiskFreeSpaceExW(PCWSTR(name.as_ptr()), Some(&raw mut free), None, None) }?;
            Ok(free)
        }

        #[allow(
            unsafe_code,
            reason = "Asks Windows for the volume that holds a folder."
        )]
        fn drive(&self, path: &Path) -> io::Result<String> {
            let name = wide(path);
            // Room for the path with a last backslash and NUL, and never less than a classic path.
            let mut root = vec![0u16; name.len().max(MAX_PATH as usize) + 1];
            // SAFETY: `name` is NUL-ended and outlives the call; Windows writes at most `root.len()` units.
            unsafe { GetVolumePathNameW(PCWSTR(name.as_ptr()), &mut root) }?;
            let end = root.iter().position(|u| *u == 0).unwrap_or(root.len());
            Ok(shown(&String::from_utf16_lossy(&root[..end])))
        }
    }

    /// `path` as UTF-16 with an ending NUL, exactly as Windows names it.
    fn wide(path: &Path) -> Vec<u16> {
        path.as_os_str().encode_wide().chain([0]).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_drive_root_shows_as_its_letter_and_colon() {
        assert_eq!(shown("D:\\"), "D:");
    }

    #[test]
    fn a_mounted_folder_or_a_share_shows_without_its_last_backslash() {
        assert_eq!(shown("C:\\mnt\\disk\\"), "C:\\mnt\\disk");
        assert_eq!(shown("\\\\server\\share\\"), "\\\\server\\share");
    }

    #[test]
    fn a_root_without_a_backslash_shows_as_it_is() {
        assert_eq!(shown("D:"), "D:");
        assert_eq!(shown(&SEP.to_string()), "\\");
    }

    #[cfg(windows)]
    mod windows {
        use super::super::WinDiskRoom;
        use kx_crypto::DiskRoom;
        use kx_test_support::tempdir::TempDir;

        #[test]
        fn a_folder_has_free_bytes_on_its_drive() {
            let dir = TempDir::new("disk-free").unwrap();
            assert!(WinDiskRoom.free(dir.path()).unwrap() > 0);
        }

        #[test]
        fn a_folder_names_the_drive_it_starts_with() {
            let dir = TempDir::new("disk-name").unwrap();
            let drive = WinDiskRoom.drive(dir.path()).unwrap();
            let path = dir.path().to_string_lossy().to_lowercase();
            assert!(!drive.is_empty() && !drive.ends_with('\\'), "{drive}");
            assert!(path.starts_with(&drive.to_lowercase()), "{drive}");
        }

        #[test]
        fn a_missing_folder_reads_as_an_error() {
            let dir = TempDir::new("disk-gone").unwrap();
            let gone = dir.path().join("gone");
            assert!(WinDiskRoom.free(&gone).is_err());
        }
    }
}
