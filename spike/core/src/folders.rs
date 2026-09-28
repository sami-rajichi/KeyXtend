//! Per-user folders from Windows' known-folder list, the same whatever TEMP a shell has set.

use std::path::PathBuf;

use windows::Win32::System::Com::CoTaskMemFree;
use windows::Win32::UI::Shell::{FOLDERID_LocalAppData, KF_FLAG_DEFAULT, SHGetKnownFolderPath};

/// The user's local app-data folder, asked of Windows rather than read from the environment.
pub fn local_data() -> Result<PathBuf, String> {
    // SAFETY: Windows allocates the returned string; it is copied once, then freed once with CoTaskMemFree.
    unsafe {
        let p = SHGetKnownFolderPath(&FOLDERID_LocalAppData, KF_FLAG_DEFAULT, None)
            .map_err(|e| format!("SHGetKnownFolderPath: {e}"))?;
        let text = p.to_string();
        CoTaskMemFree(Some(p.0 as *const _));
        text.map(PathBuf::from).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn local_data_is_an_existing_absolute_folder() {
        let dir = super::local_data().expect("Windows knows the folder");
        assert!(dir.is_absolute() && dir.is_dir(), "{}", dir.display());
    }
}
