//! The program and per-user folders; the per-user one comes from Windows' known-folder list, whatever TEMP a shell has set.

use std::path::Path;

use kx_platform::{APP_DIR_NAME, AppDirs};

use crate::PlatformError;

#[cfg(windows)]
pub use os::{current, local_data};

/// The folders for a program at `exe`: its own folder, and the app folder inside the per-user folder `local`.
///
/// # Errors
/// `PlatformError::NoExeDir` when `exe` names no folder.
pub fn from_paths(exe: &Path, local: &Path) -> Result<AppDirs, PlatformError> {
    let exe_dir = exe
        .parent()
        .filter(|dir| !dir.as_os_str().is_empty())
        .ok_or(PlatformError::NoExeDir)?;
    Ok(AppDirs {
        exe_dir: exe_dir.to_path_buf(),
        user_dir: local.join(APP_DIR_NAME),
    })
}

#[cfg(windows)]
mod os {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use std::path::PathBuf;

    use kx_platform::AppDirs;
    use windows::Win32::System::Com::CoTaskMemFree;
    use windows::Win32::UI::Shell::{FOLDERID_LocalAppData, KF_FLAG_DEFAULT, SHGetKnownFolderPath};

    use super::from_paths;
    use crate::PlatformError;

    /// The user's local app-data folder, asked of Windows rather than read from the environment.
    ///
    /// # Errors
    /// `PlatformError::LocalData` when Windows does not name it.
    #[allow(unsafe_code, reason = "SHGetKnownFolderPath, then frees its string.")]
    pub fn local_data() -> Result<PathBuf, PlatformError> {
        // SAFETY: a plain query with a known-folder id; Windows allocates the string it returns.
        let p = unsafe { SHGetKnownFolderPath(&FOLDERID_LocalAppData, KF_FLAG_DEFAULT, None) }
            .map_err(|e| PlatformError::LocalData(e.into()))?;
        // SAFETY: `p` is the NUL-terminated string Windows just returned, still unfreed.
        let path = OsString::from_wide(unsafe { p.as_wide() });
        // SAFETY: frees the string Windows allocated, once; `path` is a copy and `p` is not used again.
        unsafe { CoTaskMemFree(Some(p.0.cast_const().cast())) };
        Ok(PathBuf::from(path))
    }

    /// The folders of the running program.
    ///
    /// # Errors
    /// `PlatformError` when Windows names neither the program nor the per-user folder.
    pub fn current() -> Result<AppDirs, PlatformError> {
        let exe = std::env::current_exe().map_err(PlatformError::ExePath)?;
        from_paths(&exe, &local_data()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// A program path and a per-user folder, built with this system's separator.
    fn paths() -> (PathBuf, PathBuf) {
        let exe = Path::new("root").join("app").join("keyxtend.exe");
        (exe, Path::new("root").join("local"))
    }

    #[test]
    fn the_exe_dir_holds_the_program_and_the_user_dir_is_the_app_folder() {
        let (exe, local) = paths();
        let dirs = from_paths(&exe, &local).unwrap();
        assert_eq!(dirs.exe_dir, Path::new("root").join("app"));
        assert_eq!(dirs.user_dir, local.join(APP_DIR_NAME));
    }

    #[test]
    fn a_program_path_without_a_folder_is_refused() {
        let (_, local) = paths();
        for exe in [Path::new(""), Path::new("keyxtend.exe")] {
            let got = from_paths(exe, &local);
            assert!(matches!(got, Err(PlatformError::NoExeDir)), "{got:?}");
        }
    }

    #[cfg(windows)]
    #[test]
    fn local_data_is_an_existing_absolute_folder() {
        let dir = local_data().expect("Windows knows the folder");
        assert!(dir.is_absolute() && dir.is_dir(), "{}", dir.display());
    }

    #[cfg(windows)]
    #[test]
    fn the_current_dirs_use_the_known_folder_and_this_program() {
        let dirs = current().unwrap();
        assert_eq!(dirs.user_dir, local_data().unwrap().join(APP_DIR_NAME));
        let exe = std::env::current_exe().unwrap();
        assert_eq!(Some(dirs.exe_dir.as_path()), exe.parent());
    }
}
