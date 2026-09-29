//! Key tables for injection: the tag on our own input, keys Windows sends as extended, and how
//! `VkKeyScanEx` names the modifiers a character needs.

use std::fmt;

use kx_module_api::REDACTED_MARKER;

/// Marks input we inject (`dwExtraInfo`, "KXSP"), so our own hooks can skip it.
pub const TAG: usize = 0x4B58_5350;

/// Prefix bytes `MapVirtualKeyExW` puts on extended scan codes (`0xE0`, `0xE1`).
const EXTENDED_PREFIXES: [u8; 2] = [0xE0, 0xE1];

/// Virtual keys Windows documents as extended, though `MapVirtualKeyExW` gives their numpad scan code:
/// Page Up, Page Down, End, Home, the four arrows, Insert and Delete.
pub const EXTENDED_VKS: [u16; 10] = [0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x2D, 0x2E];

/// `VK_SHIFT`, held for a character on the Shift level.
pub const VK_SHIFT: u16 = 0x10;
/// `VK_CONTROL` and `VK_MENU` (Alt), held together for a character on the `AltGr` level.
pub const ALTGR_VKS: [u16; 2] = [0x11, 0x12];

/// `VkKeyScanEx` shift-state bit for Shift.
const SHIFT_BIT: u8 = 0x01;
/// `VkKeyScanEx` shift-state bit for Ctrl.
const CTRL_BIT: u8 = 0x02;
/// `VkKeyScanEx` shift-state bit for Alt.
const ALT_BIT: u8 = 0x04;
/// The key byte `VkKeyScanEx` gives when no key types a character.
const NO_KEY: u8 = 0xFF;

/// Line feed: the Keys route types it as Enter, since `VkKeyScanEx` maps it to Ctrl+Enter.
pub const LINE_FEED: char = '\n';
/// Carriage return: the character Enter types.
pub const ENTER: char = '\r';

/// The key that types a character, as `VkKeyScanEx` names it.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct KeyFor {
    /// The virtual key.
    pub vk: u16,
    /// Shift is held.
    pub shift: bool,
    /// Ctrl and Alt are held, as `AltGr`.
    pub altgr: bool,
}

/// Shows no key, so a log never holds what was typed.
impl fmt::Debug for KeyFor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "KeyFor({REDACTED_MARKER})")
    }
}

/// Reads a `VkKeyScanEx` answer; `None` when no key types the character or only a shortcut does
/// (Ctrl or Alt alone, or a state bit we cannot press).
#[must_use]
pub fn key_for(raw: i16) -> Option<KeyFor> {
    let [vk, mods] = raw.to_le_bytes();
    let (ctrl, alt) = (mods & CTRL_BIT != 0, mods & ALT_BIT != 0);
    let pressable = mods & !(SHIFT_BIT | CTRL_BIT | ALT_BIT) == 0;
    (vk != NO_KEY && pressable && ctrl == alt).then_some(KeyFor {
        vk: u16::from(vk),
        shift: mods & SHIFT_BIT != 0,
        altgr: ctrl,
    })
}

