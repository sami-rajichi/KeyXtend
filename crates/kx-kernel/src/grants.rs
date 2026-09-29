//! Capability grants: what a module declares that the policy allows.
//! Capabilities are policy inside one process, not a sandbox (ADR-0005).

use kx_module_api::{Capability, Manifest, ModuleId};
use std::collections::{BTreeMap, BTreeSet};

/// The capabilities each module may have; the app builds it.
#[derive(Clone, Debug, Default)]
pub struct Policy {
    allowed: BTreeMap<ModuleId, BTreeSet<Capability>>,
}

impl Policy {
    /// A policy that allows nothing.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Also allows `caps` to `module`.
    #[must_use]
    pub fn allow(mut self, module: ModuleId, caps: &[Capability]) -> Self {
        self.allowed.entry(module).or_default().extend(caps);
        self
    }

    fn allows(&self, module: ModuleId, cap: Capability) -> bool {
        self.allowed
            .get(&module)
            .is_some_and(|caps| caps.contains(&cap))
    }
}

/// What one module may use: the capabilities it declares that the policy allows.
/// Only the kernel builds it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grants {
    granted: Vec<Capability>,
    denied: Vec<Capability>,
}

impl Grants {
    /// The grants of `manifest`'s module under `policy`.
    pub(crate) fn new(manifest: &Manifest, policy: &Policy) -> Self {
        let declared: BTreeSet<Capability> = manifest.capabilities.iter().copied().collect();
        let (granted, denied) = declared
            .into_iter()
            .partition(|&cap| policy.allows(manifest.id, cap));
        Self { granted, denied }
    }

    /// True when the module may use `cap`.
    #[must_use]
    pub fn has(&self, cap: Capability) -> bool {
        self.granted.contains(&cap)
    }

    /// What the module declared but the policy did not allow, for the start log.
    #[must_use]
    pub fn denied(&self) -> &[Capability] {
        &self.denied
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kx_module_api::Capability::{InjectInput, Microphone, Network, ScreenCapture};

    const TYPIST: ModuleId = ModuleId::new("typist");
    const OTHER: ModuleId = ModuleId::new("other");

    static TYPIST_MANIFEST: Manifest = Manifest {
        id: TYPIST,
        version: env!("CARGO_PKG_VERSION"),
        requires: &[],
        provides: &[],
        capabilities: &[Network, Microphone, InjectInput, Microphone],
        settings: None,
    };

    #[test]
    fn grants_are_what_is_both_declared_and_allowed() {
        let policy = Policy::new()
            .allow(TYPIST, &[Microphone, ScreenCapture])
            .allow(TYPIST, &[InjectInput]);
        let grants = Grants::new(&TYPIST_MANIFEST, &policy);
        assert!(grants.has(Microphone) && grants.has(InjectInput));
        assert!(!grants.has(Network), "declared but not allowed");
        assert!(!grants.has(ScreenCapture), "allowed but not declared");
        assert_eq!(grants.denied(), [Network]);
    }

    #[test]
    fn a_module_the_policy_does_not_name_gets_nothing() {
        let policy = Policy::new().allow(OTHER, &[Network]);
        let grants = Grants::new(&TYPIST_MANIFEST, &policy);
        assert!(Capability::ALL.iter().all(|&cap| !grants.has(cap)));
        assert_eq!(grants.denied(), [InjectInput, Microphone, Network]);
    }
}
