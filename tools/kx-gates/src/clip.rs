//! Reads and writes Unicode text on the clipboard, with a cap on how much is read, plus marker formats.
#![cfg(windows)]

use std::marker::PhantomData;

use windows::Win32::Foundation::{GlobalFree, HANDLE, HGLOBAL, HWND};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, EnumClipboardFormats, GetClipboardData,
    GetClipboardFormatNameW, GetClipboardSequenceNumber, IsClipboardFormatAvailable, OpenClipboard,
    RegisterClipboardFormatW, SetClipboardData,
};
use windows::Win32::System::Memory::{
    GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock,
};
use windows::Win32::System::Ole::CF_UNICODETEXT;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DestroyWindow, HWND_MESSAGE, WINDOW_EX_STYLE, WINDOW_STYLE,
};
use windows::core::{HSTRING, w};

use crate::win::poll_until;

/// Most UTF-16 units one character takes.
const UNITS_PER_CHAR: usize = 2;
/// First id of a registered (named) clipboard format.
pub const FIRST_NAMED: u32 = 0xC000;
/// Longest format name read, in UTF-16 units.
const NAME_MAX: usize = 256;
/// Every write by kx-gates carries this format holding 0, so Windows never syncs test texts to the cloud.
const CLOUD_OFF: &str = "CanUploadToCloudClipboard";

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
#[allow(unsafe_code, reason = "Plain query.")]
pub fn sequence() -> u32 {
    // SAFETY: plain query.
    unsafe { GetClipboardSequenceNumber() }
}

/// Waits up to `timeout_ms` for the clipboard to change from `before`; true if it did.
pub fn wait_change(before: u32, timeout_ms: u64, poll_ms: u64) -> bool {
    poll_until(timeout_ms, poll_ms, || (sequence() != before).then_some(())).is_some()
}

