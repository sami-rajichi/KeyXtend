//! The open clipboard: its formats with their sizes and bytes, put and empty, and the change counter. The watch, the
//! writer and the gate tool's tests share it.

use std::marker::PhantomData;
use std::time::{Duration, Instant};

use kx_module_api::ports::{ClipError, ClipWait};
use windows::Win32::Foundation::{GlobalFree, HANDLE, HGLOBAL, HWND};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, EnumClipboardFormats, GetClipboardData,
    GetClipboardSequenceNumber, IsClipboardFormatAvailable, OpenClipboard,
    RegisterClipboardFormatW, SetClipboardData,
};
use windows::Win32::System::Memory::{
    GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DestroyWindow, HWND_MESSAGE, WINDOW_EX_STYLE, WINDOW_STYLE,
};
use windows::core::{HSTRING, PCWSTR, w};

/// First id of a registered (named) clipboard format.
pub const FIRST_NAMED: u32 = 0xC000;
/// The system class of our hidden owner windows, which need no procedure of ours.
const OWNER_CLASS: PCWSTR = w!("STATIC");

/// The clipboard, open until dropped on the thread that opened it; the functions that need it open take it.
pub struct Open {
    /// Our own hidden owner window, destroyed after the clipboard closes.
    temp: Option<HWND>,
    /// Not `Send`: the clipboard must close on the thread that opened it.
    _here: PhantomData<*const ()>,
}

impl Drop for Open {
    #[allow(unsafe_code, reason = "Built only after `OpenClipboard` succeeded.")]
    fn drop(&mut self) {
        // SAFETY: only built after a successful `OpenClipboard` on this thread; `temp` is ours.
        unsafe {
            let _ = CloseClipboard();
            if let Some(w) = self.temp {
                let _ = DestroyWindow(w);
            }
        }
    }
}

/// The clipboard change counter.
#[must_use]
#[allow(unsafe_code, reason = "Plain query.")]
pub fn sequence() -> u32 {
    // SAFETY: plain query.
    unsafe { GetClipboardSequenceNumber() }
}

/// Waits up to `wait.open_ms` for the clipboard to change from `before`; true if it did.
#[must_use]
pub fn wait_change(before: u32, wait: ClipWait) -> bool {
    retry(wait, || sequence() != before)
}

/// A hidden message-only window of this thread, to own the clipboard; `close_window` ends it.
///
/// # Errors
/// `Refused` when Windows makes no window.
#[allow(unsafe_code, reason = "A plain call with a system class.")]
pub(crate) fn owner_window() -> Result<HWND, ClipError> {
    let none = (WINDOW_EX_STYLE(0), WINDOW_STYLE(0));
    // SAFETY: a plain call with a system class; the caller destroys the window.
    unsafe {
        CreateWindowExW(
            none.0,
            OWNER_CLASS,
            PCWSTR::null(),
            none.1,
            0,
            0,
            0,
            0,
            Some(HWND_MESSAGE),
            None,
            None,
            None,
        )
    }
    .map_err(|_| ClipError::Refused)
}

/// Ends a window `owner_window` made on this thread.
#[allow(unsafe_code, reason = "Destroys our own window.")]
pub(crate) fn close_window(window: HWND) {
    // SAFETY: our own window, made on this thread and never used again.
    let _ = unsafe { DestroyWindow(window) };
}

/// Opens the clipboard with a fresh owner window, trying again while another app holds it.
///
/// # Errors
/// `Busy` past `wait.open_ms`, `Refused` when Windows makes no owner window.
pub fn open(wait: ClipWait) -> Result<Open, ClipError> {
    // A NULL owner is not enough: two NULL opens were seen holding the clipboard at once, and `SetClipboardData`
    // fails after `EmptyClipboard` without one.
    let temp = owner_window()?;
    open_with(temp, Some(temp), wait)
}

/// Opens the clipboard for `owner`, a window of this thread that outlives the open.
///
/// # Errors
/// `Busy` past `wait.open_ms`.
pub(crate) fn open_for(owner: HWND, wait: ClipWait) -> Result<Open, ClipError> {
    open_with(owner, None, wait)
}

/// Opens for `owner`; `temp` is destroyed on drop, or at once when the open fails.
#[allow(unsafe_code, reason = "Plain call.")]
fn open_with(owner: HWND, temp: Option<HWND>, wait: ClipWait) -> Result<Open, ClipError> {
    // SAFETY: plain call; a busy clipboard gives an error and we retry.
    if !retry(wait, || unsafe { OpenClipboard(Some(owner)) }.is_ok()) {
        if let Some(w) = temp {
            close_window(w);
        }
        return Err(ClipError::Busy);
    }
    Ok(Open {
        temp,
        _here: PhantomData,
    })
}

/// Calls `f` every `wait.poll_ms` until it is true or `wait.open_ms` has passed; true when it was.
fn retry(wait: ClipWait, mut f: impl FnMut() -> bool) -> bool {
    let end = Instant::now() + Duration::from_millis(wait.open_ms.into());
    loop {
        if f() {
            return true;
        }
        if Instant::now() >= end {
            return false;
        }
        std::thread::sleep(Duration::from_millis(wait.poll_ms.into()));
    }
}

