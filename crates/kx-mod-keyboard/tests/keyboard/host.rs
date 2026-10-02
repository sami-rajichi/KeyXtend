//! The shared fake host of kx-test-support on the keyboard's settings, which checks each write as the kernel does,
//! and the fakes behind the typing services on the captured EN, FR and AR tables.

#![allow(
    clippy::unwrap_used,
    reason = "these fixtures run only in tests, where a failed setup should fail the test"
)]

use kx_mod_keyboard::{KeyboardSettings, MANIFEST};
use kx_module_api::ports::{
    InputInjector, InputInjectorKey, LayoutSource, LayoutSourceKey, LayoutTable, Layouts,
    LayoutsKey,
};
use kx_module_api::{Capability, ServiceId, ServiceKey};
use kx_module_api::{is_arabic, parse_as};
use kx_platform_fake::{FakeInjector, FakeLayoutSource, FakeLayouts, LayoutFile};
pub use kx_test_support::host::FakeHost;
use std::any::Any;
use std::sync::Arc;

/// The captured tables: English, French AZERTY and Arabic (102) AZERTY.
pub const GOLDEN: [&str; 3] = ["en-us", "fr-fr", "ar-tn"];

/// The short names `kx-mod-layouts` gives these languages by default: its `[layouts] short`, else the code in capitals.
const SHORT: [(&str, &str); 3] = [("en", "EN"), ("fr", "FR"), ("ar", "ع")];

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

/// The settings the store holds, which the kernel saves at the latest when it stops.
pub fn saved(host: &FakeHost) -> KeyboardSettings {
    host.sink.read_as().unwrap()
}

/// The id of golden layout `name`.
pub fn id(name: &str) -> kx_module_api::ports::LayoutId {
    LayoutFile::golden(name).unwrap().id
}

/// A layouts service over the golden tables `names`, each named as the layouts module names it.
pub fn labelled(names: &[&str]) -> FakeLayouts {
    let table = |name: &&str| {
        let file = LayoutFile::golden(name).unwrap();
        let mut table = LayoutTable::from(&file);
        let facts = &file.facts;
        let short = SHORT.iter().find(|(iso, _)| *iso == facts.iso);
        table.label.short = short.map_or_else(|| facts.iso.to_uppercase(), |s| s.1.to_owned());
        table.label.name = vec![facts.native.clone(), facts.layout.clone()];
        table.label.arabic = is_arabic(&facts.native);
        table
    };
    FakeLayouts::new(names.iter().map(table).collect())
}

/// The fakes behind the three typing services.
#[derive(Clone)]
pub struct Fakes {
    /// Behind the input injector.
    pub injector: Arc<FakeInjector>,
    /// Behind the layout source.
    pub source: Arc<FakeLayoutSource>,
    /// Behind the layouts service.
    pub layouts: Arc<FakeLayouts>,
}

impl Fakes {
    /// The fakes over the golden tables `names`, the first in front and shown.
    pub fn golden(names: &[&str]) -> Self {
        Self {
            injector: Arc::new(FakeInjector::new()),
            source: Arc::new(FakeLayoutSource::golden(names).unwrap()),
            layouts: Arc::new(labelled(names)),
        }
    }

    /// Stores the services in `host`, all but `leave`; `InjectInput` is not granted.
    pub fn store(&self, host: &mut FakeHost, leave: Option<ServiceId>) {
        let injector: Arc<dyn InputInjector> = self.injector.clone();
        let source: Arc<dyn LayoutSource> = self.source.clone();
        let layouts: Arc<dyn Layouts> = self.layouts.clone();
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
    }
}

/// Stores the typing services in `host`, all but `leave`, on the golden tables; `InjectInput` is not granted.
pub fn store(host: &mut FakeHost, leave: Option<ServiceId>) -> Fakes {
    let fakes = Fakes::golden(&GOLDEN);
    fakes.store(host, leave);
    fakes
}