/// A hidden message-only window to own one clipboard open.
#[allow(unsafe_code, reason = "A plain call with a system class.")]
fn temp_window() -> Result<HWND, String> {
    let none = (WINDOW_EX_STYLE(0), WINDOW_STYLE(0));
    // SAFETY: a plain call with a system class; the window is destroyed when the `Open` drops.
    unsafe {
        CreateWindowExW(
            none.0,
            w!("STATIC"),
            w!(""),
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
    .map_err(|e| format!("clipboard owner window: {e}"))
}

/// Opens the clipboard with a fresh owner window, trying again while another app holds it.
pub fn open(timeout_ms: u64, poll_ms: u64) -> Result<Open, String> {
    // A NULL owner is not enough: two NULL opens were seen holding the clipboard at once.
    let temp = temp_window()?;
    open_with(temp, Some(temp), timeout_ms, poll_ms)
}

/// Opens the clipboard for `owner`, a window of this thread that outlives the open.
pub fn open_for(owner: HWND, timeout_ms: u64, poll_ms: u64) -> Result<Open, String> {
    open_with(owner, None, timeout_ms, poll_ms)
}

/// Opens for `owner`; `temp` is destroyed on drop, or at once when the open fails.
#[allow(unsafe_code, reason = "Plain call.")]
fn open_with(
    owner: HWND,
    temp: Option<HWND>,
    timeout_ms: u64,
    poll_ms: u64,
) -> Result<Open, String> {
    let mut last = String::new();
    let try_open = || {
        // SAFETY: plain call; a busy clipboard gives an error and we retry.
        match unsafe { OpenClipboard(Some(owner)) } {
            Ok(()) => Some(()),
            Err(e) => {
                last = e.to_string();
                None
            }
        }
    };
    if poll_until(timeout_ms, poll_ms, try_open).is_none() {
        if let Some(w) = temp {
            // SAFETY: our own window, never used again.
            let _ = unsafe { DestroyWindow(w) };
        }
        return Err(format!("OpenClipboard: {last}"));
    }
    Ok(Open {
        temp,
        _here: PhantomData,
    })
}

/// The formats on the open clipboard, in its order.
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

/// Runs `f` on the memory of `format` while it is locked; `None` when it has no memory data.
#[allow(unsafe_code, reason = "The clipboard is open.")]
pub fn with_data<T>(_open: &Open, format: u32, f: impl FnOnce(&[u8]) -> T) -> Option<T> {
    // SAFETY: the clipboard is open; the memory is locked only while `f` reads its `GlobalSize` bytes.
    unsafe {
        IsClipboardFormatAvailable(format).ok()?;
        let mem = HGLOBAL(GetClipboardData(format).ok()?.0);
        let ptr = GlobalLock(mem).cast::<u8>();
        if ptr.is_null() {
            return None;
        }
        let got = f(std::slice::from_raw_parts(ptr, GlobalSize(mem)));
        let _ = GlobalUnlock(mem);
        Some(got)
    }
}

/// The text before the first NUL; `Err` past `max_chars` characters.
fn text_of(units: &[u16], max_chars: usize) -> Result<String, String> {
    let limit = max_chars.saturating_mul(UNITS_PER_CHAR);
    let scan = &units[..units.len().min(limit.saturating_add(1))];
    let len = scan.iter().position(|&u| u == 0).unwrap_or(scan.len());
    let text = (len <= limit).then(|| String::from_utf16_lossy(&scan[..len]));
    match text {
        Some(t) if t.chars().count() <= max_chars => Ok(t),
        _ => Err(format!(
            "the clipboard holds more than {max_chars} characters"
        )),
    }
}

/// The clipboard text, or `None` when it holds none; opens it within `timeout_ms`.
pub fn read_text(
    timeout_ms: u64,
    poll_ms: u64,
    max_chars: usize,
) -> Result<Option<String>, String> {
    unicode(&open(timeout_ms, poll_ms)?, max_chars)
}

/// The text on the open clipboard, or `None` when it holds none; only the capped start is read.
pub fn unicode(open: &Open, max_chars: usize) -> Result<Option<String>, String> {
    let limit = max_chars.saturating_mul(UNITS_PER_CHAR).saturating_add(1);
    with_data(open, u32::from(CF_UNICODETEXT.0), |bytes| {
        let units: Vec<u16> = (bytes.as_chunks::<2>().0.iter().take(limit))
            .map(|b| u16::from_ne_bytes(*b))
            .collect();
        text_of(&units, max_chars)
    })
    .transpose()
}

/// The names of the registered formats on the open clipboard, such as exclusion markers.
#[allow(unsafe_code, reason = "The buffer is valid for its whole length.")]
pub fn names(open: &Open) -> Vec<String> {
    let name = |format: u32| {
        let mut buf = [0u16; NAME_MAX];
        // SAFETY: the buffer is valid for its whole length.
        let n = unsafe { GetClipboardFormatNameW(format, &mut buf) };
        let n = usize::try_from(n).unwrap_or(0).min(NAME_MAX);
        (n > 0).then(|| String::from_utf16_lossy(&buf[..n]))
    };
    formats(open)
        .into_iter()
        .filter(|&f| f >= FIRST_NAMED)
        .filter_map(name)
        .collect()
}

/// The 32-bit number held by format `name` on the open clipboard, if it holds one.
pub fn dword(open: &Open, name: &str) -> Option<u32> {
    let bytes = with_data(open, format_id(name).ok()?, |b| {
        b.first_chunk::<4>().copied()
    })??;
    Some(u32::from_ne_bytes(bytes))
}

/// The id of the registered format `name`.
#[allow(unsafe_code, reason = "A plain call with a live string.")]
fn format_id(name: &str) -> Result<u32, String> {
    // SAFETY: a plain call with a live string.
    match unsafe { RegisterClipboardFormatW(&HSTRING::from(name)) } {
        0 => Err(format!("RegisterClipboardFormat {name} failed")),
        id => Ok(id),
    }
}

/// Puts `data` on the open, emptied clipboard as `format`.
#[allow(unsafe_code, reason = "The clipboard is open.")]
pub fn put(_open: &Open, format: u32, data: &[u8]) -> Result<(), String> {
    // SAFETY: the clipboard is open; `mem` is sized for `data` and locked while we fill it;
    // the clipboard owns it once `SetClipboardData` succeeds, else we free it.
    unsafe {
        let mem =
            GlobalAlloc(GMEM_MOVEABLE, data.len()).map_err(|e| format!("GlobalAlloc: {e}"))?;
        let ptr = GlobalLock(mem).cast::<u8>();
        if ptr.is_null() {
            let _ = GlobalFree(Some(mem));
            return Err("GlobalLock failed".to_string());
        }
        std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());
        let _ = GlobalUnlock(mem);
        match SetClipboardData(format, Some(HANDLE(mem.0))) {
            Ok(_) => Ok(()),
            Err(e) => {
                let _ = GlobalFree(Some(mem));
                Err(format!("SetClipboardData: {e}"))
            }
        }
    }
}

/// Empties the open clipboard.
#[allow(unsafe_code, reason = "The clipboard is open.")]
pub fn empty(_open: &Open) -> Result<(), String> {
    // SAFETY: the clipboard is open.
    unsafe { EmptyClipboard() }.map_err(|e| format!("EmptyClipboard: {e}"))
}

/// Puts `text` on the clipboard, or leaves it empty for `None`.
pub fn write_text(text: Option<&str>, timeout_ms: u64, poll_ms: u64) -> Result<(), String> {
    write_marked(text, &[], timeout_ms, poll_ms)
}

/// Like `write_text`, plus each format in `marks` holding the number 0, in one clipboard change.
pub fn write_marked(
    text: Option<&str>,
    marks: &[&str],
    timeout_ms: u64,
    poll_ms: u64,
) -> Result<(), String> {
    let open = open(timeout_ms, poll_ms)?;
    empty(&open)?;
    let Some(text) = text else {
        return Ok(());
    };
    let units: Vec<u8> = text
        .encode_utf16()
        .chain([0])
        .flat_map(u16::to_ne_bytes)
        .collect();
    put(&open, u32::from(CF_UNICODETEXT.0), &units)?;
    for m in marks.iter().chain([&CLOUD_OFF]) {
        put(&open, format_id(m)?, &0u32.to_ne_bytes())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn units(s: &str) -> Vec<u16> {
        s.encode_utf16().collect()
    }

    #[test]
    fn text_stops_at_the_first_nul() {
        assert_eq!(text_of(&units("ab\0cd"), 2), Ok("ab".to_string()));
        assert_eq!(text_of(&units("لا€"), 3), Ok("لا€".to_string()));
        assert_eq!(text_of(&units("\u{1F600}"), 1), Ok("\u{1F600}".to_string()));
    }

    #[test]
    fn text_past_the_cap_is_an_error() {
        assert!(text_of(&units("abc"), 2).is_err());
        assert!(text_of(&units("abcdefgh\0"), 2).is_err());
    }
}
