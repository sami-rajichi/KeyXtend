//! What the power module declares: it reaches the OS only through the platform's power dialog.

use kx_module_api::ports::PowerDialogKey;
use kx_module_api::{Capability, Manifest, ModuleId, ServiceKey};

/// The module's id.
pub const ID: ModuleId = ModuleId::new("power");

/// The module's manifest: `Power` gates the dialog, its only service, and it has nothing to configure.
pub static MANIFEST: Manifest = Manifest {
    id: ID,
    version: env!("CARGO_PKG_VERSION"),
    requires: &[PowerDialogKey::ID],
    provides: &[],
    capabilities: &[Capability::Power],
    settings: None,
};

#[cfg(test)]
mod tests {
    use super::*;
    use kx_module_api::ports::{PowerAsk, PowerAsked, PowerChanged};
    use kx_module_api::{Event, Notice};

    #[test]
    fn it_needs_only_the_dialog() {
        assert_eq!(MANIFEST.requires, [PowerDialogKey::ID]);
    }

    #[test]
    fn it_provides_nothing() {
        assert!(MANIFEST.provides.is_empty());
    }

    #[test]
    fn it_holds_only_the_dialogs_gate() {
        assert_eq!(MANIFEST.capabilities, [Capability::Power]);
        assert_eq!(PowerDialogKey::CAPABILITY, Some(Capability::Power));
    }

    #[test]
    fn it_owns_no_settings() {
        assert_eq!(MANIFEST.id.as_str(), "power");
        assert!(MANIFEST.settings.is_none());
    }

    #[test]
    fn the_events_it_hears_and_sends_need_no_capability() {
        let gates = [
            PowerAsked::CAPABILITY,
            PowerAsk::CAPABILITY,
            PowerChanged::CAPABILITY,
            Notice::CAPABILITY,
        ];
        assert!(gates.iter().all(Option::is_none));
    }
}
