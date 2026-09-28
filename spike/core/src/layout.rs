//! Key labels read from the installed keyboard layouts.

use serde::Serialize;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyNameTextW, GetKeyState, GetKeyboardLayout, GetKeyboardLayoutList, HKL,
    MAPVK_VSC_TO_VK_EX, MapVirtualKeyExW, ToUnicodeEx, VIRTUAL_KEY, VK_CAPITAL, VK_CONTROL,
    VK_LCONTROL, VK_MENU, VK_RMENU, VK_SHIFT,
};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

use crate::config::KeyboardConfig;
use crate::legend::{KeyChars, Slot};
use crate::place::{Place, places};
use crate::{is_extended, scan_byte};

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
/// Width of a key that `widths` in spike.toml does not list, in key units.
pub const DEFAULT_WIDTH: f32 = 1.0;
/// Size of the key-state array that `ToUnicodeEx` reads: one byte per virtual key.
const KEY_STATES: usize = 256;
/// Key-state bit for a key held down.
const DOWN: u8 = 0x80;
/// `GetKeyState` bit for a toggle key that is on.
const TOGGLED: i16 = 1;
/// Key-state bit for a toggle key that is on.
const ON: u8 = 0x01;
/// Keys held for Shift.
const SHIFT: [VIRTUAL_KEY; 1] = [VK_SHIFT];
/// Keys held for AltGr: Windows reads it as Ctrl with right Alt.
const ALTGR: [VIRTUAL_KEY; 4] = [VK_CONTROL, VK_LCONTROL, VK_MENU, VK_RMENU];

/// One key: its scan code, label and box.
#[derive(Debug, Clone, Serialize)]
pub struct KeyCap {
    /// Scan code; `0xE0xx` marks an extended key.
    pub code: u32,
    /// Text shown on the key.
    pub label: String,
    /// Where the key sits; JSON gets its fields as `x`, `y`, `w`, `h`.
    #[serde(flatten)]
    pub place: Place,
}

/// Bits of a layout handle that hold its language id.
const LANG_MASK: isize = 0xFFFF;

/// The language id (LANGID) of layout handle `hkl`: its low word.
pub fn lang_id(hkl: isize) -> u16 {
    (hkl & LANG_MASK) as u16
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
    if is_printable(&printed) {
        printed
    } else {
        key_name(code)
    }
}

/// True if `text` has a visible character (not only controls or spaces).
pub fn is_printable(text: &str) -> bool {
    text.chars().any(|c| !c.is_control() && !c.is_whitespace())
}

/// What `code` types in `hkl` with no modifiers; empty for non-printing keys.
pub fn character(code: u32, hkl: HKL) -> String {
    typed(code, hkl, &[], false).0
}

/// What `code` types in `hkl` in each modifier state, and which states are dead keys; control characters count as nothing.
pub fn key_chars(code: u32, hkl: HKL) -> KeyChars {
    let mut dead = Vec::new();
    let mut read = |slot: Slot, held: &[VIRTUAL_KEY], caps: bool| {
        let (text, is_dead) = typed(code, hkl, held, caps);
        if is_dead {
            dead.push(slot);
        }
        text
    };
    KeyChars {
        base: read(Slot::Base, &[], false),
        shift: read(Slot::Shift, &SHIFT, false),
        altgr: read(Slot::AltGr, &ALTGR, false),
        caps: read(Slot::Caps, &[], true),
        caps_shift: read(Slot::CapsShift, &SHIFT, true),
        dead,
    }
}

/// True while Caps Lock is on.
pub fn caps_on() -> bool {
    // SAFETY: plain query of a toggle key's state.
    unsafe { GetKeyState(i32::from(VK_CAPITAL.0)) & TOGGLED != 0 }
}

/// True when `code` types anything in `hkl`, even a control such as Enter's; only such a key ends a dead key's wait.
pub fn types_any(code: u32, hkl: HKL) -> bool {
    to_unicode(code, hkl, &[], false).0 != 0
}

/// What `code` types in `hkl` while `held` keys are down and Caps Lock is `caps`, and whether it is a dead key.
fn typed(code: u32, hkl: HKL, held: &[VIRTUAL_KEY], caps: bool) -> (String, bool) {
    let (n, buf) = to_unicode(code, hkl, held, caps);
    read_typed(n, &buf)
}

/// Windows' raw answer for `code`, as `typed` asks it: a count (negative for a dead key) and the text.
fn to_unicode(code: u32, hkl: HKL, held: &[VIRTUAL_KEY], caps: bool) -> (i32, [u16; LABEL_CAP]) {
    let mut state = [0u8; KEY_STATES];
    for vk in held {
        state[usize::from(vk.0)] = DOWN;
    }
    if caps {
        state[usize::from(VK_CAPITAL.0)] = ON;
    }
    let mut buf = [0u16; LABEL_CAP];
    // SAFETY: all buffers are valid for their lengths; the flag keeps dead-key state untouched.
    // The scan code goes in without its 0xE0 prefix, whose top bit would mean "key up".
    let n = unsafe {
        let vk = MapVirtualKeyExW(code, MAPVK_VSC_TO_VK_EX, Some(hkl));
        ToUnicodeEx(
            vk,
            scan_byte(code),
            &state,
            &mut buf,
            NO_STATE_CHANGE,
            Some(hkl),
        )
    };
    (n, buf)
}

