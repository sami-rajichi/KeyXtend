//! Every character the captured EN, FR and AR layouts type sits on some board key, the numpad aside.

#![allow(
    clippy::unwrap_used,
    reason = "the golden tables are test data, where a failed read should fail the test"
)]

use kx_mod_keyboard::{BoardSpec, board};
use kx_module_api::ports::{KeyChars, Scan, Slot};
use kx_platform_fake::{FILE_EXT, LayoutFile, golden_dir};
use std::collections::BTreeSet;

/// The captured tables: English, French AZERTY and Arabic (102) AZERTY.
const GOLDEN: [&str; 3] = ["en-us", "fr-fr", "ar-tn"];

/// True for a numpad key, which the board leaves to a later numpad.
fn numpad(sc: Scan) -> bool {
    matches!(sc.code(), 0x37 | 0x47..=0x53 | 0xE035)
}

/// Every text the keys in `keys` type, in any state.
fn texts<'a>(keys: impl Iterator<Item = &'a KeyChars>) -> BTreeSet<String> {
    let all = keys.flat_map(|c| Slot::ALL.map(|s| c.get(s).to_owned()));
    all.filter(|t| !t.is_empty()).collect()
}

#[test]
fn every_golden_character_sits_on_a_board_key() {
    let spec = BoardSpec::shipped().unwrap();
    let keys = board(&spec, 0.0, 1.0).keys;
    let on_board: BTreeSet<Scan> = keys.iter().filter_map(|k| k.sc).collect();
    let mut missing = Vec::new();
    for name in GOLDEN {
        let path = golden_dir().join(name).with_extension(FILE_EXT);
        let table = LayoutFile::read(&path).unwrap();
        let typing = table.keys.iter().filter(|(sc, _)| !numpad(**sc));
        let (on, off): (Vec<_>, Vec<_>) = typing.partition(|(sc, _)| on_board.contains(sc));
        let reach = texts(on.into_iter().map(|(_, c)| c));
        let lost = texts(off.into_iter().map(|(_, c)| c));
        missing.extend(lost.difference(&reach).map(|t| format!("{name} {t}")));
    }
    assert!(missing.is_empty(), "not on the board: {missing:?}");
}
