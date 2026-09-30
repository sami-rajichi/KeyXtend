//! The module's start on its settings, its compiled-in board and the three typing services.

#![allow(
    clippy::unwrap_used,
    reason = "these fixtures run only in tests, where a failed setup should fail the test"
)]

use crate::host::{FakeHost, fake, store};
use kx_mod_keyboard::spec::char_id;
use kx_mod_keyboard::{BoardSpec, ID, KeyboardModule, KeyboardSettings, Ports, Tapped, Typing};
use kx_module_api::Capability::InjectInput;
use kx_module_api::ports::{InputInjectorKey, LayoutsKey, Scan, TextRoute};
use kx_module_api::{
    Module, ModuleCx, ModuleError, Redacted, ServiceError, ServiceKey, SettingsError,
};
use kx_platform_fake::Stroke;

fn start(host: &mut FakeHost) -> Result<(), ModuleError> {
    KeyboardModule.start(&mut ModuleCx::new(host))
}

#[test]
fn it_starts_on_the_defaults_the_shipped_board_and_the_typing_services() {
    let mut host = fake(None);
    store(&mut host, None);
    host.granted = vec![InjectInput];
    assert_eq!(start(&mut host), Ok(()));
}

#[test]
fn without_inject_input_it_cannot_reach_the_injector() {
    let mut host = fake(None);
    store(&mut host, None);
    let refused = ServiceError::NotGranted(InputInjectorKey::ID, InjectInput);
    assert_eq!(start(&mut host), Err(ModuleError::Service(refused)));
}

#[test]
fn without_the_layouts_service_it_fails_to_start() {
    let mut host = fake(None);
    store(&mut host, Some(LayoutsKey::ID));
    host.granted = vec![InjectInput];
    let missing = ServiceError::Missing(LayoutsKey::ID);
    assert_eq!(start(&mut host), Err(ModuleError::Service(missing)));
}

#[test]
fn bad_settings_stop_the_start() {
    let mut host = fake(Some(("size", 9.0.into())));
    assert!(matches!(
        start(&mut host),
        Err(ModuleError::Settings(SettingsError::Invalid(_)))
    ));
}

#[test]
fn its_handle_writes_its_own_section() {
    let mut host = fake(None);
    host.handle().set("size", 1.25.into()).unwrap();
    let writes = host.sink.writes();
    assert_eq!(writes.len(), 1);
    assert_eq!(writes[0].0, ID);
}

#[test]
fn a_tap_through_the_kernels_ports_types_in_the_layout_shown() {
    let mut host = fake(None);
    let fakes = store(&mut host, None);
    host.granted = vec![InjectInput];
    let ports = Ports::from_cx(&ModuleCx::new(&mut host)).unwrap();
    let settings = KeyboardSettings::from_defaults().unwrap();
    let mut typing = Typing::new(&BoardSpec::shipped().unwrap(), &settings);
    let q = char_id(Scan::new(0x10));
    let table = ports.layouts.current();
    assert_eq!(typing.tap(&ports.io(&table), &q, false), Ok(Tapped::Done));
    let ids: Vec<_> = ports.layouts.list().iter().map(|l| l.id).collect();
    let (en, fr) = (ids[0], ids[1]);
    assert_eq!(table.label.id, en);
    assert!(fakes.layouts.show(fr));
    let table = ports.layouts.current();
    typing.tap(&ports.io(&table), &q, false).unwrap();
    let text = |t: &str| Stroke::Text(Redacted::new(t.into()), TextRoute::Unicode);
    assert_eq!(fakes.injector.strokes(), [text("q"), text("a")], "AZERTY");
}
