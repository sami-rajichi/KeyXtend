//! The `Kernel` starts modules in dependency order, contains failures and stops them in reverse.
//! `boot` loads and starts, `run` starts and stops one module, `control` holds runtime controls
//! and `wait` says who waits for whom.

mod boot;
mod control;
mod run;
mod wait;

use crate::bus::KernelBus;
use crate::grants::Policy;
use crate::host::Held;
use crate::lifecycle::{self, Step};
use crate::log;
use crate::order::{self, Plan};
use crate::registry::{Owner, Registry};
use kx_module_api::{
    Bus, Manifest, Module, ModuleId, ModuleState, ModuleStateChanged, Notice, ServiceError,
    ServiceId, ServiceKey, Settings, SettingsError, SettingsSpec, parse_as, validate_as,
};
use kx_platform::Platform;
use kx_settings::{Store, StoreError};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::sync::Arc;
use thiserror::Error;

/// The kernel's own settings section id; no module may take it.
pub const KERNEL: ModuleId = ModuleId::new("kernel");

/// The smallest log file cap, in MiB.
const MIN_LOG_MB: u32 = 1;

/// The key of the switched-off list in the kernel's section.
const DISABLED_KEY: &str = "disabled";

/// How the store reads the kernel's section: version 1, with the defaults embedded.
static SPEC: SettingsSpec = SettingsSpec {
    version: 1,
    defaults: include_str!("../defaults.toml"),
    migrations: &[],
    validate: validate_as::<KernelSettings>,
};

/// The kernel's settings: switched-off modules and the log limits.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KernelSettings {
    /// Ids of the modules the user switched off.
    pub disabled: Vec<String>,
    /// The least important level written, one of `log::LOG_LEVELS`.
    pub log_level: String,
    /// The largest log file, in MiB.
    pub log_max_mb: u32,
}

impl Settings for KernelSettings {
    fn check(&self) -> Result<(), SettingsError> {
        let invalid = |why: &str| Err(SettingsError::Invalid(why.to_owned()));
        if log::level(&self.log_level).is_none() {
            return invalid("log_level is not a level name");
        }
        if self.log_max_mb < MIN_LOG_MB {
            return invalid("log_max_mb is below the smallest cap");
        }
        if self.disabled.iter().any(String::is_empty) {
            return invalid("disabled holds an empty id");
        }
        Ok(())
    }
}

/// Why the kernel refused a request; nothing changed then.
#[derive(Debug, Error)]
pub enum KernelError {
    /// No added module has this id.
    #[error("module {0} is not added")]
    Unknown(ModuleId),
    /// A module with this id is already added.
    #[error("module {0} is already added")]
    Duplicate(ModuleId),
    /// The id is the kernel's own, which is no module and cannot be switched off.
    #[error("{0} is the kernel's own id")]
    Reserved(ModuleId),
    /// Only a failed module can be tried again.
    #[error("module {0} has not failed")]
    NotFailed(ModuleId),
    /// The module is switched off, waits for a switched-off provider, or cannot start at all.
    #[error("module {0} cannot start")]
    NotStartable(ModuleId),
    /// The call is not allowed in the kernel's current phase, which it names.
    #[error("not allowed while the kernel is in its {0:?} phase")]
    WrongPhase(Phase),
    /// The switched-off list changes only through `set_enabled`, so states and file agree.
    #[error("the disabled list changes only through set_enabled")]
    SwitchKey,
    /// The registry refused a platform service.
    #[error(transparent)]
    Service(#[from] ServiceError),
    /// The store refused a settings change.
    #[error(transparent)]
    Settings(#[from] StoreError),
}

/// Where the kernel is in its life; each public call belongs to one phase.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Before `boot`: modules and platform services are added.
    Setup,
    /// After `boot`: Try again, switching and settings changes work.
    Running,
    /// After `stop_all`: nothing starts or changes again.
    Stopped,
}

/// One added module and where it is in its lifecycle.
struct Slot {
    module: Box<dyn Module>,
    manifest: &'static Manifest,
    state: ModuleState,
    /// What its running start set up, released when it stops.
    held: Option<Held>,
    /// True when its dependencies or settings spec keep it from ever starting.
    blocked: bool,
}

/// Starts, stops and contains the feature modules; the app owns the one kernel.
pub struct Kernel {
    platform: Arc<dyn Platform>,
    policy: Policy,
    bus: Bus,
    registry: Registry,
    /// The services the platform provides, which never wait for a module.
    platform_ids: BTreeSet<ServiceId>,
    /// The modules in the order they were added.
    slots: Vec<Slot>,
    plan: Plan,
    /// The settings, loaded by `boot`.
    store: Option<Store>,
    /// The notices collected while `boot` runs.
    told: Option<Vec<Notice>>,
    phase: Phase,
}

impl Kernel {
    /// A kernel with no modules; its bus is live at once, so the UI can subscribe before boot.
    #[must_use]
    pub fn new(platform: Arc<dyn Platform>, policy: Policy) -> Self {
        Self {
            platform,
            policy,
            bus: Bus::new(Arc::new(KernelBus::new())),
            registry: Registry::new(),
            platform_ids: BTreeSet::new(),
            slots: Vec::new(),
            plan: order::plan(&[], &BTreeSet::new()),
            store: None,
            told: None,
            phase: Phase::Setup,
        }
    }

