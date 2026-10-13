//! The module's start through the shared fake host, which gates services as the kernel does.

use kx_mod_mouse::{MANIFEST, MouseModule};
use kx_module_api::ports::{
    HookConfig, HookError, HookRun, PointerEngine, PointerHook, PointerHookKey,
};
use kx_module_api::{
    Capability, Module, ModuleCx, ModuleError, ServiceError, ServiceKey, SettingsError,
};
use kx_test_support::host::{FakeHost, Stored};
use std::sync::Arc;

/// A hook that never starts; the module only keeps it at start.
struct Idle;

impl PointerHook for Idle {
    fn start(
        &self,
        _: Box<dyn PointerEngine>,
        _: HookConfig,
    ) -> Result<Box<dyn HookRun>, HookError> {
        Err(HookError::Thread)
    }
}

/// A host on the defaults granting `granted`, holding a hook when `with_hook`.
fn host(granted: &[Capability], with_hook: bool) -> FakeHost {
    let mut host = FakeHost::new(&MANIFEST);
    host.granted = granted.to_vec();
    if with_hook {
        let hook: Arc<dyn PointerHook> = Arc::new(Idle);
        let stored: Stored = (PointerHookKey::CAPABILITY, Box::new(hook));
        host.services.insert(PointerHookKey::ID, stored);
    }
    host
}

/// Starts the module on `host`.
fn start(host: &mut FakeHost) -> Result<(), ModuleError> {
    MouseModule::default().start(&mut ModuleCx::new(host))
}

/// Everything the manifest declares.
const ALL: [Capability; 2] = [Capability::PointerHook, Capability::InjectInput];

#[test]
fn it_starts_on_the_defaults_with_the_hook_granted() {
    assert_eq!(start(&mut host(&ALL, true)), Ok(()));
}

#[test]
fn without_the_pointer_hook_grant_it_cannot_reach_the_hook() {
    let mut host = host(&[Capability::InjectInput], true);
    let refused = ServiceError::NotGranted(PointerHookKey::ID, Capability::PointerHook);
    assert_eq!(start(&mut host), Err(ModuleError::Service(refused)));
}

#[test]
fn without_a_hook_it_fails_to_start() {
    let missing = ServiceError::Missing(PointerHookKey::ID);
    let refused = Err(ModuleError::Service(missing));
    assert_eq!(start(&mut host(&ALL, false)), refused);
}

#[test]
fn a_bad_value_fails_the_start() {
    let mut host = host(&ALL, true);
    host.put("hold_ms", 9_000.into());
    let err = start(&mut host);
    assert!(
        matches!(err, Err(ModuleError::Settings(SettingsError::Invalid(_)))),
        "{err:?}"
    );
}
