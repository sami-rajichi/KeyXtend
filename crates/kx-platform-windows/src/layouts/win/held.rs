//! The facts the fake layout source's held keys rest on, read from the real `ToUnicodeEx` on every installed layout:
//! Ctrl makes a letter key type its control character and a digit key nothing, and Win or Alt alone change nothing.

use super::{Win32, WinLayouts};
use crate::layouts::{KEY_TEXT_UNITS, Os, state};
use kx_module_api::ports::{LayoutId, LayoutSource, Scan};
use std::ops::RangeInclusive;

/// Left and right Ctrl, left and right Alt, left Win and left Shift, by scan code.
const CTRL: [u16; 2] = [0x1D, 0xE01D];
const ALT: [u16; 2] = [0x38, 0xE038];
const WIN: u16 = 0xE05B;
const SHIFT: u16 = 0x2A;
/// The scan codes of the main block, Esc to F12 with the number pad, where every letter and digit key lies.
const MAIN: RangeInclusive<u16> = 0x01..=0x58;
/// Virtual keys A to Z, as their ASCII capitals.
const LETTER_VKS: RangeInclusive<u16> = 0x41..=0x5A;
/// Virtual keys 1 to 9; 0 is left out, as French Ctrl+0 types U+0000 where the fake types nothing.
const DIGIT_VKS: RangeInclusive<u16> = 0x31..=0x39;
/// What Ctrl+A types; each later letter types the next code.
const CTRL_A: u16 = 0x01;

/// Each key of the main block in `layout` that has a virtual key, with it.
fn keys(layout: LayoutId) -> impl Iterator<Item = (Scan, u16)> {
    let vk = move |key: Scan| (key, Win32.vk_of(key, layout));
    MAIN.map(Scan::new).map(vk).filter(|&(_, vk)| vk != 0)
}

/// Windows' answer for `key`, virtual key `vk`, in `layout` with the scan codes `held` down: the count and the text.
fn typed(layout: LayoutId, key: Scan, vk: u16, held: &[u16]) -> (i32, [u16; KEY_TEXT_UNITS]) {
    let vks: Vec<u16> = held
        .iter()
        .map(|&h| Win32.vk_of(Scan::new(h), layout))
        .collect();
    let mut buf = [0; KEY_TEXT_UNITS];
    let n = Win32.to_unicode(vk, key, &state(&vks, false), layout, &mut buf);
    (n, buf)
}

/// True when `key` types anything in `layout` with the scan codes `held` down.
fn types(layout: LayoutId, key: Scan, held: &[u16]) -> bool {
    let held: Vec<Scan> = held.iter().copied().map(Scan::new).collect();
    WinLayouts.types_any(layout, key, &held)
}

#[test]
fn ctrl_makes_each_letter_key_type_its_own_control_character() {
    for layout in WinLayouts.installed() {
        let letters: Vec<_> = keys(layout)
            .filter(|(_, vk)| LETTER_VKS.contains(vk))
            .collect();
        assert_eq!(
            letters.len(),
            LETTER_VKS.len(),
            "{layout}: a key per letter"
        );
        for (key, vk) in letters {
            let want = vk - LETTER_VKS.start() + CTRL_A;
            for ctrl in CTRL {
                let (n, buf) = typed(layout, key, vk, &[ctrl]);
                assert_eq!((n, buf[0]), (1, want), "{layout}: {key}");
            }
        }
    }
}

#[test]
fn ctrl_makes_each_digit_key_type_nothing() {
    for layout in WinLayouts.installed() {
        let digits: Vec<_> = keys(layout)
            .filter(|(_, vk)| DIGIT_VKS.contains(vk))
            .collect();
        assert_eq!(digits.len(), DIGIT_VKS.len(), "{layout}: a key per digit");
        for (key, _) in digits {
            assert!(types(layout, key, &[]), "{layout}: {key} types alone");
            for ctrl in CTRL {
                assert!(!types(layout, key, &[ctrl]), "{layout}: {key}");
            }
        }
    }
}

#[test]
fn win_or_alt_alone_change_nothing_and_shift_keeps_letters_and_digits_typing() {
    for layout in WinLayouts.installed() {
        for (key, vk) in keys(layout) {
            let plain = types(layout, key, &[]);
            for held in ALT.into_iter().chain([WIN]) {
                assert_eq!(types(layout, key, &[held]), plain, "{layout}: {key}");
            }
            if LETTER_VKS.contains(&vk) || DIGIT_VKS.contains(&vk) {
                assert!(types(layout, key, &[SHIFT]), "{layout}: {key}");
            }
        }
    }
}
