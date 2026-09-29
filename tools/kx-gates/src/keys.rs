//! Virtual-key shortcuts, sent with `SendInput` and tagged as ours.
#![cfg(windows)]

use spike_core::inject::send;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_KEYBOARD, KEYBD_EVENT_FLAGS, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY,
    KEYEVENTF_KEYUP, MAPVK_VK_TO_VSC_EX, MapVirtualKeyW, VIRTUAL_KEY, VK_DELETE, VK_DOWN, VK_END,
    VK_HOME, VK_INSERT, VK_LEFT, VK_NEXT, VK_PRIOR, VK_RIGHT, VK_UP,
};

use crate::win::sleep_ms;

/// Prefix bytes `MapVirtualKeyW` puts on extended scan codes (0xE0, 0xE1).
const EXTENDED_PREFIXES: [u32; 2] = [0xE0, 0xE1];
/// Bits of the scan-code prefix byte.
const PREFIX_SHIFT: u32 = 8;
/// Keys Windows documents as extended, though `MapVirtualKeyW` gives their numpad scan code:
/// Page Up, Page Down, End, Home, the arrows, Insert and Delete (adapter table).
const EXTENDED_VKS: [VIRTUAL_KEY; 10] = [
    VK_PRIOR, VK_NEXT, VK_END, VK_HOME, VK_LEFT, VK_UP, VK_RIGHT, VK_DOWN, VK_INSERT, VK_DELETE,
];

/// Virtual key `vk` pressed or released (`up`), marked `tag`.
#[allow(unsafe_code, reason = "Plain table lookup.")]
pub fn tagged(vk: u16, up: bool, tag: usize) -> INPUT {
    // SAFETY: plain table lookup.
    let scan = unsafe { MapVirtualKeyW(u32::from(vk), MAPVK_VK_TO_VSC_EX) };
    let mut flags = KEYBD_EVENT_FLAGS(0);
    if EXTENDED_PREFIXES.contains(&(scan >> PREFIX_SHIFT))
        || EXTENDED_VKS.contains(&VIRTUAL_KEY(vk))
    {
        flags |= KEYEVENTF_EXTENDEDKEY;
    }
    if up {
        flags |= KEYEVENTF_KEYUP;
    }
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(vk),
                wScan: u16::try_from(spike_core::scan_byte(scan)).unwrap_or_default(),
                dwFlags: flags,
                time: 0,
                dwExtraInfo: tag,
            },
        },
    }
}

/// The presses of `vks` in order, then the releases in reverse.
fn presses(vks: &[u16]) -> Vec<INPUT> {
    let tag = spike_core::inject::TAG;
    let downs = vks.iter().map(|&vk| tagged(vk, false, tag));
    let ups = vks.iter().rev().map(|&vk| tagged(vk, true, tag));
    downs.chain(ups).collect()
}

/// Releases every key of `vks`, so a failed send leaves nothing held down.
fn let_go(vks: &[u16]) {
    let tag = spike_core::inject::TAG;
    let ups: Vec<INPUT> = vks.iter().rev().map(|&vk| tagged(vk, true, tag)).collect();
    let _ = send(&ups);
}

/// Presses `vks` in order, then releases them in reverse.
pub fn combo(vks: &[u16]) -> Result<(), String> {
    send(&presses(vks)).inspect_err(|_| let_go(vks))
}

/// Like `combo`, with a pause of `gap_ms` after each press and release, for shell shortcuts.
pub fn slow_combo(vks: &[u16], gap_ms: u64) -> Result<(), String> {
    for input in presses(vks) {
        send(&[input]).inspect_err(|_| let_go(vks))?;
        sleep_ms(gap_ms);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(unsafe_code, reason = "Reads the keyboard member of a key input.")]
    fn extended(vk: u16) -> bool {
        // SAFETY: reading the keyboard member of an input we built as a keyboard input.
        let ki = unsafe { tagged(vk, false, 0).Anonymous.ki };
        ki.dwFlags.contains(KEYEVENTF_EXTENDEDKEY)
    }

    #[test]
    fn navigation_keys_are_sent_as_extended_so_numlock_cannot_turn_them_into_digits() {
        for vk in [0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x2D, 0x2E, 0x21, 0x22] {
            assert!(extended(vk), "{vk:#04X}");
        }
    }

    #[test]
    fn letters_and_left_modifiers_are_not_extended() {
        for vk in [0x41, 0x10, 0x11, 0x12] {
            assert!(!extended(vk), "{vk:#04X}");
        }
    }
}
