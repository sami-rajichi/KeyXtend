//! Sends keys to the foreground app with `SendInput`.

use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_KEYBOARD, KEYBD_EVENT_FLAGS, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY,
    KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, KEYEVENTF_UNICODE, SendInput, VIRTUAL_KEY,
};

use crate::EXTENDED;

/// Marks input we injected, so our own hooks can skip it.
pub const TAG: usize = 0x4B58_5350;
/// Low byte of a scan code.
const SCAN_MASK: u32 = 0xFF;
/// Prefix byte of a scan code.
const PREFIX_MASK: u32 = 0xFF00;

fn key(scan: u16, flags: KEYBD_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(0),
                wScan: scan,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: TAG,
            },
        },
    }
}

/// Scan-code flags for `code`, adding the extended flag for `0xE0xx` codes.
fn scan_flags(code: u32) -> (u16, KEYBD_EVENT_FLAGS) {
    let mut flags = KEYEVENTF_SCANCODE;
    if code & PREFIX_MASK == EXTENDED {
        flags |= KEYEVENTF_EXTENDEDKEY;
    }
    ((code & SCAN_MASK) as u16, flags)
}

/// Presses `code` down, or releases it when `up`.
pub fn press(code: u32, up: bool) -> Result<(), String> {
    let (scan, mut flags) = scan_flags(code);
    if up {
        flags |= KEYEVENTF_KEYUP;
    }
    send(&[key(scan, flags)])
}

/// Taps `code` (down then up); the target app's layout decides the character.
pub fn tap(code: u32) -> Result<(), String> {
    let (scan, flags) = scan_flags(code);
    send(&[key(scan, flags), key(scan, flags | KEYEVENTF_KEYUP)])
}

/// Types `text` as Unicode characters, whatever the target's layout.
pub fn text(text: &str) -> Result<(), String> {
    let inputs: Vec<INPUT> = text
        .encode_utf16()
        .flat_map(|unit| {
            [
                key(unit, KEYEVENTF_UNICODE),
                key(unit, KEYEVENTF_UNICODE | KEYEVENTF_KEYUP),
            ]
        })
        .collect();
    send(&inputs)
}

fn send(inputs: &[INPUT]) -> Result<(), String> {
    // SAFETY: `inputs` is a valid slice and the size argument matches `INPUT`.
    let sent = unsafe { SendInput(inputs, std::mem::size_of::<INPUT>() as i32) };
    if sent as usize == inputs.len() {
        Ok(())
    } else {
        let err = std::io::Error::last_os_error();
        Err(format!("SendInput sent {sent} of {}: {err}", inputs.len()))
    }
}
