//! The shared fake host of kx-test-support on the keyboard's settings, which checks each write as the kernel does.

#![allow(
    clippy::unwrap_used,
    reason = "these fixtures run only in tests, where a failed setup should fail the test"
)]

use kx_mod_keyboard::{KeyboardSettings, MANIFEST};
use kx_module_api::parse_as;
pub use kx_test_support::host::FakeHost;

/// A host on the keyboard's defaults with `change` laid over them, when given.
pub fn fake(change: Option<(&str, toml::Value)>) -> FakeHost {
    let mut host = FakeHost::new(&MANIFEST);
    if let Some((key, value)) = change {
        host.put(key, value);
    }
    host
}

/// The settings the module starts with.
pub fn start_settings(host: &FakeHost) -> KeyboardSettings {
    parse_as(&host.settings).unwrap()
}

/// The settings as saved so far.
pub fn saved(host: &FakeHost) -> KeyboardSettings {
    host.sink.read_as().unwrap()
}
