//! Keys, text and raw input for the gates, sent through the product injector and tagged.
#![cfg(windows)]

use std::io;

use kx_module_api::Redacted;
use kx_module_api::ports::{InputInjector, TextRoute};
use kx_platform_windows::WinInjector;
use kx_platform_windows::inject::{self, send_keys, vk_key};
use kx_platform_windows::strokes::{self, KeyEvent};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_KEYBOARD, INPUT_MOUSE, KEYEVENTF_KEYUP, MOUSEEVENTF_HWHEEL, MOUSEEVENTF_LEFTDOWN,
    MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_RIGHTDOWN,
    MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_WHEEL, MOUSEEVENTF_XDOWN, MOUSEEVENTF_XUP,
};

use crate::guardrun;
use crate::win::sleep_ms;

/// Mouse flags that press a button or turn a wheel.
const PRESSES: u32 = MOUSEEVENTF_LEFTDOWN.0
    | MOUSEEVENTF_RIGHTDOWN.0
    | MOUSEEVENTF_MIDDLEDOWN.0
    | MOUSEEVENTF_XDOWN.0
    | MOUSEEVENTF_WHEEL.0
    | MOUSEEVENTF_HWHEEL.0;
/// Mouse flags that let a button go.
const RELEASES: u32 =
    MOUSEEVENTF_LEFTUP.0 | MOUSEEVENTF_RIGHTUP.0 | MOUSEEVENTF_MIDDLEUP.0 | MOUSEEVENTF_XUP.0;

/// Whether `input` presses something and whether it lets something go; anything but a key or mouse input presses.
#[allow(unsafe_code, reason = "Reads the union member the input's type names.")]
fn press_release(input: &INPUT) -> (bool, bool) {
    if input.r#type == INPUT_KEYBOARD {
        // SAFETY: the type says the keyboard member is set.
        let up = unsafe { input.Anonymous.ki }
            .dwFlags
            .contains(KEYEVENTF_KEYUP);
        return (!up, up);
    }
    if input.r#type == INPUT_MOUSE {
        // SAFETY: the type says the mouse member is set.
        let flags = unsafe { input.Anonymous.mi }.dwFlags.0;
        return (flags & PRESSES != 0, flags & RELEASES != 0);
    }
    (true, false)
}

/// True when `inputs` let something go and press nothing, so they still go out after the mouse guard stopped a gate.
fn only_lets_go(inputs: &[INPUT]) -> bool {
    let each: Vec<_> = inputs.iter().map(press_release).collect();
    each.iter().any(|&(_, up)| up) && !each.iter().any(|&(down, _)| down)
}

/// Sends mouse or key `inputs` in one batch; the error adds Windows' reason. Once the mouse guard stopped the gate, only
/// a batch that lets go goes out, so nothing stays pressed.
pub fn send(inputs: &[INPUT]) -> Result<(), String> {
    if !only_lets_go(inputs) {
        guardrun::allowed()?;
    }
    inject::send(inputs).map_err(|e| format!("{e}: {}", io::Error::last_os_error()))
}

/// Types `text` as Unicode characters in one batch, whatever the target's layout.
pub fn text(text: &str) -> Result<(), String> {
    guardrun::allowed()?;
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
    guardrun::allowed()?;
    send_keys(&presses(vks)).map_err(|e| e.to_string())
}

/// Like `combo`, with a pause of `gap_ms` after each press and release, for shell shortcuts.
pub fn slow_combo(vks: &[u16], gap_ms: u64) -> Result<(), String> {
    guardrun::allowed()?;
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
    fn only_a_batch_that_lets_go_and_presses_nothing_passes_a_stopped_guard() {
        use crate::mouse::input;
        use windows::Win32::UI::Input::KeyboardAndMouse::{
            MOUSE_EVENT_FLAGS, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
            MOUSEEVENTF_MOVE, MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_WHEEL,
        };
        let at = |flags: MOUSE_EVENT_FLAGS| {
            input(5, 6, MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | flags, TAG)
        };
        let (moved, down, up) = (
            at(MOUSE_EVENT_FLAGS(0)),
            at(MOUSEEVENTF_LEFTDOWN),
            at(MOUSEEVENTF_LEFTUP),
        );
        assert!(only_lets_go(&[moved, up]), "a release where it moved");
        assert!(only_lets_go(&[
            tagged(0x10, true, TAG),
            at(MOUSEEVENTF_RIGHTUP)
        ]));
        assert!(!only_lets_go(&[moved, down, up]), "a click");
        assert!(!only_lets_go(&[moved]), "a move alone");
        assert!(
            !only_lets_go(&[input(0, 0, MOUSEEVENTF_WHEEL, TAG)]),
            "a wheel turn"
        );
        assert!(!only_lets_go(&[tagged(0x41, false, TAG)]), "a key press");
        assert!(!only_lets_go(&[]), "nothing to let go of");
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
