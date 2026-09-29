//! What a module declares about itself, and the trait every module implements.

use crate::{Capability, ModuleCx, ModuleError, ModuleId, ServiceId, SettingsSpec};

/// A module's static self-description, which the kernel reads before start.
#[derive(Debug)]
pub struct Manifest {
    /// The module's unique name.
    pub id: ModuleId,
    /// The module crate's version, from `env!("CARGO_PKG_VERSION")`.
    pub version: &'static str,
    /// Services the module consumes.
    pub requires: &'static [ServiceId],
    /// Services the module registers.
    pub provides: &'static [ServiceId],
    /// Capabilities the module may use.
    pub capabilities: &'static [Capability],
    /// The module's settings section, if it has one.
    pub settings: Option<&'static SettingsSpec>,
}

impl Manifest {
    /// True when the module consumes service `id`.
    #[must_use]
    pub fn requires(&self, id: ServiceId) -> bool {
        self.requires.contains(&id)
    }

    /// True when the module registers service `id`.
    #[must_use]
    pub fn provides(&self, id: ServiceId) -> bool {
        self.provides.contains(&id)
    }

    /// True when the module declares capability `cap`.
    #[must_use]
    pub fn declares(&self, cap: Capability) -> bool {
        self.capabilities.contains(&cap)
    }
}

/// A feature module; the kernel starts it after the providers of its required services.
pub trait Module: Send + 'static {
    /// The module's manifest.
    fn manifest(&self) -> &'static Manifest;

    /// Gets and provides services and registers handlers through `cx`.
    /// A guard from the module's own `cx.bus().subscribe()` is its to drop, while a handler from
    /// `cx.subscribe` ends when the module stops or fails.
    ///
    /// # Errors
    /// Any `ModuleError`; the kernel marks the module `Failed`, never calls `stop`, and keeps the
    /// rest running.
    fn start(&mut self, cx: &mut ModuleCx<'_>) -> Result<(), ModuleError>;

    /// Releases what `start` set up; the default does nothing.
    /// It runs only after a successful start, never after a failed or panicking one.
    fn stop(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    const GREETER: ServiceId = ServiceId::new("greeter");
    const TIMER: ServiceId = ServiceId::new("timer");

    static SAMPLE: Manifest = Manifest {
        id: ModuleId::new("sample"),
        version: env!("CARGO_PKG_VERSION"),
        requires: &[GREETER],
        provides: &[TIMER],
        capabilities: &[Capability::Network],
        settings: None,
    };

    #[test]
    fn requires_answers_from_its_list() {
        assert!(SAMPLE.requires(GREETER));
        assert!(!SAMPLE.requires(TIMER));
    }

    #[test]
    fn provides_answers_from_its_list() {
        assert!(SAMPLE.provides(TIMER));
        assert!(!SAMPLE.provides(GREETER));
    }

    #[test]
    fn declares_answers_from_its_list() {
        assert!(SAMPLE.declares(Capability::Network));
        assert!(!SAMPLE.declares(Capability::Microphone));
    }
}
