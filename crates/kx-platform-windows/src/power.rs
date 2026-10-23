//! The power dialog: the system's own, which only the user answers; nothing here shuts down or confirms it.
//! The rule is pure over `Os`; only `win` calls Windows.
#![cfg_attr(
    not(windows),
    allow(
        dead_code,
        reason = "Only the Windows dialog runs this logic; tests cover it everywhere."
    )
)]

use kx_module_api::ports::PowerError;

#[cfg(windows)]
mod win;

#[cfg(windows)]
pub use win::WinPower;

/// The OS calls behind the dialog: the shell on Windows, a recorder in tests.
pub(crate) trait Os {
    /// The shell object that shows the dialog.
    type Shell;

    /// Starts COM on this thread and makes the shell object; `None` when either fails.
    fn shell(&self) -> Option<Self::Shell>;

    /// Asks `shell` to show the dialog; false when the OS refuses.
    fn show(&self, shell: &Self::Shell) -> bool;
}

/// Opens the dialog: `Unavailable` with no shell object, `Failed` when the OS refuses to show it.
pub(crate) fn open<O: Os>(os: &O) -> Result<(), PowerError> {
    let shell = os.shell().ok_or(PowerError::Unavailable)?;
    if os.show(&shell) {
        Ok(())
    } else {
        Err(PowerError::Failed)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    /// A recording shell: whether it can be made, whether it shows, and how often it was asked.
    #[derive(Default)]
    struct Fake {
        gone: bool,
        refuses: bool,
        asked: Cell<usize>,
    }

    impl Os for Fake {
        type Shell = ();

        fn shell(&self) -> Option<()> {
            (!self.gone).then_some(())
        }

        fn show(&self, (): &()) -> bool {
            self.asked.set(self.asked.get() + 1);
            !self.refuses
        }
    }

    #[test]
    fn the_dialog_is_asked_for_once_through_the_shell() {
        let fake = Fake::default();
        assert_eq!(open(&fake), Ok(()));
        assert_eq!(fake.asked.get(), 1);
    }

    #[test]
    fn with_no_shell_the_dialog_is_unavailable_and_nothing_is_asked() {
        let fake = Fake {
            gone: true,
            ..Fake::default()
        };
        assert_eq!(open(&fake), Err(PowerError::Unavailable));
        assert_eq!(fake.asked.get(), 0);
    }

    #[test]
    fn a_refused_show_is_a_failure() {
        let fake = Fake {
            refuses: true,
            ..Fake::default()
        };
        assert_eq!(open(&fake), Err(PowerError::Failed));
        assert_eq!(fake.asked.get(), 1);
    }
}