    /// Adds a module before boot; it waits `Pending` for its turn.
    ///
    /// # Errors
    /// `WrongPhase` after boot, `Reserved` for the kernel's own id or `Duplicate`; nothing is added.
    pub fn add(&mut self, module: Box<dyn Module>) -> Result<(), KernelError> {
        self.expect(Phase::Setup)?;
        let manifest = module.manifest();
        let id = manifest.id;
        if id == KERNEL {
            return Err(KernelError::Reserved(id));
        }
        if self.find(id).is_some() {
            return Err(KernelError::Duplicate(id));
        }
        self.slots.push(Slot {
            module,
            manifest,
            state: ModuleState::Pending,
            held: None,
            blocked: false,
        });
        Ok(())
    }

    /// Registers a platform service before boot, gated by `K::CAPABILITY`.
    ///
    /// # Errors
    /// `WrongPhase` after boot, or `Service(AlreadyProvided)` when the platform already provides `K`.
    pub fn provide_platform<K: ServiceKey>(
        &mut self,
        service: Arc<K::Api>,
    ) -> Result<(), KernelError> {
        self.expect(Phase::Setup)?;
        let value = Box::new(service);
        (self.registry).provide(Owner::Platform, K::ID, K::CAPABILITY, value)?;
        self.platform_ids.insert(K::ID);
        Ok(())
    }

    /// A handle to the kernel's bus, which carries every notice and state change.
    #[must_use]
    pub fn bus(&self) -> Bus {
        self.bus.clone()
    }

    /// Where module `id` is in its lifecycle, or `None` when it was never added.
    #[must_use]
    pub fn state(&self, id: ModuleId) -> Option<ModuleState> {
        self.find(id).map(|at| self.slots[at].state)
    }

    fn find(&self, id: ModuleId) -> Option<usize> {
        self.slots.iter().position(|s| s.manifest.id == id)
    }

    /// `Ok` in `phase`; otherwise the refusal naming the phase the kernel is in.
    fn expect(&self, phase: Phase) -> Result<(), KernelError> {
        if self.phase == phase {
            Ok(())
        } else {
            Err(KernelError::WrongPhase(self.phase))
        }
    }

    /// Moves module `at` by `step` and announces it; a refused move is a bug, logged and ignored.
    fn step(&mut self, at: usize, step: Step) -> bool {
        let slot = &mut self.slots[at];
        let module = slot.manifest.id;
        match lifecycle::next(slot.state, step) {
            Ok(state) => {
                slot.state = state;
                self.bus.publish(&ModuleStateChanged { module, state });
                true
            }
            Err(bad) => {
                tracing::error!(module = %module, error = %bad, "refused lifecycle move");
                false
            }
        }
    }

    /// Publishes `notice`, and keeps it for `boot` to return while boot runs.
    fn notify(&mut self, notice: Notice) {
        self.bus.publish(&notice);
        if let Some(told) = &mut self.told {
            told.push(notice);
        }
    }

    /// The ids the user switched off; none before boot.
    fn disabled(&self) -> Vec<String> {
        let table = self.store.as_ref().and_then(|s| s.get(KERNEL));
        let settings = table.and_then(|t| parse_as::<KernelSettings>(t).ok());
        settings.map(|s| s.disabled).unwrap_or_default()
    }

    fn is_disabled(&self, id: ModuleId) -> bool {
        self.disabled().iter().any(|d| d == id.as_str())
    }
}

impl Drop for Kernel {
    fn drop(&mut self) {
        self.stop_all();
    }
}
