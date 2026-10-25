//! What the app ships: every module's feature is on by default, CI lints the defaults without the scroll module, the
//! pointer hook, the scroll target, the power dialog, the box watch and the sealed store each reach only the modules
//! whose manifest declares their capability, `KeyboardUi` goes only to the ui and its sources, `ReadFocusedText` only
//! to the modules that read typed text, and the shortcuts reach the keyboard's layer.

use crate::rig::Rig;
use keyxtend::boot::{modules, policy};
use kx_kernel::Kernel;
use kx_module_api::ports::{
    FaceAction, FieldWatch, FieldWatchKey, KeyId, KeyName, PointerHook, PointerHookKey,
    PowerDialog, PowerDialogKey, ScrollTarget, ScrollTargetKey, SealedStore, SealedStoreKey,
    ShortcutId, ShortcutInfo, Step,
};
use kx_module_api::{
    Capability, Manifest, Module, ModuleCx, ModuleError, ModuleId, ModuleState, ServiceKey,
};
use kx_platform_fake::{
    FakeFieldWatch, FakePointerHook, FakePowerDialog, FakeScrollTarget, FakeSealedStore,
};
use std::marker::PhantomData;
use std::sync::Arc;

mod ci;

/// The app's manifest, whose features pick the modules.
const CARGO: &str = include_str!("../../Cargo.toml");
/// How each module's feature begins.
const MODULE_FEATURE: &str = "mod-";
/// The scroll module's feature.
const SCROLL_FEATURE: &str = "mod-scroll";
/// The power module's feature.
const POWER_FEATURE: &str = "mod-power";
/// The prediction module's feature.
const PREDICT_FEATURE: &str = "mod-predict";
/// The CI workflow, whose scroll-off lint names the app's other features by hand.
const CI: &str = include_str!("../../../../.github/workflows/ci.yml");
/// The app's package, which that lint builds.
const APP: &str = env!("CARGO_PKG_NAME");
/// The probes' version, which nothing reads.
const VERSION: &str = "0.0.0";

/// A module under a shipped module's id that declares `K`'s capability and asks for `K` at start.
struct Probe<K> {
    manifest: &'static Manifest,
    key: PhantomData<fn() -> K>,
}

impl<K: ServiceKey> Probe<K> {
    /// A probe named `id`; its manifest lives as long as the test.
    fn new(id: ModuleId) -> Self {
        let caps: Vec<_> = K::CAPABILITY.into_iter().collect();
        let manifest = Manifest {
            id,
            version: VERSION,
            requires: Box::leak(Box::new([K::ID])),
            provides: &[],
            capabilities: Box::leak(caps.into_boxed_slice()),
            settings: None,
        };
        Self {
            manifest: Box::leak(Box::new(manifest)),
            key: PhantomData,
        }
    }
}

impl<K: ServiceKey> Module for Probe<K> {
    fn manifest(&self) -> &'static Manifest {
        self.manifest
    }

    fn start(&mut self, cx: &mut ModuleCx<'_>) -> Result<(), ModuleError> {
        cx.service::<K>()?;
        Ok(())
    }
}

/// The shipped modules under whose id a probe gets `service` from the platform, with the app's grants.
fn reached<K: ServiceKey>(rig: &Rig, service: &Arc<K::Api>) -> Vec<ModuleId> {
    let shipped = modules(&rig.load().hub);
    let grants = policy(&shipped);
    let gets = |id: ModuleId| {
        let mut kernel = Kernel::new(rig.platform(), grants.clone());
        kernel.provide_platform::<K>(Arc::clone(service)).unwrap();
        kernel.add(Box::new(Probe::<K>::new(id))).unwrap();
        kernel.load().unwrap();
        kernel.start().unwrap();
        kernel.state(id) == Some(ModuleState::Active)
    };
    let ids = shipped.iter().map(|m| m.manifest().id);
    ids.filter(|&id| gets(id)).collect()
}

/// The app's `[features]` table.
fn features() -> toml::Table {
    let cargo: toml::Table = CARGO.parse().unwrap();
    cargo["features"].as_table().unwrap().clone()
}

/// The features `features` turns on by default, sorted.
fn defaults(features: &toml::Table) -> Vec<&str> {
    let default = features["default"].as_array().unwrap();
    let mut on: Vec<&str> = default.iter().filter_map(toml::Value::as_str).collect();
    on.sort_unstable();
    on
}

#[test]
fn every_module_feature_is_on_by_default_the_scroll_and_power_ones_too() {
    let features = features();
    let on = defaults(&features);
    let picks: Vec<&str> = features
        .keys()
        .map(String::as_str)
        .filter(|f| f.starts_with(MODULE_FEATURE))
        .collect();
    assert!(picks.iter().all(|f| on.contains(f)), "{on:?}");
    assert!(picks.contains(&SCROLL_FEATURE), "{picks:?}");
    assert!(picks.contains(&POWER_FEATURE), "{picks:?}");
}

