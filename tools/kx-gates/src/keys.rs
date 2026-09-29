//! Keys, text and raw input for the gates, sent through the product injector and tagged.
#![cfg(windows)]

use std::io;

use kx_module_api::Redacted;
use kx_module_api::ports::{InputInjector, TextRoute};
use kx_platform_windows::WinInjector;
use kx_platform_windows::inject::{self, send_keys, vk_key};
use kx_platform_windows::strokes::{self, KeyEvent};
use windows::Win32::UI::Input::KeyboardAndMouse::INPUT;

use crate::win::sleep_ms;

/// Sends mouse or key `inputs` in one batch; the error adds Windows' reason.
pub fn send(inputs: &[INPUT]) -> Result<(), String> {
    inject::send(inputs).map_err(|e| format!("{e}: {}", io::Error::last_os_error()))
}

/// Types `text` as Unicode characters in one batch, whatever the target's layout.
pub fn text(text: &str) -> Result<(), String> {
    let text = Redacted::new(text.to_owned());
    WinInjector
        .text(&text, TextRoute::Unicode)
        .map_err(|e| e.to_string())
}

/// Virtual key `vk` pressed or released (`up`), marked `tag`.
pub fn tagged(vk: u16, up: bool, tag: usize) -> INPUT {
    inject::input(vk_key(vk).event(up), tag)
}

/// The presses of `vks` in order, then the releases in reverse.
fn presses(vks: &[u16]) -> Vec<KeyEvent> {
    let keys: Vec<_> = vks.iter().map(|&vk| vk_key(vk)).collect();
    strokes::combo(&keys)
}

/// Presses `vks` in order, then releases them in reverse; after a failed send it tries to let go of what went down.
pub fn combo(vks: &[u16]) -> Result<(), String> {
    send_keys(&presses(vks)).map_err(|e| e.to_string())
}

/// Like `combo`, with a pause of `gap_ms` after each press and release, for shell shortcuts.
pub fn slow_combo(vks: &[u16], gap_ms: u64) -> Result<(), String> {
    let events = presses(vks);
    for (at, event) in events.iter().enumerate() {
        if let Err(e) = send_keys(std::slice::from_ref(event)) {
            let _ = send_keys(&strokes::releases(&events[..at]));
            return Err(e.to_string());
        }
        sleep_ms(gap_ms);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use kx_platform_windows::keys::TAG;
    use windows::Win32::UI::Input::KeyboardAndMouse::{KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP};

    #[allow(unsafe_code, reason = "Reads the keyboard member of a key input.")]
    fn flags(vk: u16, up: bool) -> (bool, bool, usize) {
        // SAFETY: reading the keyboard member of an input built as a keyboard input.
        let ki = unsafe { tagged(vk, up, TAG).Anonymous.ki };
        let (f, tag) = (ki.dwFlags, ki.dwExtraInfo);
        (
            f.contains(KEYEVENTF_EXTENDEDKEY),
            f.contains(KEYEVENTF_KEYUP),
            tag,
        )
    }

    #[test]
    fn navigation_keys_are_sent_as_extended_so_numlock_cannot_turn_them_into_digits() {
        for vk in [0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x2D, 0x2E, 0x21, 0x22] {
            assert_eq!(flags(vk, true), (true, true, TAG), "{vk:#04X}");
        }
    }

    #[test]
    fn letters_and_left_modifiers_are_not_extended() {
        for vk in [0x41, 0x10, 0x11, 0x12] {
            assert_eq!(flags(vk, false), (false, false, TAG), "{vk:#04X}");
        }
    }

    #[test]
    fn a_combo_presses_in_order_and_releases_in_reverse() {
        let got: Vec<_> = presses(&[0x11, 0x43]).iter().map(|e| e.up).collect();
        assert_eq!(got, [false, false, true, true]);
        let keys: Vec<_> = presses(&[0x11, 0x43]).iter().map(|e| e.key).collect();
        assert_eq!(
            keys,
            [vk_key(0x11), vk_key(0x43), vk_key(0x43), vk_key(0x11)]
        );
    }
}
