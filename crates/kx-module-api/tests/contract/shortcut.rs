//! The shortcuts port: its id, the `InjectInput` gate, and sends reached through it on the shared fake, held keys
//! and all.

use kx_module_api::Capability::{InjectInput, Scroll};
use kx_module_api::ports::{
    KeyName, Scan, ShortcutError, ShortcutId, ShortcutInfo, Shortcuts, ShortcutsKey, Step,
};
use kx_module_api::{Capability, Manifest, ModuleCx, ModuleId, ServiceError, ServiceKey};
use kx_platform_fake::FakeShortcuts;
use kx_test_support::host::FakeHost;
use std::sync::Arc;

/// A keyboard declaring `InjectInput`, and `Scroll` to show that another grant does not open the gate.
static KEYBOARD: Manifest = Manifest {
    id: ModuleId::new("keyboard"),
    version: env!("CARGO_PKG_VERSION"),
    requires: &[ShortcutsKey::ID],
    provides: &[],
    capabilities: &[InjectInput, Scroll],
    settings: None,
};

/// Left Ctrl, latched around a send.
const CTRL: Scan = Scan::new(0x1D);

/// `copy` as one Ctrl+C step.
fn copy() -> Option<ShortcutInfo> {
    let step = Step {
        held: vec![KeyName::new("ctrl")?],
        key: KeyName::new("c")?,
    };
    let id = ShortcutId::new("copy")?;
    Some(ShortcutInfo {
        id,
        steps: vec![step],
    })
}

/// A keyboard host granted `granted`, holding the shared fake as the platform would provide it.
fn host(granted: &[Capability]) -> (FakeHost, FakeShortcuts) {
    let fake = FakeShortcuts::new();
    let api: Arc<dyn Shortcuts> = Arc::new(fake.clone());
    let mut host = FakeHost::new(&KEYBOARD);
    host.granted.extend(granted);
    let stored = (ShortcutsKey::CAPABILITY, Box::new(api) as _);
    host.services.insert(ShortcutsKey::ID, stored);
    (host, fake)
}

#[test]
fn a_module_without_inject_input_cannot_get_the_shortcuts() {
    let (mut host, _fake) = host(&[Scroll]);
    let cx = ModuleCx::new(&mut host);
    let refused = ServiceError::NotGranted(ShortcutsKey::ID, InjectInput);
    assert_eq!(cx.service::<ShortcutsKey>().err(), Some(refused));
}

#[test]
fn sends_reached_through_the_gate_carry_the_held_keys() {
    let (mut host, fake) = host(&[InjectInput]);
    let info = copy().unwrap();
    fake.set_known(vec![info.clone()]);
    let shortcuts = ModuleCx::new(&mut host).service::<ShortcutsKey>().unwrap();
    assert_eq!(shortcuts.known(), std::slice::from_ref(&info));
    assert_eq!(shortcuts.send(&info.id, &[CTRL]), Ok(()));
    let paste = ShortcutId::new("paste").unwrap();
    assert_eq!(shortcuts.send(&paste, &[]), Err(ShortcutError::Unknown));
    assert_eq!(fake.sent(), [(info.id, vec![CTRL])]);
}
