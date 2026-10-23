//! The Win32 side of the power dialog: `Shell.Application`'s `ShutdownWindows`, under an apartment-threaded COM.

use kx_module_api::ports::{PowerDialog, PowerError};
use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance};
use windows::Win32::UI::Shell::{IShellDispatch, Shell as SHELL_APPLICATION};

use super::{Os, open};
use crate::com::Com;

/// The power dialog on Windows: the shell's own Shut Down Windows dialog, which only the user answers.
#[derive(Clone, Copy, Debug, Default)]
pub struct WinPower;

impl PowerDialog for WinPower {
    fn open(&self) -> Result<(), PowerError> {
        open(&Win32)
    }
}

/// The shell object, with COM kept on until it is released.
struct ShellApp {
    /// Released before COM stops, as fields drop in order.
    shell: IShellDispatch,
    _com: Com,
}

/// The Win32 calls behind `WinPower`.
struct Win32;

impl Os for Win32 {
    type Shell = ShellApp;

    #[allow(unsafe_code, reason = "Makes the shell's automation object.")]
    fn shell(&self) -> Option<ShellApp> {
        let com = Com::start_apartment().ok()?;
        // SAFETY: COM runs on this thread while the object lives, as `ShellApp` releases it before COM stops.
        let shell = unsafe { CoCreateInstance(&SHELL_APPLICATION, None, CLSCTX_INPROC_SERVER) };
        Some(ShellApp {
            shell: shell.ok()?,
            _com: com,
        })
    }

    #[allow(unsafe_code, reason = "Asks the shell to show its dialog.")]
    fn show(&self, app: &ShellApp) -> bool {
        // SAFETY: a call with no arguments on a live object made on this thread.
        unsafe { app.shell.ShutdownWindows() }.is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shell_object_is_made_on_a_thread_of_its_own_and_nothing_is_shown() {
        // Never `show`: that would open the dialog on the owner's screen.
        let made = std::thread::spawn(|| Win32.shell().is_some()).join();
        assert_eq!(made.ok(), Some(true));
    }
}
