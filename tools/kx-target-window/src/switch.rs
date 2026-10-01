//! Switches the input layout for a `--layout` run and puts the previous one back at close.
//!
//! On Windows 8+ `ActivateKeyboardLayout` switches the whole session's input language, not only this window.

use std::sync::{Mutex, PoisonError};

use kx_module_api::ports::{LayoutId, LayoutSource};
use kx_platform_windows::{WinLayouts, layouts};
use kx_target_window::layout::{self, LayoutError, Restore, Switcher};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    ACTIVATE_KEYBOARD_LAYOUT_FLAGS, ActivateKeyboardLayout, GetKeyboardLayout,
};

/// `ActivateKeyboardLayout` flags: none; Windows 8+ still switches the whole session.
const NO_FLAGS: ACTIVATE_KEYBOARD_LAYOUT_FLAGS = ACTIVATE_KEYBOARD_LAYOUT_FLAGS(0);
/// `GetKeyboardLayout` thread id that means this thread.
const OWN_THREAD: u32 = 0;

/// The layout to put back at close; empty without `--layout` or once it is back.
static BACK: Mutex<Option<Restore>> = Mutex::new(None);

/// The real switch, through Win32.
struct Win32;

impl Switcher for Win32 {
    #[allow(unsafe_code, reason = "Activates an installed layout.")]
    fn activate(&self, id: LayoutId) -> Option<LayoutId> {
        let handle = layouts::hkl(id)?;
        // SAFETY: no pointer is passed; on Windows 8+ this switches the session's input language.
        let previous = unsafe { ActivateKeyboardLayout(handle, NO_FLAGS) };
        previous.ok().map(layouts::id)
    }

    #[allow(unsafe_code, reason = "Reads this thread's layout.")]
    fn current(&self) -> LayoutId {
        // SAFETY: a plain query of this thread's layout.
        layouts::id(unsafe { GetKeyboardLayout(OWN_THREAD) })
    }
}

/// Refuses layout `id` unless Windows lists it as installed; nothing is ever loaded or added.
///
/// # Errors
/// `NotInstalled` for a layout this PC does not have.
pub fn check(id: LayoutId) -> Result<(), LayoutError> {
    if WinLayouts.installed().contains(&id) {
        Ok(())
    } else {
        Err(LayoutError::NotInstalled(id))
    }
}

/// Switches to installed layout `id` once the window has the keyboard, and keeps the previous one for `undo`.
///
/// # Errors
/// `Refused` when the layout did not change, and the previous one is back; `NotBack` when it did not come back
/// either. Either way the window never opens, so a gate fails at its start.
pub fn apply(id: LayoutId) -> Result<(), LayoutError> {
    let back = layout::switch(&Win32, id)?;
    *BACK.lock().unwrap_or_else(PoisonError::into_inner) = Some(back);
    Ok(())
}

/// Puts back the layout from before `apply`; the window calls it at close, while it still has the keyboard.
///
/// # Errors
/// `NotBack` when the previous layout did not come back; nothing to put back is fine.
pub fn undo() -> Result<(), LayoutError> {
    let back = BACK.lock().unwrap_or_else(PoisonError::into_inner).take();
    back.map_or(Ok(()), |back| layout::restore(&Win32, back))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_an_installed_layout_passes_the_check() {
        let gone = LayoutId::new(0x7FFF_0000_0000_0000);
        assert_eq!(check(gone), Err(LayoutError::NotInstalled(gone)));
        let first = WinLayouts.installed()[0];
        assert_eq!(check(first), Ok(()));
    }
}
