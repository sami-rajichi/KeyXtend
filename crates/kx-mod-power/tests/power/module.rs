//! The module's start through the shared fake host, which gates services as the kernel does.

use kx_mod_power::{MANIFEST, PowerModule};
use kx_module_api::ports::{PowerDialog, PowerDialogKey};
use kx_module_api::{Capability, Module, ModuleCx, ModuleError, ServiceError, ServiceKey};
use kx_platform_fake::FakePowerDialog;
use kx_test_support::host::{FakeHost, Stored};
use std::sync::Arc;

/// A host granting `granted`, holding `dialog` if any.
fn host(granted: &[Capability], dialog: Option<Arc<dyn PowerDialog>>) -> FakeHost {
    let mut host = FakeHost::new(&MANIFEST);
    host.granted = granted.to_vec();
    if let Some(dialog) = dialog {
        let stored: Stored = (PowerDialogKey::CAPABILITY, Box::new(dialog));
        host.services.insert(PowerDialogKey::ID, stored);
    }
    host
}

/// A host granting `granted`, holding a fake dialog.
fn faked(granted: &[Capability]) -> FakeHost {
    host(granted, Some(Arc::new(FakePowerDialog::new())))
}

/// Starts `module` on `host`.
fn start(module: &mut PowerModule, host: &mut FakeHost) -> Result<(), ModuleError> {
    module.start(&mut ModuleCx::new(host))
}

/// Everything the manifest declares.
const ALL: [Capability; 1] = [Capability::Power];

#[test]
fn it_starts_with_the_grant_and_a_dialog() {
    let mut module = PowerModule::default();
    assert_eq!(start(&mut module, &mut faked(&ALL)), Ok(()));
}

#[test]
fn without_the_power_grant_it_cannot_reach_the_dialog() {
    let refused = ServiceError::NotGranted(PowerDialogKey::ID, Capability::Power);
    let mut module = PowerModule::default();
    let got = start(&mut module, &mut faked(&[]));
    assert_eq!(got, Err(ModuleError::Service(refused)));
}

#[test]
fn without_a_dialog_it_fails_to_start() {
    let missing = ServiceError::Missing(PowerDialogKey::ID);
    let mut module = PowerModule::default();
    let got = start(&mut module, &mut host(&ALL, None));
    assert_eq!(got, Err(ModuleError::Service(missing)));
}

#[test]
fn a_stop_lets_go_of_the_dialog_and_it_starts_again() {
    let dialog: Arc<dyn PowerDialog> = Arc::new(FakePowerDialog::new());
    let mut host = host(&ALL, Some(Arc::clone(&dialog)));
    let unheld = Arc::strong_count(&dialog);
    let mut module = PowerModule::default();
    start(&mut module, &mut host).unwrap();
    assert!(Arc::strong_count(&dialog) > unheld, "the module holds it");
    module.stop();
    assert_eq!(Arc::strong_count(&dialog), unheld, "the stop let go");
    assert_eq!(start(&mut module, &mut host), Ok(()));
}
