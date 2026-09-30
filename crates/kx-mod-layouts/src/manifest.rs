//! What the layouts module declares: it needs the layout source and provides the layouts service.

use crate::settings::SPEC;
use kx_module_api::ports::{LayoutSourceKey, LayoutsKey};
use kx_module_api::{Capability, Manifest, ModuleId, ServiceKey};

/// The module's id, which also names its settings section.
pub const ID: ModuleId = ModuleId::new("layouts");

/// The module's manifest: `InjectInput` gates both the source it reads and the service it provides.
pub static MANIFEST: Manifest = Manifest {
    id: ID,
    version: env!("CARGO_PKG_VERSION"),
    requires: &[LayoutSourceKey::ID],
    provides: &[LayoutsKey::ID],
    capabilities: &[Capability::InjectInput],
    settings: Some(&SPEC),
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_needs_only_inject_input_the_gate_of_both_services() {
        assert_eq!(MANIFEST.capabilities, [Capability::InjectInput]);
        assert_eq!(LayoutSourceKey::CAPABILITY, Some(Capability::InjectInput));
        assert_eq!(LayoutsKey::CAPABILITY, Some(Capability::InjectInput));
    }

    #[test]
    fn it_reads_the_source_and_provides_the_layouts() {
        assert_eq!(MANIFEST.requires, [LayoutSourceKey::ID]);
        assert_eq!(MANIFEST.provides, [LayoutsKey::ID]);
        assert_eq!(MANIFEST.id.as_str(), "layouts");
        assert!(MANIFEST.settings.is_some_and(|s| s.version == 1));
    }
}
