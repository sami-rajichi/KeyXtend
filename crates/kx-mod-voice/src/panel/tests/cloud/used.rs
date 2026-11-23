//! A language that dictates with a cloud pick (voice-b C1, edge 29): no local row is in use, so the default keeps Use,
//! and the Home names the cloud model with the device picked, not the card the local model would take. A pick waiting
//! for a service Off leaves no local row in use either, the Home names it, and the Cloud side lists it in the services'
//! order.

use kx_module_api::ports::{
    DeviceLabel, DeviceName, ModelSide, PanelButton, ServiceMark, VoiceClick, VoiceHome,
};

use super::super::super::rig::{Rig, card};
use super::super::device::clips::parakeet_in;
use super::super::{models, on_models};
use super::{home, put};
use crate::models::table;
use crate::panel::{Key, Panel};
use crate::services::tests::shipped::{entry, first, model, pick};
use crate::services::{self, Service};

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
fn a_pick_waiting_for_a_service_off_leaves_no_local_row_in_use_and_shows_off_on_the_cloud_side() {
    let mut rig = on_local();
    pick_cloud(&mut rig);
    rig.settings.services.clear(); // Groq Off, its pick kept (owner, 2026-11-21).
    let view = rig.view().unwrap();
    let rows = &models(&view).rows;
    assert!(rows.iter().all(|r| !r.used), "Start shows the Off note");
    rig.click(&VoiceClick::Side(ModelSide::Cloud));
    let view = rig.view().unwrap();
    let name = &model(first(), 0).name;
    let held = models(&view).rows.iter().find(|r| r.name.expose() == name);
    let mark = held.and_then(|r| r.cloud.as_ref()).map(|c| c.mark);
    let off = Some(ServiceMark::Off);
    assert_eq!(mark, off, "the pick, with its Off reason");
}

#[test]
fn a_waiting_pick_sits_on_the_cloud_side_in_the_services_order() {
    let knows = |r: &&Service| r.models.iter().any(|m| m.code("en").is_some());
    let table = services::table().unwrap();
    let rows: Vec<&Service> = table.offered().filter(knows).take(3).collect();
    let mut rig = on_local();
    let on = [entry(rows[0], ""), entry(rows[2], "")].join("\n");
    put(&mut rig, "services", &on);
    let n = rows[1].models.iter().position(|m| m.code("en").is_some());
    let waits = format!(r#"en = "{}""#, pick(rows[1], n.unwrap()));
    put(&mut rig, "picks", &waits);
    let list = rig.with(|_, f| Panel::cloud_list(f));
    let mut ids: Vec<&str> = (list.iter())
        .filter_map(|(key, _)| match key {
            Key::Cloud { service, .. } => Some(service.as_str()),
            _ => None,
        })
        .collect();
    ids.dedup();
    let order: Vec<&str> = rows.iter().map(|r| r.id.as_str()).collect();
    assert_eq!(ids, order, "the Off one between the two on");
}

#[test]
fn the_home_names_the_cloud_model_and_the_device_picked_while_a_cloud_pick_holds_or_waits() {
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
    rig.settings.services.clear();
    let waits = home(&mut rig);
    let named = (waits.model.map(|m| m.expose().clone()), waits.device);
    let cpu = DeviceLabel::Auto(DeviceName::Cpu);
    assert_eq!(named, (Some(model(first(), 0).name.clone()), cpu), "Off");
    rig.settings.cloud = false;
    let back = home(&mut rig);
    let shown = |h: &VoiceHome| (h.model.clone(), h.device.clone());
    assert_eq!(
        shown(&back),
        shown(&local),
        "cloud off gives the local model back"
    );
}
