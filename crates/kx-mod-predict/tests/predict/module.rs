//! The module's start through the shared fake host, which gates services as the kernel does.

use kx_mod_predict::{MANIFEST, PredictModule};
use kx_module_api::ports::{
    FieldWatch, FieldWatchKey, SealedStore, SealedStoreKey, WordPacks, WordPacksKey,
};
use kx_module_api::{Capability, Module, ModuleCx, ModuleError, ServiceError, ServiceKey};
use kx_platform_fake::{FakeFieldWatch, FakeSealedStore, FakeWordPacks};
use kx_test_support::host::{FakeHost, Stored};
use std::sync::Arc;

/// A host granting `granted`, holding the three fakes.
fn host(granted: &[Capability], field: &FakeFieldWatch) -> FakeHost {
    let mut host = FakeHost::new(&MANIFEST);
    host.granted = granted.to_vec();
    let field: Arc<dyn FieldWatch> = Arc::new(field.clone());
    let packs: Arc<dyn WordPacks> = Arc::new(FakeWordPacks::new());
    let store: Arc<dyn SealedStore> = Arc::new(FakeSealedStore::new());
    let stored: [(_, Stored); 3] = [
        (
            FieldWatchKey::ID,
            (FieldWatchKey::CAPABILITY, Box::new(field)),
        ),
        (
            WordPacksKey::ID,
            (WordPacksKey::CAPABILITY, Box::new(packs)),
        ),
        (
            SealedStoreKey::ID,
            (SealedStoreKey::CAPABILITY, Box::new(store)),
        ),
    ];
    host.services.extend(stored);
    host
}

/// Starts `module` on `host`.
fn start(module: &mut PredictModule, host: &mut FakeHost) -> Result<(), ModuleError> {
    module.start(&mut ModuleCx::new(host))
}

#[test]
fn it_starts_with_its_grants_watches_the_box_with_its_settings_and_lets_go_at_stop() {
    let field = FakeFieldWatch::new();
    let mut host = host(MANIFEST.capabilities, &field);
    let mut module = PredictModule::default();
    assert_eq!(start(&mut module, &mut host), Ok(()));
    assert!(field.watching());
    let settings = kx_mod_predict::PredictSettings::from_defaults().unwrap();
    assert_eq!(field.config(), Some(settings.field_config()));
    host.held.clear();
    module.stop();
    assert!(!field.watching(), "the stop dropped the watch");
    assert_eq!(start(&mut module, &mut host), Ok(()), "it starts again");
    assert_eq!(field.watches(), 2);
}

#[test]
fn without_read_focused_text_it_cannot_watch_the_box() {
    let field = FakeFieldWatch::new();
    let mut host = host(&[Capability::SecretStore], &field);
    let refused = ServiceError::NotGranted(FieldWatchKey::ID, Capability::ReadFocusedText);
    let mut module = PredictModule::default();
    assert_eq!(
        start(&mut module, &mut host),
        Err(ModuleError::Service(refused))
    );
    assert!(!field.watching());
}

#[test]
fn without_secret_store_it_cannot_reach_the_sealed_store() {
    let field = FakeFieldWatch::new();
    let mut host = host(&[Capability::ReadFocusedText], &field);
    let refused = ServiceError::NotGranted(SealedStoreKey::ID, Capability::SecretStore);
    let mut module = PredictModule::default();
    assert_eq!(
        start(&mut module, &mut host),
        Err(ModuleError::Service(refused))
    );
}

#[test]
fn a_watch_that_fails_still_starts_with_no_box_to_read() {
    let field = FakeFieldWatch::new();
    field.fail_watch(kx_module_api::ports::FieldError::Unavailable);
    let mut host = host(MANIFEST.capabilities, &field);
    let mut module = PredictModule::default();
    assert_eq!(start(&mut module, &mut host), Ok(()));
    assert!(!field.watching());
}
