//! Held keys on the captured tables, as `ToUnicodeEx` reads them: Ctrl lets controls and letter keys type but no
//! digit key, and Win, Shift or Alt alone change nothing.

use super::*;
use std::ops::RangeInclusive;

/// Left and right Ctrl, left and right Alt, left Win and left Shift.
const CTRL: Scan = Scan::new(0x1D);
const RIGHT_CTRL: Scan = Scan::new(0xE01D);
const ALT: Scan = Scan::new(0x38);
const RIGHT_ALT: Scan = Scan::new(0xE038);
const WIN: Scan = Scan::new(0xE05B);
const SHIFT: Scan = Scan::new(0x2A);
/// The keys Windows gives virtual keys A to Z on US English and on Arabic (102) AZERTY, which keeps US letters.
const US_LETTERS: [RangeInclusive<u16>; 3] = [0x10..=0x19, 0x1E..=0x26, 0x2C..=0x32];
/// The same on French AZERTY, whose M sits beside L.
const FR_LETTERS: [RangeInclusive<u16>; 3] = [0x10..=0x19, 0x1E..=0x27, 0x2C..=0x31];
/// The keys of virtual keys 1 to 9 on all three.
const DIGITS: RangeInclusive<u16> = 0x02..=0x0A;
/// Esc, Backspace, Enter and Space: controls that still type with Ctrl.
const CONTROLS: [u16; 4] = [0x01, 0x0E, 0x1C, 0x39];
/// French `ù`, `;` and `!`: characters on keys that are no letter key, so Ctrl makes them type nothing.
const FR_SIGNS: [u16; 3] = [0x28, 0x33, 0x35];

/// Each captured table's name and its letter keys.
fn golden() -> [(&'static str, [RangeInclusive<u16>; 3]); 3] {
    [
        ("en-us", US_LETTERS),
        ("fr-fr", FR_LETTERS),
        ("ar-tn", US_LETTERS),
    ]
}

/// The fake over captured table `name`, and its layout.
fn source(name: &str) -> (FakeLayoutSource, LayoutId) {
    let fake = FakeLayoutSource::golden(&[name]).unwrap();
    let id = fake.installed()[0];
    (fake, id)
}

#[test]
fn ctrl_lets_letter_keys_and_controls_type_but_no_digit_key() {
    for (name, letters) in golden() {
        let (fake, id) = source(name);
        for ctrl in [CTRL, RIGHT_CTRL] {
            let types = |code: u16| fake.types_any(id, Scan::new(code), &[ctrl]);
            for code in letters.iter().cloned().flatten().chain(CONTROLS) {
                assert!(types(code), "{name}: {code:#04X} types with Ctrl");
            }
            for code in DIGITS {
                assert!(!types(code), "{name}: {code:#04X} types nothing with Ctrl");
            }
        }
    }
}

#[test]
fn ctrl_makes_french_signs_type_nothing() {
    let (fake, id) = source("fr-fr");
    for code in FR_SIGNS.map(Scan::new) {
        assert!(fake.types_any(id, code, &[]), "{code}");
        assert!(!fake.types_any(id, code, &[CTRL]), "{code}");
        assert!(!fake.types_any(id, code, &[SHIFT, CTRL]), "{code}");
    }
}

#[test]
fn win_shift_or_alt_alone_change_nothing() {
    for (name, _) in golden() {
        let (fake, id) = source(name);
        let table = LayoutFile::golden(name).unwrap();
        for key in table.keys.keys().chain(&table.controls).copied() {
            let plain = fake.types_any(id, key, &[]);
            for held in [WIN, SHIFT, ALT, RIGHT_ALT] {
                let with = fake.types_any(id, key, &[held]);
                assert_eq!(with, plain, "{name}: {key} with {held}");
            }
        }
    }
}

#[test]
fn an_unknown_layout_types_nothing_with_ctrl() {
    let (fake, _) = source("en-us");
    assert!(!fake.types_any(LayoutId::new(1), Scan::new(0x10), &[CTRL]));
}
