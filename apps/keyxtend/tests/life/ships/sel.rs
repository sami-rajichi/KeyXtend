//! The selection chips' grants (ADR-0041 decision 3): its feature is a default and the build ships it, it holds
//! `ReadFocusedText`, `Selection` and `OpenLink`, the keyboard and the ui hold `Selection` as an event gate, it alone
//! asks for the watch, and the link goes to it and to voice typing alone (ADR-0047 decision 11), reaching only them.

use super::{asking, defaults, features, holders, reached};
use crate::rig::{OS, Rig};
use keyxtend::boot::modules;
use keyxtend::hidden::Run;
use kx_module_api::ports::{SelectionWatch, SelectionWatchKey, WebLink, WebLinkKey};
use kx_module_api::{Capability, ModuleId, ServiceKey};
use kx_platform_fake::{FakeSelectionWatch, FakeWebLink};
use std::collections::BTreeSet;
use std::sync::Arc;

/// The selection module's feature.
const SEL_FEATURE: &str = "mod-selection";

/// The modules the link goes to: the selection chips' Translate and a cloud card's link.
fn linkers() -> BTreeSet<ModuleId> {
    BTreeSet::from([kx_mod_selection::ID, kx_mod_voice::ID])
}

#[test]
fn mod_selection_is_a_default_feature_and_the_default_build_ships_it() {
    assert!(defaults(&features()).contains(&SEL_FEATURE));
    let shipped = modules(&Rig::new().load().hub, OS, &Run::Normal);
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
fn only_the_selection_and_voice_modules_hold_open_link() {
    let why = "ADR-0041 decision 3, ADR-0047 decision 11";
    assert_eq!(holders(Capability::OpenLink), linkers(), "{why}");
}

#[test]
fn only_the_selection_module_asks_for_the_watch() {
    let watch = asking(&[SelectionWatchKey::ID]);
    assert_eq!(watch, BTreeSet::from([kx_mod_selection::ID]));
}

#[test]
fn only_the_selection_and_voice_modules_ask_for_the_link() {
    assert_eq!(asking(&[WebLinkKey::ID]), linkers(), "ADR-0047 decision 11");
}

#[test]
fn the_web_link_reaches_the_selection_and_voice_modules_alone() {
    let link: Arc<dyn WebLink> = Arc::new(FakeWebLink::new());
    let got = reached::<WebLinkKey>(&Rig::new(), &link);
    assert_eq!(got, linkers());
}

#[test]
fn the_selection_watch_reaches_only_the_read_focused_text_holders() {
    let watch: Arc<dyn SelectionWatch> = Arc::new(FakeSelectionWatch::new());
    let got = reached::<SelectionWatchKey>(&Rig::new(), &watch);
    assert_eq!(got, holders(Capability::ReadFocusedText), "its gate");
    assert!(got.contains(&kx_mod_selection::ID), "{got:?}");
}
