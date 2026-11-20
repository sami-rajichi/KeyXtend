//! A language that dictates with a cloud pick (voice-b C1, edge 29): no local row is in use, so the default keeps Use,
//! and the Home names the cloud model with the device picked, not the card the local model would take.

use kx_module_api::ports::{DeviceLabel, DeviceName, PanelButton, VoiceClick, VoiceHome};

use super::super::super::rig::{Rig, card};
use super::super::device::clips::parakeet_in;
use super::super::{models, on_models};
use super::{home, put};
use crate::models::table;
use crate::services::tests::shipped::{entry, first, model, pick};

/// The rig on the Models page with English's default held, a card that passed its check, and cloud on.
fn on_local() -> Rig {
    let default = table().unwrap().default_for("en").unwrap();
    let mut rig = on_models(&[default.id.as_str()]);
    rig.devices.push(card(4_096));
    rig.checks.put(parakeet_in(Some("en")));
    rig.settings.cloud = true;
    rig
}

/// English picks the first offered row's first model, that row on with a key.
fn pick_cloud(rig: &mut Rig) {
    let row = first();
    put(rig, "services", &entry(row, ""));
    put(rig, "picks", &format!(r#"en = "{}""#, pick(row, 0)));
    rig.keyed.insert(row.id.clone());
}

#[test]
fn a_cloud_pick_leaves_no_local_row_in_use_so_one_click_moves_back_to_the_default() {
    let mut rig = on_local();
    assert!(
        models(&rig.view().unwrap()).rows[0].used,
        "the default before"
    );
    pick_cloud(&mut rig);
    let view = rig.view().unwrap();
    assert!(
        models(&view).rows.iter().all(|r| !r.used),
        "the cloud model is the one in use"
    );
    rig.click(&VoiceClick::Row(0));
    let acts = rig.view().unwrap().acts;
    let ready = [PanelButton::Use, PanelButton::Delete];
    assert!(
        ready.iter().all(|b| acts.contains(b)),
        "as the mock-up: {acts:?}"
    );
}

#[test]
fn the_home_names_the_cloud_model_and_the_device_picked_while_a_cloud_pick_holds() {
    let mut rig = on_local();
    let local = home(&mut rig);
    assert_eq!(
        local.device,
        DeviceLabel::Auto(DeviceName::Card("Card".into()))
    );
    pick_cloud(&mut rig);
    let cloud = home(&mut rig);
    let name = model(first(), 0).name.clone();
    assert_eq!(cloud.model.map(|m| m.expose().clone()), Some(name));
    assert_eq!(cloud.device, DeviceLabel::Auto(DeviceName::Cpu));
    rig.settings.cloud = false;
    let back = home(&mut rig);
    let shown = |h: &VoiceHome| (h.model.clone(), h.device.clone());
    assert_eq!(
        shown(&back),
        shown(&local),
        "cloud off gives the local model back"
    );
}
