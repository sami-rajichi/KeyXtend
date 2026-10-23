//! A shortcut's tip names its keys in the keyboard's language: the board's labels, then the key-name texts, then
//! letters, digits and F-keys as themselves; keys join with `combo.join` and steps with `combo.then`.

use crate::{CONTROLS, all, lang};
use kx_module_api::ports::KeyName;
use kx_ui::i18n::keys::{COMBO_JOIN, COMBO_THEN, KEYS_TIP};
use proptest::prelude::*;

/// Copy all's steps on Windows.
const COPY_ALL: [&[&str]; 2] = [&["ctrl", "a"], &["ctrl", "c"]];
/// Reopen closed tab's step on Windows.
const REOPEN_TAB: [&[&str]; 1] = [&["ctrl", "shift", "t"]];
/// Delete all's steps on Windows.
const DELETE_ALL: [&[&str]; 2] = [&["ctrl", "a"], &["del"]];
/// Emoji's, Zoom in's and Zoom out's steps on Windows, which end on a period or a sign.
const EMOJI: [&[&str]; 1] = [&["win", "period"]];
const ZOOM_IN: [&[&str]; 1] = [&["ctrl", "numplus"]];
const ZOOM_OUT: [&[&str]; 1] = [&["ctrl", "numminus"]];

/// The tips the Quick reading-order test draws, by name key and steps: each ends on a period or a sign, or holds
/// Arabic's comma, which a right-to-left line would move.
pub const ORDERED: [(&str, &[&[&str]]); 4] = [
    ("shortcut.emoji", &EMOJI),
    ("shortcut.zoom-in", &ZOOM_IN),
    ("shortcut.zoom-out", &ZOOM_OUT),
    ("shortcut.copy-all", &COPY_ALL),
];

/// Key name `name`.
fn key(name: &str) -> KeyName {
    KeyName::new(name).expect("a well-formed key name")
}

/// `steps` as key names.
pub fn steps(steps: &[&[&str]]) -> Vec<Vec<KeyName>> {
    steps
        .iter()
        .map(|s| s.iter().map(|n| key(n)).collect())
        .collect()
}

#[test]
fn copy_all_names_both_steps() {
    let tip = lang("en").keys_tip("shortcut.copy-all", &steps(&COPY_ALL));
    assert_eq!(tip, "Copy all (Ctrl+A, Ctrl+C)");
}

#[test]
fn french_names_shift_as_its_keyboards_print_it() {
    let tip = lang("fr").keys_tip("shortcut.reopen-tab", &steps(&REOPEN_TAB));
    assert_eq!(tip, "Rouvrir l’onglet fermé (Ctrl+Maj+T)");
}

#[test]
fn french_names_delete_suppr() {
    let tip = lang("fr").keys_tip("shortcut.delete-all", &steps(&DELETE_ALL));
    assert_eq!(tip, "Supprimer tout (Ctrl+A, Suppr)");
}

#[test]
fn arabic_puts_the_keys_first_and_joins_the_steps_with_its_comma() {
    let ar = lang("ar");
    let tip = ar.keys_tip("shortcut.copy-all", &steps(&COPY_ALL));
    assert_eq!(tip, "(Ctrl+A، Ctrl+C) نسخ الكل");
    let emoji = ar.keys_tip("shortcut.emoji", &steps(&EMOJI));
    assert_eq!(emoji, "(Win+.) الرموز التعبيرية");
    assert_eq!(ar.text(COMBO_THEN), "\u{60c} ");
}

#[test]
fn every_language_joins_keys_with_a_plus() {
    for t in all() {
        assert_eq!(t.text(COMBO_JOIN), "+", "{}", t.lang().tag);
    }
    assert_eq!(lang("en").text(COMBO_THEN), ", ");
    assert_eq!(lang("fr").text(COMBO_THEN), ", ");
}

#[test]
fn named_keys_read_as_keyboards_print_them_in_every_language() {
    let want = [("period", "."), ("numplus", "Num +"), ("numminus", "Num -")];
    for t in all() {
        for (name, text) in want {
            assert_eq!(t.key_name(&key(name)).as_deref(), Some(text), "{name}");
        }
        assert_eq!(t.keys(&steps(&[&["win", "period"]])), "Win+.");
    }
}

#[test]
fn letters_digits_and_f1_to_f24_print_as_themselves() {
    let en = lang("en");
    for (name, text) in [("a", "A"), ("7", "7"), ("f1", "F1"), ("f24", "F24")] {
        assert_eq!(en.key_name(&key(name)).as_deref(), Some(text), "{name}");
    }
    for name in ["f0", "f01", "f25", "ab", "pgup"] {
        assert_eq!(en.key_name(&key(name)), None, "{name}");
    }
}

#[test]
fn an_unknown_name_shows_in_upper_case() {
    let tip = lang("fr").keys(&steps(&[&["ctrl", "pgup"], &["f25"]]));
    assert_eq!(tip, "Ctrl+PGUP, F25");
}

#[test]
fn a_shortcut_with_no_steps_is_named_alone() {
    for t in all() {
        let name = t.text("shortcut.copy");
        assert_eq!(t.keys_tip("shortcut.copy", &[]), name, "{}", t.lang().tag);
    }
}

#[test]
fn the_tip_has_its_text_and_holds_no_bidi_control() {
    for t in all() {
        assert!(t.has(KEYS_TIP), "{}", t.lang().tag);
        let tip = t.keys_tip("shortcut.copy-all", &steps(&COPY_ALL));
        assert!(!tip.contains(['{', '}']), "{tip}");
        assert!(!tip.chars().any(|c| CONTROLS.contains(&c)), "{tip:?}");
    }
}

proptest! {
    #[test]
    fn any_steps_of_any_names_make_a_tip(names in prop::collection::vec(
        prop::collection::vec("[a-z0-9]{1,8}", 0..4), 0..4,
    )) {
        let steps: Vec<Vec<KeyName>> =
            names.iter().map(|s| s.iter().map(|n| key(n)).collect()).collect();
        for t in all() {
            let tip = t.keys_tip("shortcut.copy", &steps);
            let (name, keys) = (t.text("shortcut.copy"), t.keys(&steps));
            prop_assert!(tip.contains(&name) && tip.contains(&keys), "{}", tip);
        }
    }
}
