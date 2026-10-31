//! A clipboard owner that outlives its writes: a hidden window of the gate, so the product names the gate as a copy's
//! app and follows its clear. A write through `clip::write_text` ends its window at once, so its copy has no app.
#![cfg(windows)]

use std::time::Instant;

use kx_platform_windows::clip::open::format_id;
use windows::Win32::Foundation::{GlobalFree, HANDLE, HWND};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
};
use windows::Win32::System::Memory::{GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalUnlock};
use windows::Win32::System::Ole::CF_UNICODETEXT;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DestroyWindow, DispatchMessageW, HWND_MESSAGE, MSG, PM_REMOVE, PeekMessageW,
    WINDOW_EX_STYLE, WINDOW_STYLE,
};
use windows::core::{PCWSTR, w};

use crate::clipfill::{self, OFF};
use crate::config::Timing;
use crate::win::{self, sleep_ms};

/// A system window class, so the owner needs no class of its own.
const CLASS: PCWSTR = w!("STATIC");

/// The gate's own clipboard owner: a message-only window of this thread, ended on drop.
pub struct Owner(HWND);

/// `text` as Windows' Unicode text format holds it: UTF-16 with a closing zero.
fn wide_z(text: &str) -> Vec<u8> {
    text.encode_utf16()
        .chain([0])
        .flat_map(u16::to_ne_bytes)
        .collect()
}

/// Puts `data` on the open, emptied clipboard as `format`.
#[allow(unsafe_code, reason = "The clipboard is open for our window.")]
fn put(format: u32, data: &[u8]) -> Result<(), String> {
    // SAFETY: the clipboard is open; `mem` is sized for `data` and locked while it is filled; the clipboard owns it
    // once `SetClipboardData` succeeds, else it is freed here.
    unsafe {
        let mem =
            GlobalAlloc(GMEM_MOVEABLE, data.len()).map_err(|e| format!("GlobalAlloc: {e}"))?;
        let ptr = GlobalLock(mem).cast::<u8>();
        if ptr.is_null() {
            let _ = GlobalFree(Some(mem));
            return Err("GlobalLock gave no memory".to_string());
        }
        std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());
        let _ = GlobalUnlock(mem);
        if let Err(e) = SetClipboardData(format, Some(HANDLE(mem.0))) {
            let _ = GlobalFree(Some(mem));
            return Err(format!("SetClipboardData: {e}"));
        }
    }
    Ok(())
}

impl Owner {
    /// A fresh owner window.
    #[allow(unsafe_code, reason = "A plain call with a system class.")]
    pub fn new() -> Result<Self, String> {
        let (ex, style) = (WINDOW_EX_STYLE(0), WINDOW_STYLE(0));
        // SAFETY: a plain call with a system class; the window is ended on drop.
        let made = unsafe {
            CreateWindowExW(
                ex,
                CLASS,
                PCWSTR::null(),
                style,
                0,
                0,
                0,
                0,
                Some(HWND_MESSAGE),
                None,
                None,
                None,
            )
        };
        made.map(Self)
            .map_err(|e| format!("the clipboard owner window: {e}"))
    }

    /// Puts `text` on the clipboard as this owner, with the cloud switch off, or empties it for none.
    #[allow(unsafe_code, reason = "Opens and closes the clipboard for our window.")]
    pub fn write(&self, text: Option<&str>, t: &Timing) -> Result<(), String> {
        let cloud = format_id(&clipfill::cloud_off()?).map_err(|e| e.to_string())?;
        // SAFETY: our own live window; the open is closed below whatever the fill gave.
        let open = || unsafe { OpenClipboard(Some(self.0)) }.is_ok().then_some(());
        win::poll_until(t.read_wait_ms, t.poll_ms, open).ok_or("the clipboard stayed busy")?;
        let filled = fill(text, cloud);
        // SAFETY: the clipboard was opened above by this thread.
        let _ = unsafe { CloseClipboard() };
        filled
    }

    /// Keeps the window answering its messages for `ms`, so an app that empties the clipboard is never held up by it.
    #[allow(unsafe_code, reason = "Takes and hands on the window's own messages.")]
    pub fn hold(&self, ms: u64, poll_ms: u64) {
        let start = Instant::now();
        while start.elapsed().as_millis() < u128::from(ms) {
            let mut msg = MSG::default();
            // SAFETY: plain calls on this thread's queue with a local message that outlives them.
            while unsafe { PeekMessageW(&raw mut msg, Some(self.0), 0, 0, PM_REMOVE) }.as_bool() {
                // SAFETY: a message just taken from this thread's queue.
                unsafe { DispatchMessageW(&raw const msg) };
            }
            sleep_ms(poll_ms);
        }
    }
}

/// Empties the open clipboard, then puts `text` on it with format `cloud` off; empty alone for none.
#[allow(unsafe_code, reason = "The clipboard is open for our window.")]
fn fill(text: Option<&str>, cloud: u32) -> Result<(), String> {
    // SAFETY: the clipboard is open for our window on this thread.
    unsafe { EmptyClipboard() }.map_err(|e| format!("EmptyClipboard: {e}"))?;
    let Some(text) = text else {
        return Ok(());
    };
    put(u32::from(CF_UNICODETEXT.0), &wide_z(text))?;
    put(cloud, &OFF)
}

impl Drop for Owner {
    #[allow(unsafe_code, reason = "Ends our own window.")]
    fn drop(&mut self) {
        // SAFETY: our own window, made on this thread and never used again.
        let _ = unsafe { DestroyWindow(self.0) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unicode_text_is_utf16_with_a_closing_zero() {
        assert_eq!(wide_z(""), [0, 0]);
        assert_eq!(wide_z("a").len(), 4);
        let ar = wide_z("قهوة");
        assert_eq!(ar.len(), (4 + 1) * 2);
        assert_eq!(&ar[ar.len() - 2..], &[0, 0]);
    }
}
