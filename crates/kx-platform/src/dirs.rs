//! Where the program keeps its data: beside the program (portable) or in the per-user folder.

use std::path::{Path, PathBuf};

/// Name of the folder that, when it sits beside the program, makes the edition portable.
pub const PORTABLE_DIR: &str = "data";

/// The folders an OS adapter reports.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppDirs {
    /// The folder of the running program.
    pub exe_dir: PathBuf,
    /// The OS per-user data folder of the program.
    pub user_dir: PathBuf,
}

/// The chosen data folder and the edition it belongs to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DataDir {
    /// The portable edition: the `data` folder beside the program.
    Portable(PathBuf),
    /// The installed edition: the per-user folder.
    Installed(PathBuf),
}

impl DataDir {
    /// The folder's path.
    #[must_use]
    pub fn path(&self) -> &Path {
        match self {
            Self::Portable(path) | Self::Installed(path) => path,
        }
    }
}

/// Picks the data folder: portable when `is_dir` says `exe_dir` holds a `data` folder, else installed.
#[must_use]
pub fn pick_data_dir(exe_dir: &Path, user_dir: &Path, is_dir: impl Fn(&Path) -> bool) -> DataDir {
    let portable = exe_dir.join(PORTABLE_DIR);
    if is_dir(&portable) {
        DataDir::Portable(portable)
    } else {
        DataDir::Installed(user_dir.to_path_buf())
    }
}

impl AppDirs {
    /// The data folder, checked on the real file system.
    #[must_use]
    pub fn data_dir(&self) -> DataDir {
        pick_data_dir(&self.exe_dir, &self.user_dir, Path::is_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kx_test_support::tempdir::TempDir;
    use std::cell::RefCell;

    fn dirs() -> (PathBuf, PathBuf) {
        (PathBuf::from("app"), PathBuf::from("user"))
    }

    #[test]
    fn a_data_folder_beside_the_exe_is_portable() {
        let (exe, user) = dirs();
        let got = pick_data_dir(&exe, &user, |_| true);
        assert_eq!(got, DataDir::Portable(exe.join(PORTABLE_DIR)));
    }

    #[test]
    fn no_data_folder_is_installed() {
        let (exe, user) = dirs();
        let got = pick_data_dir(&exe, &user, |_| false);
        assert_eq!(got, DataDir::Installed(user));
    }

    #[test]
    fn only_the_data_path_beside_the_exe_is_checked() {
        let (exe, user) = dirs();
        let asked = RefCell::new(Vec::new());
        let _ = pick_data_dir(&exe, &user, |p| {
            asked.borrow_mut().push(p.to_path_buf());
            false
        });
        assert_eq!(asked.into_inner(), vec![exe.join(PORTABLE_DIR)]);
    }

    #[test]
    fn the_path_is_the_folder_of_either_edition() {
        let (portable, installed) = (PathBuf::from("p"), PathBuf::from("i"));
        assert_eq!(DataDir::Portable(portable.clone()).path(), portable);
        assert_eq!(DataDir::Installed(installed.clone()).path(), installed);
    }

    fn real(exe: &TempDir, user: &TempDir) -> DataDir {
        let dirs = AppDirs {
            exe_dir: exe.path().to_path_buf(),
            user_dir: user.path().to_path_buf(),
        };
        dirs.data_dir()
    }

    #[test]
    fn a_real_data_folder_is_portable() {
        let (exe, user) = (TempDir::new("exe").unwrap(), TempDir::new("user").unwrap());
        std::fs::create_dir(exe.path().join(PORTABLE_DIR)).unwrap();
        let want = DataDir::Portable(exe.path().join(PORTABLE_DIR));
        assert_eq!(real(&exe, &user), want);
    }

    #[test]
    fn a_real_file_named_data_is_installed() {
        let (exe, user) = (TempDir::new("exe").unwrap(), TempDir::new("user").unwrap());
        std::fs::write(exe.path().join(PORTABLE_DIR), "x").unwrap();
        let want = DataDir::Installed(user.path().to_path_buf());
        assert_eq!(real(&exe, &user), want);
    }

    #[test]
    fn a_real_empty_folder_is_installed() {
        let (exe, user) = (TempDir::new("exe").unwrap(), TempDir::new("user").unwrap());
        let want = DataDir::Installed(user.path().to_path_buf());
        assert_eq!(real(&exe, &user), want);
    }
}
