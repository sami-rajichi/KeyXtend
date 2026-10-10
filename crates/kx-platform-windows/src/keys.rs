//! Key tables: the tag on our own input, keys Windows sends or reports as extended, how `VkKeyScanEx` names
//! the modifiers a character needs, the keys `ToUnicodeEx` reads held down, and the menu mask key.

use std::fmt;

use kx_module_api::REDACTED_MARKER;

/// Marks input we inject (`dwExtraInfo`, "KXSP"), so our own hooks can skip it.
pub const TAG: usize = 0x4B58_5350;

/// Prefix bytes `MapVirtualKeyExW` puts on extended scan codes (`0xE0`, `0xE1`).
const EXTENDED_PREFIXES: [u8; 2] = [0xE0, 0xE1];

/// Virtual keys Windows documents as extended, though `MapVirtualKeyExW` gives their numpad scan code:
/// Page Up, Page Down, End, Home, the four arrows, Insert and Delete.
pub const EXTENDED_VKS: [u16; 10] = [0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x2D, 0x2E];
/// Scan codes sent plain that Windows reports as extended in key messages: layouts mark right Shift `KBDEXT`.
pub const REPORTED_EXTENDED: [u16; 1] = [0x36];

/// `VK_SHIFT`, held for a character on the Shift level.
pub const VK_SHIFT: u16 = 0x10;
/// `VK_CONTROL`.
const VK_CONTROL: u16 = 0x11;
/// `VK_MENU`: Alt.
const VK_MENU: u16 = 0x12;
/// `VK_CONTROL` and `VK_MENU` (Alt), held together for a character on the `AltGr` level.
pub const ALTGR_VKS: [u16; 2] = [VK_CONTROL, VK_MENU];
/// `VK_CAPITAL`: Caps Lock.
pub const VK_CAPITAL: u16 = 0x14;
/// `VK_LSHIFT`: left Shift.
const VK_LSHIFT: u16 = 0xA0;
/// `VK_RSHIFT`: right Shift.
const VK_RSHIFT: u16 = 0xA1;
/// `VK_LCONTROL`: left Ctrl.
const VK_LCONTROL: u16 = 0xA2;
/// `VK_RCONTROL`: right Ctrl.
const VK_RCONTROL: u16 = 0xA3;
/// `VK_LMENU`: left Alt.
const VK_LMENU: u16 = 0xA4;
/// `VK_RMENU`: right Alt.
const VK_RMENU: u16 = 0xA5;
/// `VK_LWIN`: left Windows key.
const VK_LWIN: u16 = 0x5B;
/// `VK_RWIN`: right Windows key.
const VK_RWIN: u16 = 0x5C;
/// `VK_SPACE`, pressed after a dead key so it types its own character.
pub const VK_SPACE: u16 = 0x20;
/// An unassigned virtual key tapped before letting go of Alt or Win, so neither opens the menu bar
/// or Start (the `MenuMaskKey` of `AutoHotkey`).
pub const MASK_VK: u16 = 0xE8;
/// Alt and Win as virtual keys: Alt, left and right Alt, left and right Win.
pub(crate) const ALT_WIN_VKS: [u16; 5] = [VK_MENU, VK_LMENU, VK_RMENU, VK_LWIN, VK_RWIN];
/// Alt and Win as scan codes: left and right Alt, left and right Win.
pub(crate) const ALT_WIN_SCANS: [u16; 4] = [0x38, 0xE038, 0xE05B, 0xE05C];
/// Left Ctrl and right Alt: how Windows reports `AltGr` held down.
pub const ALTGR_READ: [u16; 2] = [VK_LCONTROL, VK_RMENU];
/// Left and right Shift, Ctrl and Alt, each with the generic key `ToUnicodeEx` also reads.
const SIDED: [(u16, u16); 6] = [
    (VK_LSHIFT, VK_SHIFT),
    (VK_RSHIFT, VK_SHIFT),
    (VK_LCONTROL, VK_CONTROL),
    (VK_RCONTROL, VK_CONTROL),
    (VK_LMENU, VK_MENU),
    (VK_RMENU, VK_MENU),
];

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

