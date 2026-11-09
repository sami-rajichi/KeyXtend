//! The strip's gate ids and title key against kx-ui's QML and its keys, so a renamed part breaks a test here.

use kx_module_api::ports::SelChip;

use crate::config::load;

// Workspace paths from `tools/kx-gates/src/p10cfg/tests`; a moved file breaks the build here, never silently.
/// The strip's names, its gate-id group among them.
const NAMES_QML: &str = include_str!("../../../../../crates/kx-ui/qml/SelNames.qml");
/// One keycap, which gives itself its UI Automation id from the group and its chip's id.
const KEY_QML: &str = include_str!("../../../../../crates/kx-ui/qml/SelKey.qml");
/// kx-ui's selection keys: the strip's name and each chip's id.
const KEYS: &str = include_str!("../../../../../crates/kx-ui/src/i18n/keys/sel.rs");
/// The keyboard's top bar, whose buttons take their ids from the bar group and their part.
const TOP_BAR: &str = include_str!("../../../../../crates/kx-ui/qml/TopBar.qml");
/// The keyboard's resize corner.
const CORNER: &str = include_str!("../../../../../crates/kx-ui/qml/Corner.qml");

#[test]
fn the_tray_and_corner_ids_are_the_ones_kx_ui_gives_those_buttons() {
    let g = load().expect("loads").g19_product;
    let (group, part) = g.tray.split_once('.').expect("a group and a part");
    assert!(
        TOP_BAR.contains(&format!("bar.res.uid(\"{group}\", part)")),
        "bar group"
    );
    let close = format!("part: \"{part}\"\n            onClicked: bar.close()");
    assert!(TOP_BAR.contains(&close), "{close}");
    let (group, part) = g.corner.split_once('.').expect("a group and a part");
    let id = format!("Accessible.id: res.uid(\"{group}\", \"{part}\")");
    assert!(CORNER.contains(&id), "{id}");
    assert!(
        CORNER.contains("width: px[0] * s") && CORNER.contains("px: lk.shape.extra.corner_px"),
        "its width is the token at the keyboard's size"
    );
}

#[test]
fn the_group_and_the_chip_ids_are_the_ones_kx_ui_gives_its_keycaps() {
    let g = load().expect("loads").g19_product;
    assert!(
        NAMES_QML.contains(&format!("group: \"{}\"", g.group)),
        "group"
    );
    assert!(KEY_QML.contains("res.uid(names.group, chip.id)"), "keycaps");
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