/// The formats on the open clipboard, in its order.
#[must_use]
#[allow(unsafe_code, reason = "The clipboard is open.")]
pub fn formats(_open: &Open) -> Vec<u32> {
    let (mut out, mut format) = (Vec::new(), 0);
    loop {
        // SAFETY: the clipboard is open; zero starts the list and ends it.
        format = unsafe { EnumClipboardFormats(format) };
        if format == 0 {
            return out;
        }
        out.push(format);
    }
}

/// Whether the open clipboard holds `format`.
#[must_use]
#[allow(unsafe_code, reason = "The clipboard is open.")]
pub fn available(_open: &Open, format: u32) -> bool {
    // SAFETY: a plain query; the clipboard is open.
    unsafe { IsClipboardFormatAvailable(format) }.is_ok()
}

/// The memory of `format`; `None` when the clipboard has none for it.
#[allow(unsafe_code, reason = "The clipboard is open.")]
fn memory(open: &Open, format: u32) -> Option<HGLOBAL> {
    // SAFETY: the clipboard is open, and the handle stays the clipboard's.
    available(open, format)
        .then(|| unsafe { GetClipboardData(format) }.ok())
        .flatten()
        .map(|h| HGLOBAL(h.0))
}

/// The bytes `format` holds, without reading them; `None` when it has no memory data.
#[must_use]
#[allow(unsafe_code, reason = "The clipboard is open.")]
pub fn size(open: &Open, format: u32) -> Option<u64> {
    // SAFETY: a plain query of the clipboard's own memory, valid while it is open.
    memory(open, format).map(|mem| u64::try_from(unsafe { GlobalSize(mem) }).unwrap_or(u64::MAX))
}

/// Runs `f` on the memory of `format` while it is locked; `None` when it has no memory data.
#[allow(unsafe_code, reason = "The clipboard is open.")]
pub fn with_data<T>(open: &Open, format: u32, f: impl FnOnce(&[u8]) -> T) -> Option<T> {
    let mem = memory(open, format)?;
    // SAFETY: the clipboard is open; the memory is locked only while `f` reads its `GlobalSize` bytes.
    unsafe {
        let ptr = GlobalLock(mem).cast::<u8>();
        if ptr.is_null() {
            return None;
        }
        let got = f(std::slice::from_raw_parts(ptr, GlobalSize(mem)));
        let _ = GlobalUnlock(mem);
        Some(got)
    }
}

/// The 32-bit number held by format `name` on the open clipboard, if it holds one.
#[must_use]
pub fn dword(open: &Open, name: &str) -> Option<u32> {
    dword_of(open, format_id(name).ok()?)
}

/// The 32-bit number held by `format` on the open clipboard, if it holds one.
pub(crate) fn dword_of(open: &Open, format: u32) -> Option<u32> {
    let bytes = with_data(open, format, |b| b.first_chunk::<4>().copied())??;
    Some(u32::from_ne_bytes(bytes))
}

/// The id of the registered format `name`.
///
/// # Errors
/// `Refused` when Windows will not register it.
#[allow(unsafe_code, reason = "A plain call with a live string.")]
pub fn format_id(name: &str) -> Result<u32, ClipError> {
    // SAFETY: a plain call with a live string.
    match unsafe { RegisterClipboardFormatW(&HSTRING::from(name)) } {
        0 => Err(ClipError::Refused),
        id => Ok(id),
    }
}

/// Puts `data` on the open, emptied clipboard as `format`.
///
/// # Errors
/// `Io` when the memory cannot be made or Windows refuses it.
#[allow(unsafe_code, reason = "The clipboard is open.")]
pub fn put(_open: &Open, format: u32, data: &[u8]) -> Result<(), ClipError> {
    // SAFETY: the clipboard is open; `mem` is sized for `data` and locked while we fill it;
    // the clipboard owns it once `SetClipboardData` succeeds, else we free it.
    unsafe {
        let mem = GlobalAlloc(GMEM_MOVEABLE, data.len()).map_err(|_| ClipError::Io)?;
        let ptr = GlobalLock(mem).cast::<u8>();
        if ptr.is_null() {
            let _ = GlobalFree(Some(mem));
            return Err(ClipError::Io);
        }
        std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());
        let _ = GlobalUnlock(mem);
        if SetClipboardData(format, Some(HANDLE(mem.0))).is_err() {
            let _ = GlobalFree(Some(mem));
            return Err(ClipError::Io);
        }
    }
    Ok(())
}

/// Empties the open clipboard and makes its window the owner.
///
/// # Errors
/// `Refused` when Windows refuses.
#[allow(unsafe_code, reason = "The clipboard is open.")]
pub fn empty(_open: &Open) -> Result<(), ClipError> {
    // SAFETY: the clipboard is open.
    unsafe { EmptyClipboard() }.map_err(|_| ClipError::Refused)
}
