//! Shared fixtures: the golden layouts, a bus that hears `LayoutChanged`, and the service over them.

#![allow(
    clippy::unwrap_used,
    reason = "these fixtures run only in tests, where a failed setup should fail the test"
)]

use kx_mod_layouts::{LayoutsService, LayoutsSettings, scans};
use kx_module_api::ports::{LayoutChanged, LayoutId, LayoutSource};
use kx_module_api::{Bus, BusCore, Capability, Flow, InterceptFn, NotifyFn, SubscriptionId};
use kx_platform_fake::{FILE_EXT, FakeLayoutSource, LayoutFile, golden_dir};
use std::any::{Any, TypeId};
use std::sync::{Arc, Mutex};

/// The captured tables, in the order the fake installs them: English, French AZERTY, Arabic (102) AZERTY.
pub const GOLDEN: [&str; 3] = ["en-us", "fr-fr", "ar-tn"];

/// Golden table `name`.
pub fn file(name: &str) -> LayoutFile {
    LayoutFile::read(&golden_dir().join(name).with_extension(FILE_EXT)).unwrap()
}

/// The ids of the English, French and Arabic tables.
pub fn ids() -> [LayoutId; 3] {
    GOLDEN.map(|n| file(n).id)
}

/// A fake with the golden tables installed, English in front, following every request.
pub fn source() -> Arc<FakeLayoutSource> {
    Arc::new(FakeLayoutSource::golden(&GOLDEN).unwrap())
}

/// A bus core that keeps the id of every `LayoutChanged` it carries.
#[derive(Default)]
pub struct Heard(Mutex<Vec<LayoutId>>);

impl Heard {
    /// Every id heard, oldest first.
    pub fn ids(&self) -> Vec<LayoutId> {
        self.0.lock().unwrap().clone()
    }
}

impl BusCore for Heard {
    fn publish(&self, _: TypeId, _: &'static str, event: &(dyn Any + Send + Sync)) {
        if let Some(e) = event.downcast_ref::<LayoutChanged>() {
            self.0.lock().unwrap().push(e.id);
        }
    }

    fn intercept(&self, _: TypeId, _: &'static str, _: &mut (dyn Any + Send + Sync)) -> Flow {
        Flow::Continue
    }

    fn subscribe(&self, _: TypeId, _: &'static str, _: NotifyFn) -> SubscriptionId {
        SubscriptionId::new(0)
    }

    fn add_interceptor(
        &self,
        _: TypeId,
        _: &'static str,
        _: i32,
        _: InterceptFn,
    ) -> SubscriptionId {
        SubscriptionId::new(0)
    }

    fn unsubscribe(&self, _: SubscriptionId) {}
}

/// A bus over `heard` limited as the module's own: `InjectInput` only.
pub fn bus(heard: &Arc<Heard>) -> Bus {
    let core: Arc<dyn BusCore> = heard.clone();
    Bus::new(core).restricted(&[Capability::InjectInput])
}

/// The service over `fake` with the default settings, and what its bus heard.
pub fn service(fake: &Arc<FakeLayoutSource>) -> (LayoutsService, Arc<Heard>) {
    let heard = Arc::new(Heard::default());
    let source: Arc<dyn LayoutSource> = fake.clone();
    let settings = LayoutsSettings::from_defaults().unwrap();
    let service = LayoutsService::new(source, settings, scans().unwrap(), bus(&heard));
    (service, heard)
}
