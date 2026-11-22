//! A service Off survives a restart with everything the owner chose for it (ADR-0054): a listed pick shows waiting
//! with its Off mark, and the service's Confirm brings back that pick, the typed address and model, their pick and the
//! keep-nothing switch.

use std::sync::Arc;

use kx_module_api::ports::{
    CloudButton, ModelSide, PanelButton, ServiceMark, ServiceName, VoiceClick, VoiceTab,
};
use kx_module_api::{Redacted, Zeroizing};

use super::super::cloud::key;
use super::super::rig::Rig;
use super::{ADDRESS, MODEL, confirm, lines, models, named, open, restart, tick, used};
use crate::keys::Keys;

/// French, as `LANGS` holds it.
const FRENCH: usize = 1;

/// Seals the test's key for each service `ids` names, so its models can be used.
fn seal(rig: &Rig, ids: &[&str]) {
    let keys = Keys::new(Arc::new(rig.fake.shelf.clone()), false);
    for id in ids {
        keys.set(id, Redacted::new(Zeroizing::new(key()))).unwrap();
    }
}

/// The Models page's cloud row of service `name` for language `lang`: its model's name and its service's mark.
fn cloud_row(rig: &mut Rig, lang: usize, name: &ServiceName) -> (String, ServiceMark) {
    open(rig, lang, VoiceTab::Models);
    rig.act(VoiceClick::Side(ModelSide::Cloud));
    let rows = models(rig).rows;
    let row = rows.iter().find_map(|r| {
        let cloud = r.cloud.as_ref().filter(|c| c.service == *name)?;
        Some((r.name.expose().clone(), cloud.mark))
    });
    row.expect("its row is listed")
}

/// Uses service `name`'s first listed model for language `lang`, the only service on; gives the name in use.
fn use_first(rig: &mut Rig, lang: usize, name: &ServiceName) -> String {
    let (model, _) = cloud_row(rig, lang, name);
    rig.act(VoiceClick::Row(0));
    rig.act(VoiceClick::Button(PanelButton::Use));
    assert_eq!(used(rig).as_deref(), Some(model.as_str()), "Use changed it");
    model
}

/// Switches service `name` Off from the Cloud page.
fn off(rig: &mut Rig, name: &ServiceName) {
    open(rig, 0, VoiceTab::Cloud);
    tick(rig, name);
    rig.act(VoiceClick::Service(CloudButton::Off));
}

#[test]
fn a_listed_pick_of_a_service_off_waits_through_a_restart_and_its_confirm_brings_it_back() {
    let (mut rig, lemonfox) = (Rig::new(), named("Lemonfox"));
    seal(&rig, &["lemonfox"]);
    open(&mut rig, 0, VoiceTab::Cloud);
    rig.act(VoiceClick::Cloud(true));
    confirm(&mut rig, &lemonfox);
    let model = use_first(&mut rig, FRENCH, &lemonfox);
    off(&mut rig, &lemonfox);
    let mut fresh = restart(&rig);
    let waiting = (model.clone(), ServiceMark::Off);
    assert_eq!(
        cloud_row(&mut fresh, FRENCH, &lemonfox),
        waiting,
        "it waits"
    );
    assert_eq!(used(&fresh), None, "nothing in use while it waits");
    confirm(&mut fresh, &lemonfox);
    cloud_row(&mut fresh, FRENCH, &lemonfox);
    assert_eq!(used(&fresh), Some(model), "its Confirm brings it back");
}

#[test]
fn a_service_off_keeps_its_typed_lines_pick_and_keep_switch_through_a_restart_for_its_confirm() {
    let mut rig = Rig::new();
    seal(&rig, &["custom", "openrouter"]);
    open(&mut rig, 0, VoiceTab::Cloud);
    rig.act(VoiceClick::Cloud(true));
    confirm(&mut rig, &ServiceName::Custom);
    rig.act(VoiceClick::SetAddress(Redacted::new(ADDRESS.into())));
    rig.act(VoiceClick::SetModel(Redacted::new(MODEL.into())));
    assert_eq!(use_first(&mut rig, 0, &ServiceName::Custom), MODEL);
    confirm(&mut rig, &named("OpenRouter"));
    rig.act(VoiceClick::KeepNothing(false));
    off(&mut rig, &ServiceName::Custom);
    off(&mut rig, &named("OpenRouter"));
    let mut fresh = restart(&rig);
    open(&mut fresh, 0, VoiceTab::Cloud);
    assert_eq!(tick(&mut fresh, &ServiceName::Custom), ServiceMark::Off);
    let waiting = (MODEL.to_owned(), ServiceMark::Off);
    assert_eq!(cloud_row(&mut fresh, 0, &ServiceName::Custom), waiting);
    confirm(&mut fresh, &ServiceName::Custom);
    let page = lines(&fresh);
    let address = page.address.map(|a| a.expose().clone());
    assert_eq!(
        (address.as_deref(), page.model.expose().as_str()),
        (Some(ADDRESS), MODEL)
    );
    cloud_row(&mut fresh, 0, &ServiceName::Custom);
    assert_eq!(used(&fresh).as_deref(), Some(MODEL), "its pick is back");
    confirm(&mut fresh, &named("OpenRouter"));
    assert_eq!(lines(&fresh).keep, Some(false), "its switch is back");
}
