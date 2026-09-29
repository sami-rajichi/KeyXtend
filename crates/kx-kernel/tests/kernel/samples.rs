//! Sample modules, service keys and services for the kernel tests.
//! A sample subscribes, provides, requires and reads its settings, then acts on its switch.

use crate::record::{Did, Log, Mode, SamplePanic, Switch};
use kx_module_api::{
    Capability, Event, Manifest, Module, ModuleCx, ModuleError, ModuleId, ServiceError, ServiceId,
    ServiceKey, Settings, SettingsError, SettingsSpec, validate_as,
};
use serde::{Deserialize, Serialize};
use std::panic;
use std::sync::Arc;

/// Who provided a service the platform registered.
pub const PLATFORM: &str = "platform";
/// The highest level the sample settings accept.
pub const MAX_LEVEL: i64 = 9;
/// The level in the sample defaults.
pub const DEFAULT_LEVEL: i64 = 1;
/// Why a sample start fails when told to.
pub const FAILURE: &str = "sample failure";

/// A sample service API: it names who provided it.
pub trait Api: Send + Sync {
    fn owner(&self) -> &'static str;
}

/// A sample service that logs when it is dropped, so tests see it leave the registry.
struct Svc {
    id: ServiceId,
    owner: &'static str,
    log: Log,
}

impl Api for Svc {
    fn owner(&self) -> &'static str {
        self.owner
    }
}

impl Drop for Svc {
    fn drop(&mut self) {
        self.log.push(Did::Dropped(self.id));
    }
}

/// A sample service `id` provided by `owner`.
pub fn service(id: ServiceId, owner: &'static str, log: &Log) -> Arc<dyn Api> {
    let log = log.clone();
    Arc::new(Svc { id, owner, log })
}

macro_rules! key {
    ($name:ident, $id:literal, $cap:expr) => {
        #[doc = concat!("The sample service `", $id, "`.")]
        pub struct $name;

        impl ServiceKey for $name {
            type Api = dyn Api;
            const ID: ServiceId = ServiceId::new($id);
            const CAPABILITY: Option<Capability> = $cap;
        }
    };
}

key!(Alpha, "alpha", None);
key!(Beta, "beta", None);
key!(Gamma, "gamma", None);
key!(Mic, "mic", Some(Capability::Microphone));

fn provide(cx: &mut ModuleCx<'_>, id: ServiceId, svc: Arc<dyn Api>) -> Result<(), ServiceError> {
    match id {
        Alpha::ID => cx.provide::<Alpha>(svc),
        Beta::ID => cx.provide::<Beta>(svc),
        Mic::ID => cx.provide::<Mic>(svc),
        _ => Err(ServiceError::NotDeclared(id)),
    }
}

fn get(cx: &ModuleCx<'_>, id: ServiceId) -> Result<Arc<dyn Api>, ServiceError> {
    match id {
        Alpha::ID => cx.service::<Alpha>(),
        Beta::ID => cx.service::<Beta>(),
        Mic::ID => cx.service::<Mic>(),
        _ => Err(ServiceError::NotRequired(id)),
    }
}

/// An event every running sample logs.
pub struct Ping;

impl Event for Ping {
    const NAME: &'static str = "ping";
}

/// The sample settings: one level, at most `MAX_LEVEL`.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Level {
    pub level: i64,
}

impl Settings for Level {
    fn check(&self) -> Result<(), SettingsError> {
        if self.level > MAX_LEVEL {
            return Err(SettingsError::Invalid("level is too high".into()));
        }
        Ok(())
    }
}

/// A sound spec whose defaults hold `DEFAULT_LEVEL`.
pub static LEVEL_SPEC: SettingsSpec = SettingsSpec {
    version: 1,
    defaults: "level = 1\n",
    migrations: &[],
    validate: validate_as::<Level>,
};

