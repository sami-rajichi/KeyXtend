//! Tests of the key tables and the tags on input.

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
fn the_tags_spell_their_names_and_differ() {
    let name = |tag: usize| tag.to_be_bytes()[size_of::<usize>() - 4..].to_vec();
    assert_eq!(name(TAG), b"KXSP");
    assert_eq!(name(REPLAY)[..2], *b"KR", "its low half holds the id");
    assert_eq!(name(SIM_TAG), b"KXUS");
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
    let (ctrl, alt) = (VK_CONTROL, VK_MENU);
    let want = [VK_SHIFT, VK_SHIFT, ctrl, ctrl, alt, alt];
    for ((sided, _), generic_vk) in SIDED.into_iter().zip(want) {
        assert_eq!(generic(sided), Some(generic_vk), "{sided:#04X}");
    }
    for vk in [VK_A, VK_SHIFT, ctrl, alt, VK_CAPITAL] {
        assert_eq!(generic(vk), None, "{vk:#04X}");
    }
    assert!(ALTGR_VKS.iter().all(|&vk| generic(vk).is_some()));
}

#[cfg(windows)]
#[test]
fn the_tables_match_windows() {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        VK_CAPITAL as CAPITAL, VK_CONTROL, VK_DELETE, VK_DOWN, VK_END, VK_HOME, VK_INSERT,
        VK_LCONTROL, VK_LEFT, VK_LMENU, VK_LSHIFT, VK_LWIN, VK_MENU, VK_NEXT, VK_PRIOR,
        VK_RCONTROL, VK_RIGHT, VK_RMENU, VK_RSHIFT, VK_RWIN, VK_SHIFT as SHIFT, VK_SPACE as SPACE,
        VK_UP,
    };
    let nav = [
        VK_PRIOR, VK_NEXT, VK_END, VK_HOME, VK_LEFT, VK_UP, VK_RIGHT, VK_DOWN, VK_INSERT, VK_DELETE,
    ];
    assert_eq!(EXTENDED_VKS, nav.map(|vk| vk.0));
    assert_eq!(VK_SHIFT, SHIFT.0);
    assert_eq!(
        [super::VK_CONTROL, super::VK_MENU],
        [VK_CONTROL.0, VK_MENU.0]
    );
    assert_eq!(VK_CAPITAL, CAPITAL.0);
    assert_eq!(ALTGR_VKS, [VK_LCONTROL.0, VK_RMENU.0]);
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

#[test]
fn a_replay_tag_carries_its_acts_id_beside_the_replay_mark() {
    for id in [0, 1, 0x8000, u16::MAX] {
        let tag = replay_tag(id);
        assert!(is_replay(tag), "{id:#x}");
        assert_eq!(replay_id(tag), id);
        assert_eq!(tag & !0xFFFF, REPLAY);
    }
}

#[test]
fn only_the_replay_mark_takes_an_id_and_the_other_tags_stay_whole() {
    assert!(is_replay(REPLAY), "id 0");
    let high = 1 << 40;
    for whole in [TAG, SIM_TAG, 0, TAG | high, SIM_TAG | high, REPLAY | high] {
        assert!(!is_replay(whole), "{whole:#x}");
    }
}

#[test]
#[allow(
    clippy::cast_possible_truncation,
    reason = "Windows keeps only the low 32 bits of a mouse input's extra, as this cast does."
)]
fn an_id_survives_windows_cutting_a_mouse_inputs_extra_to_32_bits() {
    for id in [0, 1, 0x7FFF, 0x8000, 0xFFFE, 0xFFFF] {
        let cut = replay_tag(id) as u32 as usize;
        assert!(is_replay(cut), "{id:#x}");
        assert_eq!(replay_id(cut), id, "{id:#x}");
    }
}
