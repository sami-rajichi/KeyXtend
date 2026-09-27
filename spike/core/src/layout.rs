//! Key labels read from the installed keyboard layouts.

use serde::Serialize;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyNameTextW, GetKeyboardLayout, GetKeyboardLayoutList, HKL, MAPVK_VSC_TO_VK_EX,
    MapVirtualKeyExW, ToUnicodeEx,
};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

use crate::EXTENDED;
use crate::config::KeyboardConfig;

/// `ToUnicodeEx` flag: leave the keyboard state unchanged (Windows 10 1607+).
const NO_STATE_CHANGE: u32 = 0x4;
/// Bit position of the scan code in a `GetKeyNameTextW` argument.
const NAME_SCAN_SHIFT: u32 = 16;
/// Extended-key bit in a `GetKeyNameTextW` argument.
const NAME_EXTENDED_BIT: i32 = 1 << 24;
/// Longest label read from Windows.
const LABEL_CAP: usize = 32;
/// Most layouts we list.
const LAYOUT_CAP: usize = 32;

/// One key: its scan code, label and width in key units.
#[derive(Debug, Clone, Serialize)]
pub struct KeyCap {
    /// Scan code; `0xE0xx` marks an extended key.
    pub code: u32,
    /// Text shown on the key.
    pub label: String,
    /// Width in key units.
    pub width: f32,
}

/// The layout of the app in front, which is where our keys go.
pub fn foreground_layout() -> HKL {
    // SAFETY: plain queries; a null window gives thread 0, which means the calling thread.
    unsafe {
        let thread = GetWindowThreadProcessId(GetForegroundWindow(), None);
        GetKeyboardLayout(thread)
    }
}

/// Every layout installed for this user.
pub fn installed_layouts() -> Vec<HKL> {
    let mut list = vec![HKL::default(); LAYOUT_CAP];
    // SAFETY: the buffer is valid for its whole length.
    let n = unsafe { GetKeyboardLayoutList(Some(&mut list)) };
    list.truncate(usize::try_from(n).unwrap_or(0));
    list
}

/// The label for `code` in `hkl`: its character, or its key name for non-printing keys.
pub fn label(code: u32, hkl: HKL) -> String {
    let printed = character(code, hkl);
    if printed.chars().any(|c| !c.is_control() && !c.is_whitespace()) {
        printed
    } else {
        key_name(code)
    }
}

fn character(code: u32, hkl: HKL) -> String {
    let state = [0u8; 256];
    let mut buf = [0u16; LABEL_CAP];
    // SAFETY: all buffers are valid for their lengths; the flag keeps dead-key state untouched.
    let n = unsafe {
        let vk = MapVirtualKeyExW(code, MAPVK_VSC_TO_VK_EX, Some(hkl));
        ToUnicodeEx(vk, code, &state, &mut buf, NO_STATE_CHANGE, Some(hkl))
    };
    let len = usize::try_from(n.unsigned_abs()).unwrap_or(0).min(LABEL_CAP);
    String::from_utf16_lossy(&buf[..len])
}

fn key_name(code: u32) -> String {
    let mut arg = ((code & 0xFF) << NAME_SCAN_SHIFT) as i32;
    if code & 0xFF00 == EXTENDED {
        arg |= NAME_EXTENDED_BIT;
    }
    let mut buf = [0u16; LABEL_CAP];
    // SAFETY: the buffer is valid for its whole length.
    let n = unsafe { GetKeyNameTextW(arg, &mut buf) };
    String::from_utf16_lossy(&buf[..usize::try_from(n).unwrap_or(0)])
}

/// The configured rows with labels from `hkl`.
pub fn rows(cfg: &KeyboardConfig, hkl: HKL) -> Vec<Vec<KeyCap>> {
    cfg.rows
        .iter()
        .map(|row| {
            row.iter()
                .map(|&code| KeyCap {
                    code,
                    label: label(code, hkl),
                    width: width(cfg, code),
                })
                .collect()
        })
        .collect()
}

/// The configured rows as JSON, for faces that build keys in QML or HTML.
pub fn rows_json(cfg: &KeyboardConfig, hkl: HKL) -> String {
    serde_json::to_string(&rows(cfg, hkl)).unwrap_or_default()
}

fn width(cfg: &KeyboardConfig, code: u32) -> f32 {
    cfg.widths
        .iter()
        .find(|(c, _)| *c == code)
        .map_or(1.0, |(_, w)| *w)
}