/// True when virtual key `vk` goes as an extended key; `mapped` is its scan code from `MapVirtualKeyExW`.
#[must_use]
pub fn vk_extended(vk: u16, mapped: u32) -> bool {
    let prefix = mapped.to_le_bytes()[1];
    EXTENDED_PREFIXES.contains(&prefix) || EXTENDED_VKS.contains(&vk)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `VK_A`, and `VK_0`, which types `@` with `AltGr` on French AZERTY.
    const VK_A: u16 = 0x41;
    const VK_0: u16 = 0x30;

    fn answer(mods: u8, vk: u16) -> i16 {
        i16::from_le_bytes([u8::try_from(vk).unwrap(), mods])
    }

    #[test]
    fn debug_output_never_shows_the_key() {
        let shown = format!("{:?}", key_for(answer(SHIFT_BIT, VK_A)));
        assert!(!shown.chars().any(|c| c.is_ascii_digit()), "{shown}");
        assert!(shown.contains(REDACTED_MARKER), "{shown}");
    }

    #[test]
    fn the_tag_spells_kxsp() {
        assert_eq!(&TAG.to_be_bytes()[size_of::<usize>() - 4..], b"KXSP");
    }

    #[test]
    fn plain_shift_and_altgr_answers_name_their_key() {
        let plain = KeyFor {
            vk: VK_A,
            shift: false,
            altgr: false,
        };
        assert_eq!(key_for(answer(0, VK_A)), Some(plain));
        let shift = KeyFor {
            shift: true,
            ..plain
        };
        assert_eq!(key_for(answer(SHIFT_BIT, VK_A)), Some(shift));
        let altgr = KeyFor {
            vk: VK_0,
            shift: false,
            altgr: true,
        };
        assert_eq!(key_for(answer(CTRL_BIT | ALT_BIT, VK_0)), Some(altgr));
        let both = KeyFor {
            shift: true,
            ..altgr
        };
        assert_eq!(
            key_for(answer(SHIFT_BIT | CTRL_BIT | ALT_BIT, VK_0)),
            Some(both)
        );
    }

    #[test]
    fn no_key_and_shortcut_only_answers_are_refused() {
        assert_eq!(key_for(-1), None, "no key");
        assert_eq!(key_for(answer(0, u16::from(NO_KEY))), None, "no key byte");
        for mods in [CTRL_BIT, ALT_BIT, SHIFT_BIT | CTRL_BIT, SHIFT_BIT | ALT_BIT] {
            assert_eq!(key_for(answer(mods, VK_A)), None, "shortcut {mods:#04X}");
        }
        for bit in [0x08, 0x10, 0x20, 0x40, 0x80] {
            assert_eq!(key_for(answer(bit, VK_A)), None, "state bit {bit:#04X}");
        }
    }

    #[test]
    fn navigation_keys_are_extended_so_numlock_cannot_turn_them_into_digits() {
        // Their numpad scan codes, as `MapVirtualKeyExW` gives them without a prefix.
        let numpad = [0x49, 0x51, 0x4F, 0x47, 0x4B, 0x48, 0x4D, 0x50, 0x52, 0x53];
        for (vk, mapped) in EXTENDED_VKS.into_iter().zip(numpad) {
            assert!(vk_extended(vk, mapped), "{vk:#04X}");
        }
    }

    #[test]
    fn prefixed_scan_codes_are_extended_and_plain_ones_are_not() {
        assert!(vk_extended(0xA5, 0xE038), "right Alt");
        assert!(vk_extended(0x13, 0xE11D), "Pause");
        for (vk, mapped) in [(VK_A, 0x1E), (VK_SHIFT, 0x2A), (0x11, 0x1D), (0x12, 0x38)] {
            assert!(!vk_extended(vk, mapped), "{vk:#04X}");
        }
    }

    #[cfg(windows)]
    #[test]
    fn the_tables_match_windows() {
        use windows::Win32::UI::Input::KeyboardAndMouse::{
            VK_CONTROL, VK_DELETE, VK_DOWN, VK_END, VK_HOME, VK_INSERT, VK_LEFT, VK_MENU, VK_NEXT,
            VK_PRIOR, VK_RIGHT, VK_SHIFT as SHIFT, VK_UP,
        };
        let nav = [
            VK_PRIOR, VK_NEXT, VK_END, VK_HOME, VK_LEFT, VK_UP, VK_RIGHT, VK_DOWN, VK_INSERT,
            VK_DELETE,
        ];
        assert_eq!(EXTENDED_VKS, nav.map(|vk| vk.0));
        assert_eq!(VK_SHIFT, SHIFT.0);
        assert_eq!(ALTGR_VKS, [VK_CONTROL.0, VK_MENU.0]);
    }
}