/// Scan code `sent` as Windows reports it in key messages: with the `E0` prefix when it is in `REPORTED_EXTENDED`.
#[must_use]
pub fn reported(sent: u16) -> u16 {
    let [low, _] = sent.to_le_bytes();
    if REPORTED_EXTENDED.contains(&sent) {
        u16::from_be_bytes([EXTENDED_PREFIXES[0], low])
    } else {
        sent
    }
}

/// The generic key of a left or right modifier `vk`, such as `VK_SHIFT` for `VK_LSHIFT`; `None` for other keys.
#[must_use]
pub fn generic(vk: u16) -> Option<u16> {
    SIDED.iter().find(|(s, _)| *s == vk).map(|&(_, g)| g)
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

    #[test]
    fn right_shift_is_reported_extended_and_other_plain_codes_as_sent() {
        assert_eq!(reported(0x36), 0xE036, "right Shift");
        for sent in [0x2A, 0x1C, 0xE038] {
            assert_eq!(reported(sent), sent, "{sent:#06X}");
        }
    }

    #[test]
    fn sided_modifiers_add_their_generic_key_and_other_keys_none() {
        let [ctrl, alt] = ALTGR_VKS;
        let want = [VK_SHIFT, VK_SHIFT, ctrl, ctrl, alt, alt];
        for ((sided, _), generic_vk) in SIDED.into_iter().zip(want) {
            assert_eq!(generic(sided), Some(generic_vk), "{sided:#04X}");
        }
        for vk in [VK_A, VK_SHIFT, ctrl, alt, VK_CAPITAL] {
            assert_eq!(generic(vk), None, "{vk:#04X}");
        }
        assert!(ALTGR_READ.iter().all(|&vk| generic(vk).is_some()));
    }

    #[cfg(windows)]
    #[test]
    fn the_tables_match_windows() {
        use windows::Win32::UI::Input::KeyboardAndMouse::{
            VK_CAPITAL as CAPITAL, VK_CONTROL, VK_DELETE, VK_DOWN, VK_END, VK_HOME, VK_INSERT,
            VK_LCONTROL, VK_LEFT, VK_LMENU, VK_LSHIFT, VK_LWIN, VK_MENU, VK_NEXT, VK_PRIOR,
            VK_RCONTROL, VK_RIGHT, VK_RMENU, VK_RSHIFT, VK_RWIN, VK_SHIFT as SHIFT,
            VK_SPACE as SPACE, VK_UP,
        };
        let nav = [
            VK_PRIOR, VK_NEXT, VK_END, VK_HOME, VK_LEFT, VK_UP, VK_RIGHT, VK_DOWN, VK_INSERT,
            VK_DELETE,
        ];
        assert_eq!(EXTENDED_VKS, nav.map(|vk| vk.0));
        assert_eq!(VK_SHIFT, SHIFT.0);
        assert_eq!(ALTGR_VKS, [VK_CONTROL.0, VK_MENU.0]);
        assert_eq!(VK_CAPITAL, CAPITAL.0);
        assert_eq!(ALTGR_READ, [VK_LCONTROL.0, VK_RMENU.0]);
        let sided = [
            VK_LSHIFT,
            VK_RSHIFT,
            VK_LCONTROL,
            VK_RCONTROL,
            VK_LMENU,
            VK_RMENU,
        ];
        assert_eq!(SIDED.map(|(s, _)| s), sided.map(|vk| vk.0));
        let named = [super::VK_LSHIFT, super::VK_RSHIFT, super::VK_RCONTROL];
        assert_eq!(named, [VK_LSHIFT.0, VK_RSHIFT.0, VK_RCONTROL.0]);
        assert_eq!(VK_SPACE, SPACE.0);
        let alt_win = [VK_MENU, VK_LMENU, VK_RMENU, VK_LWIN, VK_RWIN];
        assert_eq!(ALT_WIN_VKS, alt_win.map(|vk| vk.0));
    }
}
