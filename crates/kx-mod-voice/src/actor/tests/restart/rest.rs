//! Every option changed, the owner's named ones among them, and the device picked survive a restart, as the Options and
//! Device pages show them.

use kx_module_api::ports::{
    DeviceItem, DeviceRow, OptionRow, PanelButton, VoiceClick, VoiceSheet, VoiceTab,
};

use super::super::cards::devices as cards;
use super::super::rig::Rig;
use super::{open, restart, sheet};

/// The options the owner named, which this test must see changed.
const NAMED: [&str; 4] = ["insert", "place", "sounds", "history"];

/// The Options page's rows as last drawn.
fn rows(rig: &Rig) -> Vec<OptionRow> {
    match sheet(rig) {
        VoiceSheet::Options(rows) => rows,
        other => panic!("not the Options page: {other:?}"),
    }
}

/// True when the shown page is the list's last.
fn last_page(rig: &Rig) -> bool {
    let panel = rig.views().last().and_then(|v| v.panel.clone());
    let pages = panel.unwrap().pages;
    pages.at + 1 >= pages.count
}

/// Every option row, page by page from the first.
fn every_row(rig: &mut Rig) -> Vec<OptionRow> {
    open(rig, 0, VoiceTab::Options);
    let mut all = rows(rig);
    while !last_page(rig) {
        rig.act(VoiceClick::Page { back: false });
        all.extend(rows(rig));
    }
    all
}

/// Changes the shown page's row `at` once: a switch flips, anything else steps up, or down at its top.
fn change(rig: &mut Rig, at: u8) {
    let before = rows(rig)[usize::from(at)];
    let up = |up| VoiceClick::Step { row: at, up };
    for click in [VoiceClick::Flip(at), up(true), up(false)] {
        rig.act(click);
        if rows(rig).get(usize::from(at)) != Some(&before) {
            return;
        }
    }
}

/// The row keyed `key` among `rows`.
fn row<'r>(rows: &'r [OptionRow], key: &str) -> &'r OptionRow {
    rows.iter().find(|r| r.key == key).unwrap()
}

/// The Device page's rows as last drawn.
fn devices(rig: &Rig) -> Vec<DeviceRow> {
    match sheet(rig) {
        VoiceSheet::Device(page) => page.rows,
        other => panic!("not the Device page: {other:?}"),
    }
}

/// The device the Device page shows picked.
fn picked(rig: &Rig) -> Option<DeviceItem> {
    devices(rig).into_iter().find(|r| r.used).map(|r| r.item)
}

#[test]
fn every_option_changed_survives_a_restart() {
    let mut rig = Rig::new();
    open(&mut rig, 0, VoiceTab::Options);
    loop {
        let mut at = 0;
        while at < rows(&rig).len() {
            change(&mut rig, u8::try_from(at).unwrap());
            at += 1;
        }
        if last_page(&rig) {
            break;
        }
        rig.act(VoiceClick::Page { back: false });
    }
    let (changed, defaults) = (every_row(&mut rig), every_row(&mut Rig::new()));
    for key in NAMED {
        assert_ne!(row(&changed, key), row(&defaults, key), "{key} changed");
    }
    assert_eq!(every_row(&mut restart(&rig)), changed);
}

#[test]
fn the_device_picked_survives_a_restart() {
    let mut rig = Rig::new();
    rig.fake.speech.set_devices(Ok(cards()));
    open(&mut rig, 0, VoiceTab::Device);
    let at = devices(&rig).iter().position(|r| !r.used).unwrap();
    rig.act(VoiceClick::Row(u8::try_from(at).unwrap()));
    rig.act(VoiceClick::Button(PanelButton::Use));
    let chosen = picked(&rig);
    assert!(chosen.is_some(), "a device is picked");
    let mut fresh = restart(&rig);
    open(&mut fresh, 0, VoiceTab::Device);
    assert_eq!(picked(&fresh), chosen);
}
