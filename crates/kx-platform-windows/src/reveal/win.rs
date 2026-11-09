//! The Win32 side of the reveal: Explorer's desktop shell object opens the folder there, through `crate::explorer`.

use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use kx_platform::ui::{Reveal, RevealError};
use windows::core::BSTR;

use super::{Os, show};
use crate::explorer::Explorer;

/// Shows a folder in Explorer: the running Explorer opens it itself (Raymond Chen's "launch from the shell" route), so
/// the window belongs to Explorer and nothing starts with our token.
#[derive(Clone, Copy, Debug, Default)]
pub struct WinReveal;

impl Reveal for WinReveal {
    fn show(&self, folder: &Path) -> Result<(), RevealError> {
        show(&Win32, folder)
    }
}

/// The Win32 calls behind `WinReveal`.
struct Win32;

impl Os for Win32 {
    type Shell = Explorer;

    fn is_folder(&self, folder: &Path) -> bool {
        folder.is_absolute() && folder.is_dir()
    }

    fn shell(&self) -> Result<Explorer, i32> {
        Explorer::find()
    }

    fn open(&self, explorer: &Explorer, folder: &Path) -> Result<(), i32> {
        let wide: Vec<u16> = folder.as_os_str().encode_wide().collect();
        explorer.open(&BSTR::from_wide(&wide))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_or_partial_folder_is_missing_and_opens_nothing() {
        // A real folder is never shown: that would open a window on the owner's screen.
        let missing = std::env::temp_dir().join("kx-reveal-no-such-folder");
        assert_eq!(WinReveal.show(&missing), Err(RevealError::Missing));
        assert_eq!(WinReveal.show(Path::new(".")), Err(RevealError::Missing));
    }
}
