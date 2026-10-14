//! Key tables: the tags on input, keys Windows sends or reports as extended, how `VkKeyScanEx` names the
//! modifiers a character needs, the keys `ToUnicodeEx` reads held down, and the menu mask key.

use std::fmt;

use kx_module_api::REDACTED_MARKER;

/// Marks input we inject (`dwExtraInfo`, "KXSP"), so our own hooks can skip it.
pub const TAG: usize = 0x4B58_5350;
/// Marks the input the pointer hook sends for its engine ("KXSR") in the low 32 bits of `dwExtraInfo`, the act's id in
/// the high 32; our hooks skip it and the gate guard can tell it.
pub const REPLAY: usize = 0x4B58_5352;
/// Marks the gate tool's simulated hand ("KXUS"); only a pointer hook built for simulated input acts on it.
pub const SIM_TAG: usize = 0x4B58_5553;
/// Where a replayed act's id sits in `dwExtraInfo`: the high 32 bits, above `REPLAY`.
const ID_SHIFT: u32 = 32;
/// The low 32 bits of `dwExtraInfo`, which hold `REPLAY`.
const MARK_MASK: usize = (1 << ID_SHIFT) - 1;
// The id needs the high half of a 64-bit `dwExtraInfo`, so this crate builds only for 64-bit Windows.
const _: () = assert!(
    usize::BITS == 2 * ID_SHIFT,
    "the replay tag needs a 64-bit dwExtraInfo"
);

/// The `dwExtraInfo` of our replayed act `id`: `REPLAY` in the low 32 bits and `id` in the high 32.
#[must_use]
pub fn replay_tag(id: u32) -> usize {
    (usize::try_from(id).unwrap_or_default() << ID_SHIFT) | REPLAY
}

/// True when `extra` marks input the pointer hook replayed, whatever act it names; `TAG` and `SIM_TAG` stay whole.
#[must_use]
pub fn is_replay(extra: usize) -> bool {
    extra & MARK_MASK == REPLAY
}

/// The act id in our replay tag `extra`.
pub(crate) fn replay_id(extra: usize) -> u32 {
    u32::try_from(extra >> ID_SHIFT).unwrap_or(u32::MAX)
}

/// Prefix bytes `MapVirtualKeyExW` puts on extended scan codes (`0xE0`, `0xE1`).
pub(crate) const EXTENDED_PREFIXES: [u8; 2] = [0xE0, 0xE1];

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
/// Left Ctrl and right Alt, in that order: the keys a physical `AltGr` holds, as Windows reports them and as the Keys
/// route sends them for a character on the `AltGr` level.
pub const ALTGR_VKS: [u16; 2] = [VK_LCONTROL, VK_RMENU];
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
    /// The `AltGr` level, which `VkKeyScanEx` names as Ctrl and Alt held.
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
mod tests;
