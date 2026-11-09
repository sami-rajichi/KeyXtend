//! The selection chips' grants (ADR-0041 decision 3): its feature is a default and the build ships it, it holds
//! `ReadFocusedText`, `Selection` and `OpenLink`, the keyboard and the ui hold `Selection` as an event gate, and it
//! alone asks for the watch and the link, which reach only their gates' holders.

use super::{asking, defaults, features, holders, reached};
use crate::rig::{OS, Rig};
use keyxtend::boot::modules;
use kx_module_api::ports::{SelectionWatch, SelectionWatchKey, WebLink, WebLinkKey};
use kx_module_api::{Capability, ModuleId, ServiceKey};
use kx_platform_fake::{FakeSelectionWatch, FakeWebLink};
use std::collections::BTreeSet;
use std::sync::Arc;

/// The selection module's feature.
const SEL_FEATURE: &str = "mod-selection";

#[test]
fn mod_selection_is_a_default_feature_and_the_default_build_ships_it() {
    assert!(defaults(&features()).contains(&SEL_FEATURE));
    let shipped = modules(&Rig::new().load().hub, OS);
    let ids: Vec<ModuleId> = shipped.iter().map(|m| m.manifest().id).collect();
    assert!(ids.contains(&kx_mod_selection::ID), "{ids:?}");
}

#[test]
fn the_selection_module_holds_read_focused_text_selection_and_open_link() {
    let want = [
        Capability::ReadFocusedText,
        Capability::Selection,
        Capability::OpenLink,
    ];
    assert_eq!(kx_mod_selection::MANIFEST.capabilities, want);
}

#[test]
fn selection_goes_to_the_module_and_as_an_event_gate_to_the_keyboard_and_the_ui() {
    let want = BTreeSet::from([kx_mod_selection::ID, kx_mod_keyboard::ID, kx_ui::ID]);
    assert_eq!(holders(Capability::Selection), want, "ADR-0041 decision 3");
}

#[test]
fn only_the_selection_module_holds_open_link() {
    let want = BTreeSet::from([kx_mod_selection::ID]);
    assert_eq!(holders(Capability::OpenLink), want, "ADR-0041 decision 3");
}

#[test]
fn only_the_selection_module_asks_for_the_watch_or_the_link() {
    let keys = [SelectionWatchKey::ID, WebLinkKey::ID];
    assert_eq!(asking(&keys), BTreeSet::from([kx_mod_selection::ID]));
}

#[test]
fn the_web_link_reaches_the_selection_module_alone() {
    let link: Arc<dyn WebLink> = Arc::new(FakeWebLink::new());
    let got = reached::<WebLinkKey>(&Rig::new(), &link);
    assert_eq!(got, BTreeSet::from([kx_mod_selection::ID]));
}

#[test]
fn the_selection_watch_reaches_only_the_read_focused_text_holders() {
    let watch: Arc<dyn SelectionWatch> = Arc::new(FakeSelectionWatch::new());
    let got = reached::<SelectionWatchKey>(&Rig::new(), &watch);
    assert_eq!(got, holders(Capability::ReadFocusedText), "its gate");
    assert!(got.contains(&kx_mod_selection::ID), "{got:?}");
}
