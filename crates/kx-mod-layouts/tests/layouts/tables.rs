//! The tables read from the golden layouts: AZERTY `AltGr`, Arabic harakat and لا, and the labels.

use crate::common::{GOLDEN, file, service, source};
use kx_module_api::ports::{KeyChars, Layouts, Scan, Slot};
use kx_platform_fake::FakeLayoutSource;
use proptest::prelude::*;
use std::collections::BTreeMap;
use std::sync::Arc;

/// The table shown while the app in front has golden layout `name`.
fn showing(name: &str) -> BTreeMap<Scan, KeyChars> {
    let fake = source();
    fake.set_foreground(Some(file(name).id));
    let (layouts, _) = service(&fake);
    assert_eq!(layouts.current().label.id, file(name).id);
    layouts.current().keys.clone()
}

/// The text of `scan` in `slot`, or nothing when the table leaves the key out.
fn text(keys: &BTreeMap<Scan, KeyChars>, scan: u16, slot: Slot) -> String {
    keys.get(&Scan::new(scan))
        .map(|k| k.get(slot).to_owned())
        .unwrap_or_default()
}

#[test]
fn azerty_altgr_gives_the_specs_characters_with_its_dead_keys() {
    let keys = showing("fr-fr");
    let want = [
        (0x03, "~"),
        (0x04, "#"),
        (0x05, "{"),
        (0x06, "["),
        (0x07, "|"),
        (0x08, "`"),
        (0x09, "\\"),
        (0x0A, "^"),
        (0x0B, "@"),
        (0x0C, "]"),
        (0x0D, "}"),
        (0x12, "€"),
    ];
    for (scan, ch) in want {
        assert_eq!(text(&keys, scan, Slot::AltGr), ch, "{scan:#x}");
    }
    let dead = |scan: u16, slot| keys[&Scan::new(scan)].is_dead(slot);
    assert!(dead(0x03, Slot::AltGr) && dead(0x08, Slot::AltGr));
    assert!(dead(0x1A, Slot::Base) && !dead(0x04, Slot::AltGr));
}

#[test]
fn arabic_shift_gives_bare_harakat_one_code_point_each() {
    let keys = showing("ar-tn");
    let want = [
        (0x10, '\u{064E}'),
        (0x11, '\u{064B}'),
        (0x12, '\u{064F}'),
        (0x13, '\u{064C}'),
        (0x2E, '\u{0650}'),
        (0x2F, '\u{064D}'),
        (0x2B, '\u{0651}'),
        (0x2D, '\u{0652}'),
    ];
    for (scan, mark) in want {
        assert_eq!(
            text(&keys, scan, Slot::Shift),
            mark.to_string(),
            "{scan:#x}"
        );
    }
    assert_eq!(
        text(&keys, 0x10, Slot::AltGr),
        "\u{0651}\u{064E}",
        "shadda with fatha"
    );
}

#[test]
fn lam_alef_types_two_code_points_in_logical_order() {
    let keys = showing("ar-tn");
    let pairs = [
        (0x30, Slot::Base, "\u{0644}\u{0627}"),
        (0x30, Slot::Shift, "\u{0644}\u{0622}"),
        (0x22, Slot::Shift, "\u{0644}\u{0623}"),
        (0x14, Slot::Shift, "\u{0644}\u{0625}"),
    ];
    for (scan, slot, pair) in pairs {
        let got = text(&keys, scan, slot);
        assert_eq!(got, pair, "{scan:#x} {slot:?}");
        assert!(!got.contains('\u{FEFB}'), "never the ligature form");
    }
}

#[test]
fn a_table_holds_only_keys_that_type_something() {
    let keys = showing("en-us");
    for control in [0x01, 0x0E, 0x0F, 0x1C, 0x39] {
        assert!(!keys.contains_key(&Scan::new(control)), "{control:#x}");
    }
    assert!(keys.values().all(|k| *k != KeyChars::default()));
    assert_eq!(text(&keys, 0x1E, Slot::Base), "a");
}

#[test]
fn the_list_names_each_layout_in_the_installed_order() {
    let (layouts, _) = service(&source());
    let list = layouts.list();
    let shorts: Vec<&str> = list.iter().map(|l| l.short.as_str()).collect();
    assert_eq!(shorts, ["EN", "FR", "ع"]);
    let names: Vec<Vec<String>> = list.iter().map(|l| l.name.clone()).collect();
    let want = [
        ["English", "US"],
        ["français", "French (Legacy, AZERTY)"],
        ["العربية", "Arabic (102) AZERTY"],
    ];
    assert_eq!(names, want);
    let flags: Vec<(bool, bool)> = list.iter().map(|l| (l.rtl, l.arabic)).collect();
    assert_eq!(flags, [(false, false), (false, false), (true, true)]);
}

proptest! {
    #[test]
    fn any_order_and_front_shows_that_layouts_golden_keys(
        order in Just(GOLDEN.to_vec()).prop_shuffle(),
        front in 0usize..GOLDEN.len(),
    ) {
        let fake = Arc::new(FakeLayoutSource::golden(&order).unwrap());
        let want = file(order[front]);
        fake.set_foreground(Some(want.id));
        let (layouts, heard) = service(&fake);
        let table = layouts.current();
        prop_assert_eq!(table.label.id, want.id);
        prop_assert_eq!(&table.keys, &want.keys);
        let listed: Vec<_> = layouts.list().iter().map(|l| l.id).collect();
        let installed: Vec<_> = order.iter().map(|n| file(n).id).collect();
        prop_assert_eq!(listed, installed);
        prop_assert!(heard.ids().is_empty(), "starting is not a change");
    }
}