/// A spec at version 2 with no migration to reach it, which the store calls broken.
pub static BROKEN_SPEC: SettingsSpec = SettingsSpec {
    version: 2,
    ..LEVEL_SPEC
};

/// A sample module built from a `Shape`.
struct Sample {
    manifest: &'static Manifest,
    switch: Switch,
    strict: bool,
    log: Log,
}

impl Sample {
    /// Gets service `id` and logs who provided it; a strict sample fails when it cannot.
    fn take(&self, cx: &ModuleCx<'_>, id: ServiceId) -> Result<(), ServiceError> {
        let me = self.manifest.id;
        match get(cx, id) {
            Ok(svc) => self.log.push(Did::Got(me, svc.owner())),
            Err(e) if self.strict => return Err(e),
            Err(e) => self.log.push(Did::Refused(me, e)),
        }
        Ok(())
    }
}

impl Module for Sample {
    fn manifest(&self) -> &'static Manifest {
        self.manifest
    }

    fn start(&mut self, cx: &mut ModuleCx<'_>) -> Result<(), ModuleError> {
        let me = self.manifest.id;
        self.log.push(Did::Start(me));
        let log = self.log.clone();
        cx.subscribe(move |_: &Ping| log.push(Did::Ping(me)));
        for &id in self.manifest.provides {
            provide(cx, id, service(id, me.as_str(), &self.log))?;
        }
        for &id in self.manifest.requires {
            self.take(cx, id)?;
        }
        if self.manifest.settings.is_some() {
            let level: Level = cx.settings()?;
            self.log.push(Did::Saw(me, level.level));
        }
        match self.switch.get() {
            Mode::Fail => Err(ModuleError::Start(FAILURE)),
            Mode::Panic => panic::panic_any(SamplePanic),
            Mode::Succeed | Mode::PanicInStop => Ok(()),
        }
    }

    fn stop(&mut self) {
        self.log.push(Did::Stop(self.manifest.id));
        if self.switch.get() == Mode::PanicInStop {
            panic::panic_any(SamplePanic);
        }
    }
}

/// A sample module's shape before it is built.
pub struct Shape {
    manifest: Manifest,
    switch: Switch,
    strict: bool,
}

/// A sample that needs nothing, provides nothing and starts fine.
pub fn sample(id: ModuleId) -> Shape {
    let manifest = Manifest {
        id,
        version: env!("CARGO_PKG_VERSION"),
        requires: &[],
        provides: &[],
        capabilities: &[],
        settings: None,
    };
    let (switch, strict) = (Switch::new(Mode::Succeed), true);
    Shape {
        manifest,
        switch,
        strict,
    }
}

impl Shape {
    pub fn requires(mut self, ids: &'static [ServiceId]) -> Self {
        self.manifest.requires = ids;
        self
    }

    pub fn provides(mut self, ids: &'static [ServiceId]) -> Self {
        self.manifest.provides = ids;
        self
    }

    pub fn caps(mut self, caps: &'static [Capability]) -> Self {
        self.manifest.capabilities = caps;
        self
    }

    pub fn settings(mut self, spec: &'static SettingsSpec) -> Self {
        self.manifest.settings = Some(spec);
        self
    }

    /// Behaves as `switch` says at each start and stop.
    pub fn switch(mut self, switch: &Switch) -> Self {
        self.switch = switch.clone();
        self
    }

    pub fn mode(self, mode: Mode) -> Self {
        self.switch(&Switch::new(mode))
    }

    /// A tolerant sample logs a refused service and starts anyway.
    pub fn tolerant(mut self) -> Self {
        self.strict = false;
        self
    }

    /// The module, with its manifest leaked so it lives for the whole test.
    pub fn build(self, log: &Log) -> Box<dyn Module> {
        Box::new(Sample {
            manifest: Box::leak(Box::new(self.manifest)),
            switch: self.switch,
            strict: self.strict,
            log: log.clone(),
        })
    }
}
