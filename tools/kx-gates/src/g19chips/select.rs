//! Selections the gate makes in its own document with no input, as an app's own caret step or a script would: the
//! whole document selected, or the caret moved one character back. UI Automation tells the module as the app does.
#![cfg(windows)]

use kx_platform_windows::uia::{self, Uia};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Accessibility::{IUIAutomationTextPattern, TextUnit_Character};

/// The caret step: one character back.
const BACK: i32 = -1;

/// The text pattern of the document in `hwnd`.
fn pattern(uia: &Uia, hwnd: HWND) -> Result<IUIAutomationTextPattern, String> {
    let doc = uia.text(hwnd).ok_or("the document shows no text")?;
    uia.text_of(&doc)
        .ok_or_else(|| "the document has no text pattern".to_string())
}

/// Selects the whole document in `hwnd`.
#[allow(unsafe_code, reason = "Plain UI Automation text range calls.")]
pub fn all(uia: &Uia, hwnd: HWND) -> Result<(), String> {
    let tp = pattern(uia, hwnd)?;
    // SAFETY: plain calls on live UI Automation objects.
    unsafe { tp.DocumentRange().and_then(|r| r.Select()) }.map_err(|e| format!("select all: {e}"))
}

/// Moves the bare caret in `hwnd` one character back, as the app's own caret step.
#[allow(unsafe_code, reason = "Plain UI Automation text range calls.")]
pub fn back(uia: &Uia, hwnd: HWND) -> Result<(), String> {
    let tp = pattern(uia, hwnd)?;
    let step = || -> windows::core::Result<()> {
        // SAFETY: plain calls on live UI Automation objects; the range is our own copy.
        unsafe {
            let at = tp.GetSelection()?.GetElement(0)?;
            at.Move(TextUnit_Character, BACK)?;
            at.Select()
        }
    };
    step().map_err(|e| format!("the caret step: {e}"))
}

/// The caret's place in `hwnd`, in characters from the start, up to `max`.
pub fn caret(uia: &Uia, hwnd: HWND, max: usize) -> Result<usize, String> {
    uia::caret(&pattern(uia, hwnd)?, max).map_err(|e| e.to_string())
}

/// True when the caret went from `before` to exactly one character back.
pub fn stepped(before: usize, after: usize) -> bool {
    after.checked_add(1) == Some(before)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_one_character_back_is_a_caret_step() {
        assert!(stepped(5, 4));
        assert!(!stepped(5, 5), "no move");
        assert!(!stepped(5, 3) && !stepped(4, 5) && !stepped(0, 0));
    }
}
