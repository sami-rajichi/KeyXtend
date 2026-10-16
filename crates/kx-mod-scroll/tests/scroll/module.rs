//! The module's start through the shared fake host, which gates services as the kernel does.

use kx_mod_scroll::{MANIFEST, ScrollModule};
use kx_module_api::ports::{ScrollConfig, ScrollError, ScrollRun, ScrollTarget, ScrollTargetKey};
use kx_module_api::{
    Capability, Module, ModuleCx, ModuleError, ServiceError, ServiceKey, SettingsError,
};
use kx_test_support::host::{FakeHost, Stored};
use std::sync::Arc;

/// A target that never opens a run.
struct Shut;

impl ScrollTarget for Shut {
    fn open(&self, _: ScrollConfig) -> Result<Box<dyn ScrollRun>, ScrollError> {
        Err(ScrollError::Refused)
    }
}

/// A host on the defaults granting `granted`, holding `target` if any.
fn host(granted: &[Capability], target: Option<Arc<dyn ScrollTarget>>) -> FakeHost {
    let mut host = FakeHost::new(&MANIFEST);
    host.granted = granted.to_vec();
    if let Some(target) = target {
        let stored: Stored = (ScrollTargetKey::CAPABILITY, Box::new(target));
        host.services.insert(ScrollTargetKey::ID, stored);
    }
    host
}

/// A host granting `granted`, holding a target that never opens.
fn shut(granted: &[Capability]) -> FakeHost {
    host(granted, Some(Arc::new(Shut)))
}

/// Starts `module` on `host`.
fn start(module: &mut ScrollModule, host: &mut FakeHost) -> Result<(), ModuleError> {
    module.start(&mut ModuleCx::new(host))
}

/// Everything the manifest declares.
const ALL: [Capability; 1] = [Capability::Scroll];

#[test]
fn it_starts_on_the_defaults_with_the_target_granted() {
    let mut module = ScrollModule::default();
    assert_eq!(start(&mut module, &mut shut(&ALL)), Ok(()));
}

#[test]
fn without_the_scroll_grant_it_cannot_reach_the_target() {
    let refused = ServiceError::NotGranted(ScrollTargetKey::ID, Capability::Scroll);
    let mut module = ScrollModule::default();
    let got = start(&mut module, &mut shut(&[]));
    assert_eq!(got, Err(ModuleError::Service(refused)));
}

#[test]
fn without_a_target_it_fails_to_start() {
    let missing = ServiceError::Missing(ScrollTargetKey::ID);
    let mut module = ScrollModule::default();
    let got = start(&mut module, &mut host(&ALL, None));
    assert_eq!(got, Err(ModuleError::Service(missing)));
}

#[test]
fn a_bad_value_fails_the_start() {
    let mut host = shut(&ALL);
    host.put("fast", 2.into());
    let err = start(&mut ScrollModule::default(), &mut host);
    assert!(
        matches!(err, Err(ModuleError::Settings(SettingsError::Invalid(_)))),
        "{err:?}"
    );
}

#[test]
fn a_stop_lets_go_of_the_target_and_it_starts_again() {
    let target: Arc<dyn ScrollTarget> = Arc::new(Shut);
    let mut host = host(&ALL, Some(Arc::clone(&target)));
    let unheld = Arc::strong_count(&target);
    let mut module = ScrollModule::default();
    start(&mut module, &mut host).unwrap();
    assert!(Arc::strong_count(&target) > unheld, "the module holds it");
    module.stop();
    assert_eq!(Arc::strong_count(&target), unheld, "the stop let go");
    assert_eq!(start(&mut module, &mut host), Ok(()));
}
