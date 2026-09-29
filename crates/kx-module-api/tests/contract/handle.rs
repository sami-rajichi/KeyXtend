//! A module's settings handle over a fake sink: bound to its own module and usable from any thread.

use crate::fakes::FakeHost;
use kx_module_api::{Manifest, ModuleCx, ModuleId, SettingsError, SettingsHandle, WriteError};
use std::thread;
use toml::Value;

static OWN: Manifest = Manifest {
    id: ModuleId::new("own"),
    version: env!("CARGO_PKG_VERSION"),
    requires: &[],
    provides: &[],
    capabilities: &[],
    settings: None,
};

/// A key of the module's own section.
const KEY: &str = "size";

#[test]
fn a_handle_is_a_shareable_value() {
    fn shareable<T: Clone + Send + Sync + 'static>() {}
    shareable::<SettingsHandle>();
}

#[test]
fn a_handle_writes_under_its_own_module_id() {
    let mut host = FakeHost::new(&OWN);
    let handle = ModuleCx::new(&mut host).settings_handle();
    assert_eq!(handle.set(KEY, Value::Integer(3)), Ok(()));
    let own = (OWN.id, KEY.to_owned(), Value::Integer(3));
    assert_eq!(host.sink.writes(), [own]);
}

#[test]
fn a_clone_writes_from_another_thread() {
    let mut host = FakeHost::new(&OWN);
    let copy = ModuleCx::new(&mut host).settings_handle().clone();
    let written = thread::spawn(move || copy.set(KEY, Value::Integer(4)));
    assert_eq!(written.join().unwrap(), Ok(()));
    let own = (OWN.id, KEY.to_owned(), Value::Integer(4));
    assert_eq!(host.sink.writes(), [own]);
}

#[test]
fn the_sink_refusal_reaches_the_module() {
    let mut host = FakeHost::new(&OWN);
    let refusal = WriteError::Invalid(SettingsError::Invalid("too big".into()));
    host.sink.refuse(refusal.clone());
    let handle = ModuleCx::new(&mut host).settings_handle();
    assert_eq!(handle.set(KEY, Value::Integer(99)), Err(refusal));
}

#[test]
fn the_debug_text_names_the_module() {
    let mut host = FakeHost::new(&OWN);
    let text = format!("{:?}", ModuleCx::new(&mut host).settings_handle());
    assert!(text.contains(OWN.id.as_str()), "{text}");
}
