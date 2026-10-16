//! What the app ships: every module's feature is on by default, and each gated platform service reaches only the
//! module whose manifest declares its capability.

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

#[test]
fn every_module_feature_is_on_by_default_the_scroll_one_too() {
    let cargo: toml::Table = CARGO.parse().unwrap();
    let features = cargo["features"].as_table().unwrap();
    let default = features["default"].as_array().unwrap();
    let on = |f: &str| default.iter().any(|d| d.as_str() == Some(f));
    let picks: Vec<&String> = features
        .keys()
        .filter(|f| f.starts_with(MODULE_FEATURE))
        .collect();
    assert!(picks.iter().all(|f| on(f)), "{default:?}");
    assert!(picks.iter().any(|f| *f == SCROLL_FEATURE), "{picks:?}");
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
