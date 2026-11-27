//! The system's Save and folder dialogs on Windows. Each runs on this thread, made per-monitor DPI aware, owned by a
//! hidden topmost window of ours so it opens above our topmost windows, and adds nothing to the recent lists.

use windows::Win32::Foundation::{ERROR_CANCELLED, ERROR_NO_UNICODE_TRANSLATION, HWND};
use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE, CoCreateInstance,
    CoTaskMemFree,
};
use windows::Win32::UI::Shell::Common::COMDLG_FILTERSPEC;
use windows::Win32::UI::Shell::{
    FILEOPENDIALOGOPTIONS, FOS_DONTADDTORECENT, FOS_FORCEFILESYSTEM, FOS_NOCHANGEDIR,
    FOS_OVERWRITEPROMPT, FOS_PICKFOLDERS, FOS_STRICTFILETYPES, FileOpenDialog, FileSaveDialog,
    IFileDialog, IShellItem, SHCreateItemFromParsingName, SIGDN_FILESYSPATH,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DestroyWindow, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
};
use windows::core::{GUID, HSTRING, PCWSTR, w};

use super::{Dialogs, FileKind, Picked, Unshown, pattern};
use crate::win::{Com, physical};

/// Every dialog's options: a file-system path only, our working folder kept, nothing added to the recent lists.
const EVERY: FILEOPENDIALOGOPTIONS =
    FILEOPENDIALOGOPTIONS(FOS_FORCEFILESYSTEM.0 | FOS_NOCHANGEDIR.0 | FOS_DONTADDTORECENT.0);
/// The Save dialog's own: it asks before it replaces a file, and keeps the one type offered.
const SAVE: FILEOPENDIALOGOPTIONS =
    FILEOPENDIALOGOPTIONS(FOS_OVERWRITEPROMPT.0 | FOS_STRICTFILETYPES.0);
/// A system window class the hidden owner borrows, so none is registered.
const OWNER_CLASS: PCWSTR = w!("Static");

/// The system's Save and folder dialogs.
#[derive(Clone, Copy, Debug, Default)]
pub struct WinDialogs;

impl Dialogs for WinDialogs {
    fn save(&self, folder: &str, name: &str, kind: FileKind) -> Result<Picked, Unshown> {
        show(|| save_dialog(folder, name, kind))
    }

    fn folder(&self, at: &str) -> Result<Picked, Unshown> {
        show(|| dialog(&FileOpenDialog, at, FOS_PICKFOLDERS))
    }
}

impl From<windows::core::Error> for Unshown {
    /// Keeps the error's `HRESULT`, which names no path.
    fn from(e: windows::core::Error) -> Self {
        Self {
            code: e.code().0.cast_unsigned(),
        }
    }
}

/// Starts COM on this thread, makes it per-monitor aware, then shows the dialog `make` sets up over a hidden topmost
/// owner and reads its answer.
#[allow(unsafe_code, reason = "Shows a dialog of ours.")]
fn show(make: impl FnOnce() -> windows::core::Result<IFileDialog>) -> Result<Picked, Unshown> {
    let _com = Com::start(COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE)?;
    physical();
    let owner = Owner::new()?;
    let dialog = make()?;
    // SAFETY: a live dialog and our live owner window, both on this thread.
    match unsafe { dialog.Show(Some(owner.0)) } {
        Ok(()) => chosen(&dialog).map(Picked::Path),
        Err(e) if e.code() == ERROR_CANCELLED.to_hresult() => Ok(Picked::Cancelled),
        Err(e) => Err(e.into()),
    }
}

/// A dialog of `class` with `own` options beside `EVERY`, opening at `folder`, or where it chooses once that is gone.
#[allow(unsafe_code, reason = "Sets up a dialog of ours.")]
fn dialog(
    class: &GUID,
    folder: &str,
    own: FILEOPENDIALOGOPTIONS,
) -> windows::core::Result<IFileDialog> {
    let folder = HSTRING::from(folder);
    // SAFETY: a new in-process dialog, set from live values that outlive each call.
    unsafe {
        let dialog: IFileDialog = CoCreateInstance(class, None, CLSCTX_INPROC_SERVER)?;
        let at: windows::core::Result<IShellItem> = SHCreateItemFromParsingName(&folder, None);
        if let Ok(at) = at {
            dialog.SetFolder(&at)?;
        }
        dialog.SetOptions(dialog.GetOptions()? | EVERY | own)?;
        Ok(dialog)
    }
}

/// The Save dialog at `folder`, proposing `name` and offering only `kind`, whose extension it adds when missing.
#[allow(unsafe_code, reason = "Sets up a dialog of ours.")]
fn save_dialog(folder: &str, name: &str, kind: FileKind) -> windows::core::Result<IFileDialog> {
    let dialog = dialog(&FileSaveDialog, folder, SAVE)?;
    let (name, ext, spec) = (
        HSTRING::from(name),
        HSTRING::from(kind.ext()),
        HSTRING::from(pattern(kind)),
    );
    let types = [COMDLG_FILTERSPEC {
        pszName: PCWSTR(spec.as_ptr()),
        pszSpec: PCWSTR(spec.as_ptr()),
    }];
    // SAFETY: our dialog, not yet shown, set from live values that outlive each call.
    unsafe {
        dialog.SetFileTypes(&types)?;
        dialog.SetDefaultExtension(&ext)?;
        dialog.SetFileName(&name)?;
    }
    Ok(dialog)
}

/// The path the closed dialog answered; a name that is not whole UTF-16 cannot be told, so it is no answer.
#[allow(unsafe_code, reason = "Reads and frees the dialog's answer.")]
fn chosen(dialog: &IFileDialog) -> Result<String, Unshown> {
    // SAFETY: the dialog closed with an answer; the name it hands back is ours, read once and freed once.
    unsafe {
        let name = dialog.GetResult()?.GetDisplayName(SIGDN_FILESYSPATH)?;
        let path = String::from_utf16(name.as_wide());
        CoTaskMemFree(Some(name.0.cast()));
        path.map_err(|_| {
            windows::core::Error::from(ERROR_NO_UNICODE_TRANSLATION.to_hresult()).into()
        })
    }
}

/// A hidden topmost window that owns the dialog, so the dialog is topmost too; destroyed when dropped.
struct Owner(HWND);

impl Owner {
    /// Makes the window, never shown.
    #[allow(unsafe_code, reason = "Makes a hidden window of ours.")]
    fn new() -> Result<Self, Unshown> {
        let style = WS_EX_TOPMOST | WS_EX_TOOLWINDOW;
        // SAFETY: a borrowed system class with no parent, menu or creation data.
        let made = unsafe {
            CreateWindowExW(
                style,
                OWNER_CLASS,
                PCWSTR::null(),
                WS_POPUP,
                0,
                0,
                0,
                0,
                None,
                None,
                None,
                None,
            )
        };
        Ok(Self(made?))
    }
}

impl Drop for Owner {
    #[allow(unsafe_code, reason = "Destroys a window of ours.")]
    fn drop(&mut self) {
        // SAFETY: our window, destroyed once on the thread that made it.
        let _ = unsafe { DestroyWindow(self.0) };
    }
}
