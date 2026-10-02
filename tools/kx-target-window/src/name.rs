//! Gives the text box an accessible name through Windows' dynamic annotation, so Narrator and NVDA read it.

use kx_platform_windows::com::Com;
use windows::Win32::Foundation::HWND;
use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance};
use windows::Win32::UI::Accessibility::{CAccPropServices, IAccPropServices, PROPID_ACC_NAME};
use windows::Win32::UI::WindowsAndMessaging::{CHILDID_SELF, OBJID_CLIENT};
use windows::core::HSTRING;

/// Names window `edit` `name` for screen readers; the name stays with the window.
///
/// # Errors
/// The reason, when COM or the annotation service refuses.
#[allow(unsafe_code, reason = "COM calls that annotate our own window.")]
pub fn set(edit: HWND, name: &str) -> Result<(), String> {
    // The annotation service may be used again later on this thread, so COM stays on.
    Com::start().map_err(|e| e.to_string())?.keep();
    // SAFETY: creates the system's annotation service object.
    let props: IAccPropServices =
        unsafe { CoCreateInstance(&CAccPropServices, None, CLSCTX_INPROC_SERVER) }
            .map_err(|e| e.to_string())?;
    let client = OBJID_CLIENT.0.cast_unsigned();
    // SAFETY: `edit` is our live window, and the call copies the name.
    unsafe {
        props.SetHwndPropStr(
            edit,
            client,
            CHILDID_SELF,
            PROPID_ACC_NAME,
            &HSTRING::from(name),
        )
    }
    .map_err(|e| e.to_string())
}
