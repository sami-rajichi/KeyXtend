//! The running Explorer's desktop shell object, under an apartment-threaded COM on the calling thread: its
//! `ShellExecute` opens a folder or a link in Explorer itself (Raymond Chen's "launch from the shell" route), so nothing
//! starts as our child and nothing inherits our uiAccess. The reveal and the link both use it.

use windows::Win32::Foundation::E_FAIL;
use windows::Win32::System::Com::{
    CLSCTX_LOCAL_SERVER, CoCreateInstance, IDispatch, IServiceProvider,
};
use windows::Win32::System::Variant::VARIANT;
use windows::Win32::UI::Shell::{
    CSIDL_DESKTOP, IShellBrowser, IShellDispatch2, IShellFolderViewDual, IShellView, IShellWindows,
    SID_STopLevelBrowser, SVGIO_BACKGROUND, SWC_DESKTOP, SWFO_NEEDDISPATCH, ShellWindows,
};
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
use windows::core::{BSTR, Interface};

use crate::PlatformError;
use crate::com::Com;

/// The shell verb that opens a folder in Explorer or a link in the default browser.
const OPEN: &str = "open";
/// No arguments and no working folder: the empty strings the recipe passes, not empty variants.
const NONE: &str = "";

/// Explorer's shell object, with COM kept on until it is released.
pub(crate) struct Explorer {
    /// Released before COM stops, as fields drop in order.
    shell: IShellDispatch2,
    _com: Com,
}

impl Explorer {
    /// Starts COM on this thread and finds Explorer's desktop shell object, or gives the code that refused.
    pub(crate) fn find() -> Result<Self, i32> {
        let com = Com::start_apartment().map_err(|e| com_code(&e))?;
        let shell = desktop_shell().map_err(|e| code(&e))?;
        Ok(Self { shell, _com: com })
    }

    /// Has Explorer open `file` with the `open` verb and no arguments, or gives the code that refused.
    #[allow(unsafe_code, reason = "Asks Explorer's shell object to open a file.")]
    pub(crate) fn open(&self, file: &BSTR) -> Result<(), i32> {
        let (empty, verb, show) = (
            VARIANT::from(NONE),
            VARIANT::from(OPEN),
            VARIANT::from(SW_SHOWNORMAL.0),
        );
        // SAFETY: a call on a live object made on this thread, with a BSTR and VARIANTs that outlive it.
        let opened = unsafe { self.shell.ShellExecute(file, &empty, &empty, &verb, &show) };
        opened.map_err(|e| code(&e))
    }
}

/// The shell object of Explorer's desktop window, reached through its view; the caller has COM on.
#[allow(
    unsafe_code,
    reason = "COM calls to the running Explorer's desktop window."
)]
fn desktop_shell() -> windows::core::Result<IShellDispatch2> {
    let desktop = VARIANT::from(i32::try_from(CSIDL_DESKTOP).unwrap_or_default());
    let root = VARIANT::default();
    let mut window: i32 = 0;
    // SAFETY: COM runs on this thread, and each call takes live objects and VARIANTs that outlive it.
    unsafe {
        let windows: IShellWindows = CoCreateInstance(&ShellWindows, None, CLSCTX_LOCAL_SERVER)?;
        let found = windows.FindWindowSW(
            &raw const desktop,
            &raw const root,
            SWC_DESKTOP,
            &raw mut window,
            SWFO_NEEDDISPATCH,
        )?;
        let browser: IShellBrowser = found
            .cast::<IServiceProvider>()?
            .QueryService(&SID_STopLevelBrowser)?;
        let view: IShellView = browser.QueryActiveShellView()?;
        let background: IDispatch = view.GetItemObject(SVGIO_BACKGROUND)?;
        background
            .cast::<IShellFolderViewDual>()?
            .Application()?
            .cast()
    }
}

/// The code a failed COM start gave, or Windows' generic failure; the error names no path or link.
fn com_code(e: &PlatformError) -> i32 {
    let raw = match e {
        PlatformError::Com(io) => io.raw_os_error(),
        _ => None,
    };
    raw.filter(|&c| c != 0).unwrap_or(E_FAIL.0)
}

/// The error's code, or Windows' generic failure for an error that holds none.
fn code(e: &windows::core::Error) -> i32 {
    let hr = e.code();
    if hr.is_ok() { E_FAIL.0 } else { hr.0 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::Foundation::E_OUTOFMEMORY;

    #[test]
    fn a_failed_com_start_keeps_its_own_code() {
        let refused = std::io::Error::from_raw_os_error(E_OUTOFMEMORY.0);
        assert_eq!(com_code(&PlatformError::Com(refused)), E_OUTOFMEMORY.0);
        assert_eq!(com_code(&PlatformError::NoExeDir), E_FAIL.0);
        let none = std::io::Error::from_raw_os_error(0);
        assert_eq!(com_code(&PlatformError::Com(none)), E_FAIL.0);
    }

    #[test]
    fn an_empty_error_keeps_a_failure_code() {
        assert_eq!(code(&windows::core::Error::empty()), E_FAIL.0);
        assert_eq!(code(&windows::core::Error::from(E_FAIL)), E_FAIL.0);
    }
}