#[test]
fn mod_predict_is_a_default_feature_and_the_default_build_ships_predict() {
    assert!(defaults(&features()).contains(&PREDICT_FEATURE));
    let shipped = modules(&Rig::new().load().hub);
    assert!(
        shipped
            .iter()
            .any(|m| m.manifest().id == kx_mod_predict::ID)
    );
}

#[test]
fn the_scroll_off_lint_lists_the_defaults_without_scroll() {
    let features = features();
    let mut want = defaults(&features);
    want.retain(|f| *f != SCROLL_FEATURE);
    let lists = ci::lists(CI, APP);
    assert!(
        !lists.is_empty(),
        "ci.yml has no step that builds {APP} with its default features off"
    );
    let why = "ci.yml's scroll-off step must list the app's default features without";
    for listed in lists {
        assert_eq!(listed, want, "{why} {SCROLL_FEATURE}");
    }
}

#[test]
fn the_scroll_target_reaches_only_the_scroll_module() {
    let target: Arc<dyn ScrollTarget> = Arc::new(FakeScrollTarget::new());
    let got = reached::<ScrollTargetKey>(&Rig::new(), &target);
    assert_eq!(got, [kx_mod_scroll::ID]);
}

#[test]
fn the_power_dialog_reaches_only_the_power_module() {
    let dialog: Arc<dyn PowerDialog> = Arc::new(FakePowerDialog::new());
    let got = reached::<PowerDialogKey>(&Rig::new(), &dialog);
    assert_eq!(got, [kx_mod_power::ID]);
}

#[test]
fn the_box_watch_reaches_only_the_modules_that_read_typed_text() {
    let watch: Arc<dyn FieldWatch> = Arc::new(FakeFieldWatch::new());
    let got = reached::<FieldWatchKey>(&Rig::new(), &watch);
    assert_eq!(got, [kx_mod_keyboard::ID, kx_mod_predict::ID], "ADR-0033");
}

#[test]
fn the_sealed_store_reaches_only_the_predict_module() {
    let store: Arc<dyn SealedStore> = Arc::new(FakeSealedStore::new());
    let got = reached::<SealedStoreKey>(&Rig::new(), &store);
    assert_eq!(got, [kx_mod_predict::ID], "ADR-0033");
}

#[test]
fn the_mouse_hook_still_reaches_only_the_mouse_module() {
    let hook: Arc<dyn PointerHook> = Arc::new(FakePointerHook::new());
    let got = reached::<PointerHookKey>(&Rig::new(), &hook);
    assert_eq!(got, [kx_mod_mouse::ID]);
}

#[test]
fn keyboard_ui_goes_only_to_the_ui_the_keyboard_and_the_mouse_module() {
    let shipped = modules(&Rig::new().load().hub);
    let holders: Vec<_> = shipped
        .iter()
        .map(|m| m.manifest())
        .filter(|m| m.capabilities.contains(&Capability::KeyboardUi))
        .map(|m| m.id)
        .collect();
    let want = [kx_mod_keyboard::ID, kx_mod_mouse::ID, kx_ui::ID];
    assert_eq!(
        holders, want,
        "ADR-0033: the ui, the face's provider and the notes' sender"
    );
}

#[test]
fn read_focused_text_goes_only_to_the_keyboard_and_predict_among_the_modules_shipped_now() {
    let shipped = modules(&Rig::new().load().hub);
    let holders: Vec<_> = shipped
        .iter()
        .map(|m| m.manifest())
        .filter(|m| m.capabilities.contains(&Capability::ReadFocusedText))
        .map(|m| m.id)
        .collect();
    assert_eq!(
        holders,
        [kx_mod_keyboard::ID, kx_mod_predict::ID],
        "ADR-0033: the typist and prediction; the ui joins when it ships"
    );
}

#[test]
fn the_shortcuts_reach_the_keyboard_whose_layer_sends_them() {
    let rig = Rig::new();
    let copy = ShortcutId::new("copy").unwrap();
    let name = |n: &str| KeyName::new(n).unwrap();
    let steps = vec![Step {
        held: vec![name("ctrl")],
        key: name("c"),
    }];
    let info = ShortcutInfo {
        id: copy.clone(),
        steps,
    };
    rig.shortcuts.set_known(vec![info]);
    let mut boot = rig.load();
    boot.start().unwrap();
    for id in ["shortcuts", "sc-copy"] {
        let _ = boot.hub.act(FaceAction::Tap {
            id: KeyId::new(id),
            back: false,
        });
    }
    assert_eq!(rig.shortcuts.sent(), [(copy, Vec::new())]);
}
