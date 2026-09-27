//! Virtual-key shortcuts, sent with `SendInput` and tagged as ours.

use spike_core::inject::send;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_KEYBOARD, KEYBD_EVENT_FLAGS, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY,
    KEYEVENTF_KEYUP, MAPVK_VK_TO_VSC_EX, MapVirtualKeyW, VIRTUAL_KEY,
};

/// Prefix bytes `MapVirtualKeyW` puts on extended scan codes (0xE0, 0xE1).
const EXTENDED_PREFIXES: [u32; 2] = [0xE0, 0xE1];
/// Bits of the scan-code prefix byte.
const PREFIX_SHIFT: u32 = 8;

fn key(vk: u16, up: bool) -> INPUT {
    // SAFETY: plain table lookup.
    let scan = unsafe { MapVirtualKeyW(u32::from(vk), MAPVK_VK_TO_VSC_EX) };
    let mut flags = KEYBD_EVENT_FLAGS(0);
    if EXTENDED_PREFIXES.contains(&(scan >> PREFIX_SHIFT)) {
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
                wScan: spike_core::scan_byte(scan) as u16,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: spike_core::inject::TAG,
            },
        },
    }
}

/// Presses `vks` in order, then releases them in reverse.
pub fn combo(vks: &[u16]) -> Result<(), String> {
    let downs = vks.iter().map(|&vk| key(vk, false));
    let ups = vks.iter().rev().map(|&vk| key(vk, true));
    send(&downs.chain(ups).collect::<Vec<_>>())
}
