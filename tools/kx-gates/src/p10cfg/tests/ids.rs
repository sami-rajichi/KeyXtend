//! The strip's gate ids and title key against kx-ui's QML and its keys, so a renamed part breaks a test here.

use kx_module_api::ports::SelChip;

use crate::config::load;

// Workspace paths from `tools/kx-gates/src/p10cfg/tests`; a moved file breaks the build here, never silently.
/// One keycap, which gives itself its UI Automation id from the group and its chip's id.
const KEY_QML: &str = include_str!("../../../../../crates/kx-ui/qml/SelKey.qml");
/// kx-ui's selection keys: the strip's name and each chip's id.
const KEYS: &str = include_str!("../../../../../crates/kx-ui/src/i18n/keys/sel.rs");

#[test]
fn the_group_and_the_chip_ids_are_the_ones_kx_ui_gives_its_keycaps() {
    let g = load().expect("loads").g19_product;
    assert!(
        KEY_QML.contains(&format!("res.uid(\"{}\", chip.id)", g.group)),
        "group"
    );
    assert_eq!(g.chips.len(), SelChip::ALL.len());
    for (chip, id) in SelChip::ALL.iter().zip(&g.chips) {
        let arm = format!("SelChip::{chip:?} => \"{id}\"");
        assert!(KEYS.contains(&arm), "{arm}");
    }
    assert!(
        KEYS.contains(&format!("SEL_NAME: &str = \"{}\"", g.strip_title)),
        "title"
    );
    assert_eq!(g.chip_id(SelChip::Paste), Some("sel.paste".to_string()));
}

#[test]
fn a_keycap_id_reads_back_as_its_chip_and_any_other_id_as_none() {
    let g = load().expect("loads").g19_product;
    for chip in SelChip::ALL {
        let id = g.chip_id(chip).expect("an id");
        assert_eq!(g.chip_of(&id), Some(chip), "{id}");
    }
    for other in ["sel.copied", "clip.copy", "copy", "", "sel."] {
        assert_eq!(g.chip_of(other), None, "{other}");
    }
}
