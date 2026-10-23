//! The power dialog port and the Power key's events: the `Power` gate, the dialog reached through it on the shared
//! fake, and events a module holding only `InjectInput` may send and hear.

use kx_module_api::Capability::{InjectInput, Power};
use kx_module_api::ports::{
    PowerAsk, PowerAsked, PowerChanged, PowerDialog, PowerDialogKey, PowerError,
};
use kx_module_api::{Capability, Host, Manifest, ModuleCx, ModuleId, ServiceError, ServiceKey};
use kx_platform_fake::FakePowerDialog;
use kx_test_support::host::FakeHost;
use std::sync::{Arc, Mutex};

/// A module declaring `Power` and `InjectInput`; each test grants it some, as the kernel's policy would.
static POWER: Manifest = Manifest {
    id: ModuleId::new("power"),
    version: env!("CARGO_PKG_VERSION"),
    requires: &[PowerDialogKey::ID],
    provides: &[],
    capabilities: &[Power, InjectInput],
    settings: None,
};

/// A host granted `granted`, holding the shared fake dialog as the platform would provide it.
fn host(granted: &[Capability]) -> (FakeHost, FakePowerDialog) {
    let fake = FakePowerDialog::new();
    let api: Arc<dyn PowerDialog> = Arc::new(fake.clone());
    let mut host = FakeHost::new(&POWER);
    host.granted.extend(granted);
    let stored = (PowerDialogKey::CAPABILITY, Box::new(api) as _);
    host.services.insert(PowerDialogKey::ID, stored);
    (host, fake)
}

#[test]
fn a_host_without_power_cannot_get_the_dialog() {
    let (mut host, fake) = host(&[InjectInput]);
    let cx = ModuleCx::new(&mut host);
    let refused = ServiceError::NotGranted(PowerDialogKey::ID, Power);
    assert_eq!(cx.service::<PowerDialogKey>().err(), Some(refused));
    assert_eq!(fake.opened(), 0);
}

#[test]
fn the_dialog_reached_through_the_gate_opens_or_says_why_not() {
    let (mut host, fake) = host(&[Power]);
    let dialog = ModuleCx::new(&mut host)
        .service::<PowerDialogKey>()
        .unwrap();
    assert_eq!(dialog.open(), Ok(()));
    fake.set_error(Some(PowerError::Unavailable));
    assert_eq!(dialog.open(), Err(PowerError::Unavailable));
    assert_eq!(fake.opened(), 1);
}

#[test]
fn the_power_events_pass_on_a_bus_with_no_grants() {
    let (host, _fake) = host(&[]);
    let bus = host.bus();
    assert_eq!(bus.publish(&PowerAsked), Ok(()));
    assert_eq!(bus.publish(&PowerChanged::OFF), Ok(()));
    assert_eq!(bus.publish(&PowerAsk), Ok(()));
}

#[test]
fn a_module_holding_only_inject_input_hears_the_power_state() {
    let (host, _fake) = host(&[InjectInput]);
    let seen = Arc::new(Mutex::new(Vec::new()));
    let log = Arc::clone(&seen);
    let _sub = host
        .bus()
        .subscribe(move |p: &PowerChanged| log.lock().unwrap().push(p.live))
        .unwrap();
    host.bus().publish(&PowerChanged { live: true }).unwrap();
    assert_eq!(*seen.lock().unwrap(), [true]);
}
