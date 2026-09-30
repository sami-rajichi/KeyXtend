//! The module's start on its settings and its compiled-in board.

use crate::host::{FakeHost, fake};
use kx_mod_keyboard::{ID, KeyboardModule};
use kx_module_api::{Module, ModuleCx, ModuleError, SettingsError};

fn start(host: &mut FakeHost) -> Result<(), ModuleError> {
    KeyboardModule.start(&mut ModuleCx::new(host))
}

#[test]
fn it_starts_on_the_defaults_and_the_shipped_board() {
    assert_eq!(start(&mut fake(None)), Ok(()));
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
