//! Quick-fill's grants (ADR-0038, ADR-0039): its feature is a default and the build ships it, it holds `Vault`,
//! `VaultData` and `ReadClipboard`, the keyboard, prediction and the ui hold `Vault` as an event gate only, the notebook
//! store, the boxes and the places reach the vault alone, and random bytes and the calendar need no gate.

use super::{asking, defaults, features, holders, reached};
use crate::rig::{OS, Rig};
use keyxtend::boot::modules;
use kx_crypto::MemoryStore;
use kx_module_api::ports::{
    Boxes, BoxesKey, Calendar, CalendarKey, Places, PlacesKey, Random, RandomKey, SealedStore,
    VaultStoreKey,
};
use kx_module_api::{Capability, ModuleId, ServiceKey};
use kx_platform_fake::{FakeBoxes, FakePlaces, FakeRandom};
use std::collections::BTreeSet;
use std::sync::Arc;

/// The vault module's feature.
const VAULT_FEATURE: &str = "mod-vault";
/// The seed of the probes' random bytes.
const SEED: u64 = 5;
/// The modules that hold `Vault` as an event gate only.
const GATES: [ModuleId; 3] = [kx_mod_keyboard::ID, kx_mod_predict::ID, kx_ui::ID];

/// Every shipped module's id.
fn shipped() -> BTreeSet<ModuleId> {
    let shipped = modules(&Rig::new().load().hub, OS);
    shipped.iter().map(|m| m.manifest().id).collect()
}

#[test]
fn mod_vault_is_a_default_feature_and_the_default_build_ships_it() {
    assert!(defaults(&features()).contains(&VAULT_FEATURE));
    assert!(shipped().contains(&kx_mod_vault::ID));
}

#[test]
fn the_vault_holds_vault_vault_data_and_read_clipboard() {
    let want = [
        Capability::Vault,
        Capability::VaultData,
        Capability::ReadClipboard,
    ];
    assert_eq!(kx_mod_vault::MANIFEST.capabilities, want);
}

#[test]
fn vault_goes_to_the_vault_and_as_an_event_gate_to_the_keyboard_prediction_and_the_ui() {
    let want: BTreeSet<ModuleId> = GATES.into_iter().chain([kx_mod_vault::ID]).collect();
    assert_eq!(holders(Capability::Vault), want, "ADR-0038 decision 3");
}

#[test]
fn only_the_vault_holds_vault_data() {
    let want = BTreeSet::from([kx_mod_vault::ID]);
    assert_eq!(holders(Capability::VaultData), want, "ADR-0039");
}

/// Checks that `got`, the modules a service reached, is the vault alone: the event gates are refused it.
fn vault_alone(got: &BTreeSet<ModuleId>) {
    assert_eq!(got, &BTreeSet::from([kx_mod_vault::ID]), "ADR-0039");
    assert!(GATES.iter().all(|id| !got.contains(id)), "{got:?}");
}

#[test]
fn the_notebook_store_reaches_the_vault_alone() {
    let store: Arc<dyn SealedStore> = Arc::new(MemoryStore::new());
    vault_alone(&reached::<VaultStoreKey>(&Rig::new(), &store));
}

#[test]
fn the_boxes_and_the_places_reach_the_vault_alone() {
    let boxes: Arc<dyn Boxes> = Arc::new(FakeBoxes::new());
    let places: Arc<dyn Places> = Arc::new(FakePlaces::new());
    let rig = Rig::new();
    vault_alone(&reached::<BoxesKey>(&rig, &boxes));
    vault_alone(&reached::<PlacesKey>(&rig, &places));
}

#[test]
fn only_the_vault_asks_for_the_notebook_store_the_boxes_or_the_places() {
    let keys = [VaultStoreKey::ID, BoxesKey::ID, PlacesKey::ID];
    assert_eq!(asking(&keys), BTreeSet::from([kx_mod_vault::ID]));
    let ui = kx_ui::MANIFEST.requires;
    assert!(
        keys.iter().all(|k| !ui.contains(k)),
        "the ui's Vault is an event gate"
    );
}

#[test]
fn random_bytes_and_the_calendar_need_no_capability_and_reach_every_module() {
    assert_eq!(
        (RandomKey::CAPABILITY, CalendarKey::CAPABILITY),
        (None, None)
    );
    let rig = Rig::new();
    let random: Arc<dyn Random> = Arc::new(FakeRandom::new(SEED));
    assert_eq!(reached::<RandomKey>(&rig, &random), shipped());
    let calendar: Arc<dyn Calendar> = Arc::new(rig.calendar.clone());
    assert_eq!(reached::<CalendarKey>(&rig, &calendar), shipped());
}
