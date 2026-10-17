//! What the app ships: every module's feature is on by default, CI lints the defaults without the scroll module, and
//! the pointer hook and the scroll target each reach only the module whose manifest declares their capability.

use crate::rig::Rig;
use keyxtend::boot::{modules, policy};
use kx_kernel::Kernel;
use kx_module_api::ports::{PointerHook, PointerHookKey, ScrollTarget, ScrollTargetKey};
use kx_module_api::{Manifest, Module, ModuleCx, ModuleError, ModuleId, ModuleState, ServiceKey};
use kx_platform_fake::{FakePointerHook, FakeScrollTarget};
use std::marker::PhantomData;
use std::sync::Arc;

/// The app's manifest, whose features pick the modules.
const CARGO: &str = include_str!("../../Cargo.toml");
/// How each module's feature begins.
const MODULE_FEATURE: &str = "mod-";
/// The scroll module's feature.
const SCROLL_FEATURE: &str = "mod-scroll";
/// The CI workflow, whose scroll-off lint names the app's other features by hand.
const CI: &str = include_str!("../../../../.github/workflows/ci.yml");
/// What marks the scroll-off lint's line in `CI`.
const SCROLL_OFF: &str = "-p keyxtend --all-targets --no-default-features";
/// The flag before that lint's feature list.
const FEATURES_FLAG: &str = "--features";
/// What separates the features in that list.
const FEATURE_SEP: char = ',';
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

/// The features each scroll-off lint line in `CI` lists, each list sorted.
fn scroll_off_lists() -> Vec<Vec<&'static str>> {
    let lines = CI.lines().filter(|l| l.contains(SCROLL_OFF));
    let list = |line: &'static str| {
        let mut words = line.split_whitespace().skip_while(|w| *w != FEATURES_FLAG);
        let mut listed: Vec<&str> = words
            .nth(1)
            .unwrap_or_default()
            .split(FEATURE_SEP)
            .collect();
        listed.sort_unstable();
        listed
    };
    lines.map(list).collect()
}

#[test]
fn every_module_feature_is_on_by_default_the_scroll_one_too() {
    let features = features();
    let on = defaults(&features);
    let picks: Vec<&str> = features
        .keys()
        .map(String::as_str)
        .filter(|f| f.starts_with(MODULE_FEATURE))
        .collect();
    assert!(picks.iter().all(|f| on.contains(f)), "{on:?}");
    assert!(picks.contains(&SCROLL_FEATURE), "{picks:?}");
}

#[test]
fn the_scroll_off_lint_lists_the_defaults_without_scroll() {
    let features = features();
    let mut want = defaults(&features);
    want.retain(|f| *f != SCROLL_FEATURE);
    let lists = scroll_off_lists();
    assert!(
        !lists.is_empty(),
        "ci.yml lints the app with the scroll module off"
    );
    assert!(lists.iter().all(|l| *l == want), "{lists:?} != {want:?}");
}

#[test]
fn the_scroll_target_reaches_only_the_scroll_module() {
    let target: Arc<dyn ScrollTarget> = Arc::new(FakeScrollTarget::new());
    let got = reached::<ScrollTargetKey>(&Rig::new(), &target);
    assert_eq!(got, [kx_mod_scroll::ID]);
}

#[test]
fn the_mouse_hook_still_reaches_only_the_mouse_module() {
    let hook: Arc<dyn PointerHook> = Arc::new(FakePointerHook::new());
    let got = reached::<PointerHookKey>(&Rig::new(), &hook);
    assert_eq!(got, [kx_mod_mouse::ID]);
}
