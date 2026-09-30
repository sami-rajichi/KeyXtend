//! The shared fake host of kx-test-support on the keyboard's settings, which checks each write as the kernel does,
//! and the fakes behind the typing services on the captured EN, FR and AR tables.

#![allow(
    clippy::unwrap_used,
    reason = "these fixtures run only in tests, where a failed setup should fail the test"
)]

use kx_mod_keyboard::{KeyboardSettings, MANIFEST};
use kx_module_api::parse_as;
use kx_module_api::ports::{
    InputInjector, InputInjectorKey, LayoutSource, LayoutSourceKey, Layouts, LayoutsKey,
};
use kx_module_api::{Capability, ServiceId, ServiceKey};
use kx_platform_fake::{FakeInjector, FakeLayoutSource, FakeLayouts};
pub use kx_test_support::host::FakeHost;
use std::any::Any;
use std::sync::Arc;

/// The captured tables: English, French AZERTY and Arabic (102) AZERTY.
pub const GOLDEN: [&str; 3] = ["en-us", "fr-fr", "ar-tn"];

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

/// The fakes behind the three typing services.
pub struct Fakes {
    /// Behind the input injector.
    pub injector: Arc<FakeInjector>,
    /// Behind the layout source.
    pub source: Arc<FakeLayoutSource>,
    /// Behind the layouts service.
    pub layouts: Arc<FakeLayouts>,
}

/// Stores the typing services in `host`, all but `leave`, on the golden tables; `InjectInput` is not granted.
pub fn store(host: &mut FakeHost, leave: Option<ServiceId>) -> Fakes {
    let fakes = Fakes {
        injector: Arc::new(FakeInjector::new()),
        source: Arc::new(FakeLayoutSource::golden(&GOLDEN).unwrap()),
        layouts: Arc::new(FakeLayouts::golden(&GOLDEN).unwrap()),
    };
    let injector: Arc<dyn InputInjector> = fakes.injector.clone();
    let source: Arc<dyn LayoutSource> = fakes.source.clone();
    let layouts: Arc<dyn Layouts> = fakes.layouts.clone();
    let stored: [(ServiceId, Box<dyn Any + Send + Sync>); 3] = [
        (InputInjectorKey::ID, Box::new(injector)),
        (LayoutSourceKey::ID, Box::new(source)),
        (LayoutsKey::ID, Box::new(layouts)),
    ];
    for (id, service) in stored {
        if leave != Some(id) {
            let gate = Some(Capability::InjectInput);
            host.services.insert(id, (gate, service));
        }
    }
    fakes
}
