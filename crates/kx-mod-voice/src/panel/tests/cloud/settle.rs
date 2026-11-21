//! Open service on a cloud note (final review m5): the panel opens, closed or on another page, on the service's card or
//! on its key page for the key note, Back goes to the Cloud page, and with cloud off the Cloud page opens, with its
//! switch.

use kx_module_api::ports::{PanelButton, VoiceClick, VoiceTab};

use super::super::super::rig::Rig;
use super::super::super::{Do, Page};
use super::{card, lines_sheet, put};
use crate::services::tests::shipped::{entry, first};

/// A closed panel with cloud on, and the first offered row on with its key sealed.
fn closed() -> Rig {
    let mut rig = Rig::new();
    rig.settings.cloud = true;
    let row = first();
    put(&mut rig, "services", &entry(row, ""));
    rig.keyed.insert(row.id.clone());
    rig.panel.close();
    rig
}

/// Open service on the first offered row's note, the key note when `key`.
fn settle(rig: &mut Rig, key: bool) -> Vec<Do> {
    let id = first().id.clone();
    rig.with(|panel, facts| panel.settle(&id, key, facts))
}

#[test]
fn open_service_opens_the_closed_panel_on_the_card_and_back_goes_to_the_cloud_page() {
    let mut rig = closed();
    settle(&mut rig, false);
    assert_eq!(card(&mut rig).service, first().shown());
    rig.click(&VoiceClick::Button(PanelButton::Back));
    assert_eq!(rig.panel.page(), Some(Page::Tab(VoiceTab::Cloud)));
}

#[test]
fn open_service_on_the_key_note_opens_the_key_page_with_its_dots_read() {
    let mut rig = closed();
    rig.panel.open();
    rig.click(&VoiceClick::Open(VoiceTab::Models));
    assert_eq!(rig.panel.page(), Some(Page::Tab(VoiceTab::Models)));
    let dos = settle(&mut rig, true);
    assert_eq!(dos, [Do::Dots(first().id.clone())]);
    assert_eq!(lines_sheet(&mut rig).service, first().shown());
    rig.click(&VoiceClick::Button(PanelButton::Back));
    assert_eq!(rig.panel.page(), Some(Page::Tab(VoiceTab::Cloud)));
}

#[test]
fn open_service_with_cloud_off_opens_the_cloud_page_where_its_switch_is() {
    let mut rig = closed();
    rig.settings.cloud = false;
    settle(&mut rig, false);
    assert_eq!(rig.panel.page(), Some(Page::Tab(VoiceTab::Cloud)));
}
