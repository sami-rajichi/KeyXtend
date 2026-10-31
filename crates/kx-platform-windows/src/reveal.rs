//! The reveal port: the running Explorer opens the folder itself, so no program starts as our child and Explorer keeps
//! its own rights, never our uiAccess (ADR-0034). The rule is pure over `Os`; only `win` calls Windows.
#![cfg_attr(
    not(windows),
    allow(
        dead_code,
        reason = "Only the Windows reveal runs this logic; tests cover it everywhere."
    )
)]

use kx_platform::ui::RevealError;
use std::path::Path;

#[cfg(windows)]
mod win;

#[cfg(windows)]
pub use win::WinReveal;

/// The OS calls behind the reveal: the running shell on Windows, a recorder in tests.
pub(crate) trait Os {
    /// The running file manager's shell object.
    type Shell;

    /// True when `folder` is a whole path to a folder that exists.
    fn is_folder(&self, folder: &Path) -> bool;

    /// The running file manager's shell object, or the code that refused it.
    fn shell(&self) -> Result<Self::Shell, i32>;

    /// Asks `shell` to open `folder`, or gives the code that refused it.
    fn open(&self, shell: &Self::Shell, folder: &Path) -> Result<(), i32>;
}

/// Shows `folder`: `Missing` without asking the shell when it is no folder, `Refused` when the shell says no.
pub(crate) fn show<O: Os>(os: &O, folder: &Path) -> Result<(), RevealError> {
    if !os.is_folder(folder) {
        return Err(RevealError::Missing);
    }
    let shell = os.shell().map_err(RevealError::Refused)?;
    os.open(&shell, folder).map_err(RevealError::Refused)
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::path::PathBuf;

    use super::*;

    /// A code the fake shell refuses with.
    const CODE: i32 = -5;

    /// A recording shell: whether the folder exists, whether the shell is there and opens, and what it opened.
    #[derive(Default)]
    struct Fake {
        missing: bool,
        gone: bool,
        refuses: bool,
        opened: RefCell<Vec<PathBuf>>,
    }

    impl Os for Fake {
        type Shell = ();

        fn is_folder(&self, _: &Path) -> bool {
            !self.missing
        }

        fn shell(&self) -> Result<(), i32> {
            if self.gone { Err(CODE) } else { Ok(()) }
        }

        fn open(&self, (): &(), folder: &Path) -> Result<(), i32> {
            self.opened.borrow_mut().push(folder.to_path_buf());
            if self.refuses { Err(CODE) } else { Ok(()) }
        }
    }

    #[test]
    fn the_shell_opens_the_folder_once() {
        let fake = Fake::default();
        assert_eq!(show(&fake, Path::new("history")), Ok(()));
        assert_eq!(*fake.opened.borrow(), [PathBuf::from("history")]);
    }

    #[test]
    fn a_missing_folder_never_reaches_the_shell() {
        let fake = Fake {
            missing: true,
            ..Fake::default()
        };
        assert_eq!(show(&fake, Path::new("gone")), Err(RevealError::Missing));
        assert!(fake.opened.borrow().is_empty());
    }

    #[test]
    fn no_shell_or_a_refused_open_is_refused_with_its_code() {
        let gone = Fake {
            gone: true,
            ..Fake::default()
        };
        assert_eq!(show(&gone, Path::new("a")), Err(RevealError::Refused(CODE)));
        assert!(gone.opened.borrow().is_empty());
        let refuses = Fake {
            refuses: true,
            ..Fake::default()
        };
        assert_eq!(
            show(&refuses, Path::new("a")),
            Err(RevealError::Refused(CODE))
        );
    }
}
