//! Sends keys to the foreground app with `SendInput`.

use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_KEYBOARD, KEYBD_EVENT_FLAGS, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY,
    KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, KEYEVENTF_UNICODE, SendInput, VIRTUAL_KEY,
};

use crate::{is_extended, scan_byte};

/// Marks input we injected, so our own hooks can skip it.
pub const TAG: usize = 0x4B58_5350;

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
    if is_extended(code) {
        flags |= KEYEVENTF_EXTENDEDKEY;
    }
    (scan_byte(code) as u16, flags)
}

/// Presses `code` down, or releases it when `up`.
pub fn press(code: u32, up: bool) -> Result<(), String> {
    let (scan, mut flags) = scan_flags(code);
    if up {
        flags |= KEYEVENTF_KEYUP;
    }
    send(&[key(scan, flags)])
}

/// Taps `code` the ADR-0009 way: its character in the target's layout as Unicode, else the key itself.
pub fn tap(code: u32) -> Result<(), String> {
    let character = crate::layout::character(code, crate::layout::foreground_layout());
    if crate::layout::is_printable(&character) {
        text(&character)
    } else {
        tap_scan(code)
    }
}

/// Taps `code` as a scan code (down then up); the target app's layout decides the result.
pub fn tap_scan(code: u32) -> Result<(), String> {
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

/// One virtual-key press or release, tagged as ours.
fn vk(v: u16, up: bool) -> INPUT {
    let mut i = key(
        0,
        if up {
            KEYEVENTF_KEYUP
        } else {
            KEYBD_EVENT_FLAGS(0)
        },
    );
    i.Anonymous.ki.wVk = VIRTUAL_KEY(v);
    i
}

/// Presses virtual keys `vks` in order and releases them in reverse, such as Ctrl+C.
fn combo_inputs(vks: &[u16]) -> Vec<INPUT> {
    let down = vks.iter().map(|&v| vk(v, false));
    down.chain(vks.iter().rev().map(|&v| vk(v, true))).collect()
}

/// Sends the shortcut `vks` in one batch; virtual keys work whatever the app's layout.
pub fn combo(vks: &[u16]) -> Result<(), String> {
    send(&combo_inputs(vks))
}

/// Sends a batch of inputs in one call, which other input cannot split; all or an error.
pub fn send(inputs: &[INPUT]) -> Result<(), String> {
    // SAFETY: `inputs` is a valid slice and the size argument matches `INPUT`.
    let sent = unsafe { SendInput(inputs, std::mem::size_of::<INPUT>() as i32) };
    if sent as usize == inputs.len() {
        Ok(())
    } else {
        let err = std::io::Error::last_os_error();
        Err(format!("SendInput sent {sent} of {}: {err}", inputs.len()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_combo_presses_in_order_and_releases_in_reverse() {
        let got: Vec<(u16, bool)> = combo_inputs(&[0x11, 0x43])
            .iter()
            // SAFETY: every input built here is a keyboard input.
            .map(|i| unsafe {
                (
                    i.Anonymous.ki.wVk.0,
                    i.Anonymous.ki.dwFlags.contains(KEYEVENTF_KEYUP),
                )
            })
            .collect();
        assert_eq!(
            got,
            [(0x11, false), (0x43, false), (0x43, true), (0x11, true)]
        );
    }
}
