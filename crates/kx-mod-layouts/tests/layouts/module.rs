//! The module's start through the shared fake host, which gates services as the kernel does.

#![allow(
    clippy::unwrap_used,
    reason = "these fixtures run only in tests, where a failed setup should fail the test"
)]

use crate::common::{Heard, ids, source};
use kx_mod_layouts::{LayoutsModule, MANIFEST};
use kx_module_api::Capability::InjectInput;
use kx_module_api::ports::{LayoutSource, LayoutSourceKey, Layouts, LayoutsKey};
use kx_module_api::{
    Capability, Module, ModuleCx, ModuleError, ServiceError, ServiceKey, SettingsError,
};
use kx_test_support::host::{FakeHost, Stored};
use std::sync::Arc;

/// A host granting `granted`, holding the golden source when `with_source`, and what its bus heard.
fn host(granted: &[Capability], with_source: bool) -> (FakeHost, Arc<Heard>) {
    let mut host = FakeHost::new(&MANIFEST);
    host.granted = granted.to_vec();
    if with_source {
        let source: Arc<dyn LayoutSource> = source();
        let stored: Stored = (LayoutSourceKey::CAPABILITY, Box::new(source));
        host.services.insert(LayoutSourceKey::ID, stored);
    }
    let heard = Arc::new(Heard::default());
    host.core = heard.clone();
    (host, heard)
}

/// Starts the module on `host`.
fn start(host: &mut FakeHost) -> Result<(), ModuleError> {
    LayoutsModule.start(&mut ModuleCx::new(host))
}

/// The layouts service the module provided.
fn layouts(host: &FakeHost) -> Arc<dyn Layouts> {
    let (_, stored) = &host.services[&LayoutsKey::ID];
    stored.downcast_ref::<Arc<dyn Layouts>>().unwrap().clone()
}

#[test]
fn it_provides_the_layouts_of_the_app_in_front() {
    let [en, fr, _] = ids();
    let (mut host, heard) = host(&[InjectInput], true);
    assert_eq!(start(&mut host), Ok(()));
    let layouts = layouts(&host);
    assert_eq!(layouts.current().label.id, en);
    let shorts: Vec<String> = layouts.list().into_iter().map(|l| l.short).collect();
    assert_eq!(shorts, ["EN", "FR", "ع"]);
    assert_eq!(layouts.pick(fr), Ok(()));
    assert!(layouts.refresh());
    assert_eq!(heard.ids(), [fr], "the change goes out on the bus");
}

#[test]
fn the_short_setting_names_the_language_key() {
    let (mut host, _) = host(&[InjectInput], true);
    host.settings = "short = { fr = \"Fr\" }".parse().unwrap();
    assert_eq!(start(&mut host), Ok(()));
    let shorts: Vec<String> = layouts(&host).list().into_iter().map(|l| l.short).collect();
    assert_eq!(
        shorts,
        ["EN", "Fr", "AR"],
        "a user's map replaces the default one"
    );
}

#[test]
fn without_inject_input_it_cannot_read_the_source() {
    let (mut host, _) = host(&[], true);
    let refused = ServiceError::NotGranted(LayoutSourceKey::ID, InjectInput);
    assert_eq!(start(&mut host), Err(ModuleError::Service(refused)));
    assert!(!host.services.contains_key(&LayoutsKey::ID));
}

#[test]
fn without_a_source_it_fails_to_start() {
    let (mut host, _) = host(&[InjectInput], false);
    let missing = ServiceError::Missing(LayoutSourceKey::ID);
    assert_eq!(start(&mut host), Err(ModuleError::Service(missing)));
}

#[test]
fn bad_settings_fail_the_start_instead_of_a_panic() {
    let (mut host, _) = host(&[InjectInput], true);
    host.settings = "short = { AR = \"x\" }".parse().unwrap();
    let err = start(&mut host).unwrap_err();
    assert!(
        matches!(err, ModuleError::Settings(SettingsError::Invalid(_))),
        "{err}"
    );
    assert!(!host.services.contains_key(&LayoutsKey::ID));
}
