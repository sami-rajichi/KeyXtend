//! One module's start and stop, each contained so a failure or a panic leaves the rest running.

use super::{Kernel, Phase};
use crate::grants::Grants;
use crate::host::{Held, StartHost};
use crate::lifecycle::Step;
use crate::order;
use crate::registry::Registry;
use kx_module_api::{ModuleCx, ModuleError, ModuleId, ModuleState, Notice, keys};
use kx_settings::ARG_MODULE;
use std::fmt::Display;
use std::panic::{AssertUnwindSafe, catch_unwind};

/// Why a module that panicked in `start` failed; the payload itself is never logged.
const PANICKED: &str = "start panicked";

/// Why a module whose dependencies or settings spec are broken cannot start.
const BLOCKED: &str = "its dependencies or settings spec are broken";

impl Kernel {
    /// Starts module `at` and says whether it is `Active`; any refusal, error or panic fails it.
    pub(super) fn start(&mut self, at: usize) -> bool {
        if !self.step(at, Step::Start) {
            return false;
        }
        if self.slots[at].blocked {
            self.fail(at, &BLOCKED);
            return false;
        }
        if let Some(service) = self.waiting_on(at) {
            self.fail(
                at,
                &format_args!("service {service} has no active provider"),
            );
            return false;
        }
        match self.run_start(at) {
            Ok(held) => {
                self.slots[at].held = Some(held);
                self.step(at, Step::Started)
            }
            Err(why) => {
                self.fail(at, &why);
                false
            }
        }
    }

    /// Runs module `at`'s `start` over the registry; on failure its setup is rolled back.
    fn run_start(&mut self, at: usize) -> Result<Held, String> {
        let slot = &mut self.slots[at];
        let (manifest, id) = (slot.manifest, slot.manifest.id);
        let grants = Grants::new(manifest, &self.policy);
        if !grants.denied().is_empty() {
            tracing::info!(module = %id, denied = ?grants.denied(), "capabilities not granted");
        }
        let settings = self.store.as_ref().and_then(|s| s.get(id)).cloned();
        let (bus, clock) = (self.bus.clone(), self.platform.clock());
        let mut host = StartHost::new(manifest, &mut self.registry, grants, bus, clock, settings);
        let module = &mut slot.module;
        let run = catch_unwind(AssertUnwindSafe(|| {
            module.start(&mut ModuleCx::new(&mut host))
        }));
        let held = host.into_held();
        let why = match run {
            Ok(Ok(())) => return Ok(held),
            Ok(Err(error)) => loggable(&error),
            Err(_) => PANICKED.to_owned(),
        };
        release(&mut self.registry, id, held);
        Err(why)
    }

    /// Marks module `at` `Failed`, logs one warning and tells the user; the rest keep running.
    pub(super) fn fail(&mut self, at: usize, why: &dyn Display) {
        if !self.step(at, Step::Fail) {
            return;
        }
        let id = self.slots[at].manifest.id;
        tracing::warn!(module = %id, error = %why, "module failed");
        self.notify(Notice {
            key: keys::MODULE_FAILED,
            module: Some(id),
            args: vec![(ARG_MODULE, id.to_string())],
        });
    }

    /// Stops module `at`: its handlers leave the bus, its `stop` runs contained, then its services go.
    pub(super) fn stop(&mut self, at: usize) {
        if !self.step(at, Step::Stop) {
            return;
        }
        let slot = &mut self.slots[at];
        let (id, module) = (slot.manifest.id, &mut slot.module);
        if let Some(held) = &mut slot.held {
            contained(id, || held.drop_handlers());
        }
        if catch_unwind(AssertUnwindSafe(|| module.stop())).is_err() {
            tracing::warn!(module = %id, "module stop panicked");
        }
        if let Some(held) = slot.held.take() {
            release(&mut self.registry, id, held);
        }
        self.step(at, Step::Stopped);
    }

    /// Stops every `Active` module, dependents first, and ends the kernel's life; calling it twice is safe.
    /// A kernel that booted tries once more to save a change the file is still missing.
    pub fn stop_all(&mut self) {
        let booted = self.phase == Phase::Running;
        self.phase = Phase::Stopped;
        for id in order::stop_order(&self.plan) {
            if let Some(at) = self.active(id) {
                self.stop(at);
            }
        }
        if booted {
            self.save();
        }
    }

    /// Where module `id` sits, when it is `Active`.
    pub(super) fn active(&self, id: ModuleId) -> Option<usize> {
        let at = self.find(id)?;
        (self.slots[at].state == ModuleState::Active).then_some(at)
    }
}

/// `error` as the log may show it: a settings error by its kind alone, as its text may quote a file value.
fn loggable(error: &ModuleError) -> String {
    match error {
        ModuleError::Settings(settings) => settings.kind().to_owned(),
        other => other.to_string(),
    }
}

/// Releases what module `id` held; a panic while its services drop is logged, never passed on.
fn release(registry: &mut Registry, id: ModuleId, held: Held) {
    contained(id, || held.release(registry));
}

/// Runs `cleanup` for module `id`; a panic in it is logged, never passed on.
fn contained(id: ModuleId, cleanup: impl FnOnce()) {
    if catch_unwind(AssertUnwindSafe(cleanup)).is_err() {
        tracing::warn!(module = %id, "module cleanup panicked");
    }
}
