//! Reads and writes Unicode text on the clipboard, with a cap on how much is read.

use windows::Win32::Foundation::{GlobalFree, HANDLE, HGLOBAL};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, EnumClipboardFormats, GetClipboardData,
    GetClipboardSequenceNumber, IsClipboardFormatAvailable, OpenClipboard, SetClipboardData,
};
use windows::Win32::System::Memory::{
    GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock,
};
use windows::Win32::System::Ole::{CF_LOCALE, CF_OEMTEXT, CF_TEXT, CF_UNICODETEXT};

use crate::win::poll_until;

/// Most UTF-16 units one character takes.
const UNITS_PER_CHAR: usize = 2;

/// Closes the clipboard when dropped.
struct Open;

impl Drop for Open {
    fn drop(&mut self) {
        // SAFETY: only built after a successful `OpenClipboard`.
        let _ = unsafe { CloseClipboard() };
    }
}

/// The clipboard change counter.
pub fn sequence() -> u32 {
    // SAFETY: plain query.
    unsafe { GetClipboardSequenceNumber() }
}

/// Waits up to `timeout_ms` for the clipboard to change from `before`; true if it did.
pub fn wait_change(before: u32, timeout_ms: u64, poll_ms: u64) -> bool {
    poll_until(timeout_ms, poll_ms, || (sequence() != before).then_some(())).is_some()
}

/// Opens the clipboard, trying again while another app holds it.
fn open(timeout_ms: u64, poll_ms: u64) -> Result<Open, String> {
    let mut last = String::new();
    let try_open = || {
        // SAFETY: plain call; a busy clipboard gives an error and we retry.
        match unsafe { OpenClipboard(None) } {
            Ok(()) => Some(Open),
            Err(e) => {
                last = e.to_string();
                None
            }
        }
    };
    poll_until(timeout_ms, poll_ms, try_open).ok_or_else(|| format!("OpenClipboard: {last}"))
}

/// True when the clipboard is empty or holds only text, which `write_text` can put back whole.
pub fn only_text(timeout_ms: u64, poll_ms: u64) -> Result<bool, String> {
    let _open = open(timeout_ms, poll_ms)?;
    let text = [CF_TEXT, CF_OEMTEXT, CF_UNICODETEXT, CF_LOCALE].map(|f| u32::from(f.0));
    let mut format = 0;
    loop {
        // SAFETY: the clipboard is open; zero starts the list and ends it.
        format = unsafe { EnumClipboardFormats(format) };
        if format == 0 {
            return Ok(true);
        }
        if !text.contains(&format) {
            return Ok(false);
        }
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
    let _open = open(timeout_ms, poll_ms)?;
    let format = u32::from(CF_UNICODETEXT.0);
    // SAFETY: the clipboard is open; the handle is locked only while we copy out of it,
    // and we never read past `GlobalSize`.
    unsafe {
        if IsClipboardFormatAvailable(format).is_err() {
            return Ok(None);
        }
        let handle = GetClipboardData(format).map_err(|e| format!("GetClipboardData: {e}"))?;
        let mem = HGLOBAL(handle.0);
        let ptr = GlobalLock(mem).cast::<u16>();
        if ptr.is_null() {
            return Err("GlobalLock failed".to_string());
        }
        let units = std::slice::from_raw_parts(ptr, GlobalSize(mem) / size_of::<u16>());
        let text = text_of(units, max_chars);
        let _ = GlobalUnlock(mem);
        text.map(Some)
    }
}

/// Puts `text` on the clipboard, or leaves it empty for `None`.
pub fn write_text(text: Option<&str>, timeout_ms: u64, poll_ms: u64) -> Result<(), String> {
    let _open = open(timeout_ms, poll_ms)?;
    // SAFETY: the clipboard is open; `mem` is sized for `units` and locked while we fill it;
    // the clipboard owns it once `SetClipboardData` succeeds, else we free it.
    unsafe {
        EmptyClipboard().map_err(|e| format!("EmptyClipboard: {e}"))?;
        let Some(text) = text else {
            return Ok(());
        };
        let units: Vec<u16> = text.encode_utf16().chain([0]).collect();
        let mem = GlobalAlloc(GMEM_MOVEABLE, units.len() * size_of::<u16>())
            .map_err(|e| format!("GlobalAlloc: {e}"))?;
        let ptr = GlobalLock(mem).cast::<u16>();
        if ptr.is_null() {
            let _ = GlobalFree(Some(mem));
            return Err("GlobalLock failed".to_string());
        }
        std::ptr::copy_nonoverlapping(units.as_ptr(), ptr, units.len());
        let _ = GlobalUnlock(mem);
        match SetClipboardData(u32::from(CF_UNICODETEXT.0), Some(HANDLE(mem.0))) {
            Ok(_) => Ok(()),
            Err(e) => {
                let _ = GlobalFree(Some(mem));
                Err(format!("SetClipboardData: {e}"))
            }
        }
    }
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