/// A `ToUnicodeEx` answer `n` over `buf`: the visible text (empty for controls) and whether the key is dead (`n < 0`).
fn read_typed(n: i32, buf: &[u16]) -> (String, bool) {
    let len = usize::try_from(n.unsigned_abs())
        .unwrap_or(0)
        .min(buf.len());
    let text = String::from_utf16_lossy(&buf[..len]);
    let shown = if is_printable(&text) {
        text
    } else {
        String::new()
    };
    (shown, n < 0)
}

fn key_name(code: u32) -> String {
    let mut arg = (scan_byte(code) << NAME_SCAN_SHIFT) as i32;
    if is_extended(code) {
        arg |= NAME_EXTENDED_BIT;
    }
    let mut buf = [0u16; LABEL_CAP];
    // SAFETY: the buffer is valid for its whole length.
    let n = unsafe { GetKeyNameTextW(arg, &mut buf) };
    String::from_utf16_lossy(&buf[..usize::try_from(n).unwrap_or(0)])
}

/// The configured rows with labels from `hkl` and each key's box.
pub fn rows(cfg: &KeyboardConfig, hkl: HKL) -> Vec<Vec<KeyCap>> {
    cfg.rows
        .iter()
        .zip(places(cfg))
        .map(|(row, boxes)| {
            row.iter()
                .zip(boxes)
                .map(|(&code, place)| KeyCap {
                    code,
                    label: label(code, hkl),
                    place,
                })
                .collect()
        })
        .collect()
}

/// Remembers which layout the labels show, to relabel only when the app in front changes it.
#[derive(Debug, Default)]
pub struct Follow {
    shown: isize,
}

impl Follow {
    /// Starts from the labels of `hkl`.
    pub fn new(hkl: HKL) -> Self {
        Self {
            shown: hkl.0 as isize,
        }
    }

    /// The new layout when another app is in front and its layout differs; else `None`.
    pub fn changed(&mut self) -> Option<HKL> {
        let front = crate::window::foreground();
        let now = foreground_layout();
        let other = !front.is_invalid() && !crate::window::is_ours(front);
        self.step(other, now.0 as isize).then_some(now)
    }

    fn step(&mut self, other_app_in_front: bool, now: isize) -> bool {
        let new = other_app_in_front && now != self.shown;
        if new {
            self.shown = now;
        }
        new
    }
}

/// The configured rows as JSON, for faces that build keys in QML or HTML.
pub fn rows_json(cfg: &KeyboardConfig, hkl: HKL) -> String {
    serde_json::to_string(&rows(cfg, hkl)).unwrap_or_default()
}

/// Width of `code` in key units, from `widths` in spike.toml.
pub fn width(cfg: &KeyboardConfig, code: u32) -> f32 {
    cfg.widths
        .iter()
        .find(|(c, _)| *c == code)
        .map_or(DEFAULT_WIDTH, |(_, w)| *w)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_language_id_is_the_low_word_of_a_layout() {
        assert_eq!(lang_id(0x0409_0409), 0x0409);
        assert_eq!(lang_id(0xF0A8_1C01_u32 as i32 as isize), 0x1C01);
    }

    #[test]
    fn only_keys_that_type_something_end_a_dead_key_wait() {
        // Scan codes: Space, Enter, Tab; Left arrow, Delete (extended).
        let hkl = foreground_layout();
        for code in [0x39, 0x1C, 0x0F] {
            assert!(types_any(code, hkl), "{code:#x} types");
        }
        for code in [0xE04B, 0xE053] {
            assert!(!types_any(code, hkl), "{code:#x} types nothing");
        }
    }

    #[test]
    fn printable_means_a_visible_character() {
        for (text, want) in [
            ("\r", false),
            (" ", false),
            ("\u{1b}", false),
            ("", false),
            ("a", true),
            ("ض", true),
            ("\u{64e}", true),
        ] {
            assert_eq!(is_printable(text), want, "{text:?}");
        }
    }

    #[test]
    fn a_windows_answer_gives_the_text_and_whether_the_key_is_dead() {
        let buf: Vec<u16> = "^x".encode_utf16().collect();
        assert_eq!(read_typed(-1, &buf), ("^".to_string(), true), "a dead key");
        assert_eq!(read_typed(2, &buf), ("^x".to_string(), false));
        assert_eq!(read_typed(0, &buf), (String::new(), false), "types nothing");
        let bs: Vec<u16> = "\u{8}".encode_utf16().collect();
        assert_eq!(
            read_typed(1, &bs),
            (String::new(), false),
            "a control character"
        );
        assert_eq!(read_typed(9, &bs).0, "", "a count past the buffer");
    }

    #[test]
    fn follow_relabels_only_for_another_app_with_a_new_layout() {
        let mut f = Follow { shown: 1 };
        assert!(!f.step(true, 1));
        assert!(!f.step(false, 2), "our own window or none in front");
        assert!(f.step(true, 2));
        assert!(!f.step(true, 2), "already showing it");
    }

    #[test]
    fn width_uses_the_table_else_the_default() {
        let cfg = KeyboardConfig {
            titles: Default::default(),
            key_px: 48.0,
            gap_px: 4.0,
            font_px: 16.0,
            guard_delay_ms: 0,
            guard_tries: 1,
            relabel_ms: 0,
            rows: vec![],
            widths: vec![(0x0F, 1.5)],
        };
        assert_eq!(width(&cfg, 0x0F), 1.5);
        assert_eq!(width(&cfg, 0x10), DEFAULT_WIDTH);
    }
}
