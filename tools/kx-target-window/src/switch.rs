//! Switches the window's own thread to an installed layout; the user's layout list and default stay as they are.

use kx_module_api::ports::{LayoutId, LayoutSource};
use kx_platform_windows::{WinLayouts, layouts};
use kx_target_window::layout::LayoutError;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    ACTIVATE_KEYBOARD_LAYOUT_FLAGS, ActivateKeyboardLayout, GetKeyboardLayout,
};

/// `ActivateKeyboardLayout` flags: none, so only this thread switches.
const THIS_THREAD: ACTIVATE_KEYBOARD_LAYOUT_FLAGS = ACTIVATE_KEYBOARD_LAYOUT_FLAGS(0);
/// `GetKeyboardLayout` thread id that means this thread.
const OWN_THREAD: u32 = 0;

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

/// Switches this thread to installed layout `id` and reads it back; call it once the window has the keyboard.
///
/// # Errors
/// `NotInstalled` for an id that is no handle, `Refused` when this thread's layout did not change.
#[allow(
    unsafe_code,
    reason = "Switches this thread's layout, then reads it back."
)]
pub fn apply(id: LayoutId) -> Result<(), LayoutError> {
    let handle = layouts::hkl(id).ok_or(LayoutError::NotInstalled(id))?;
    // SAFETY: switches only this thread, to a layout Windows lists as installed.
    let switched = unsafe { ActivateKeyboardLayout(handle, THIS_THREAD) }.is_ok();
    // SAFETY: a plain query of this thread's layout.
    let now = layouts::id(unsafe { GetKeyboardLayout(OWN_THREAD) });
    (switched && now == id)
        .then_some(())
        .ok_or(LayoutError::Refused(id))
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
