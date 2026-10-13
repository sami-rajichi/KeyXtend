//! What the mouse module declares: it runs its hold engine on the pointer hook, which sends clicks and modifier keys.

use crate::settings::SPEC;
use kx_module_api::ports::PointerHookKey;
use kx_module_api::{Capability, Manifest, ModuleId, ServiceKey};

/// The module's id, which also names its settings section.
pub const ID: ModuleId = ModuleId::new("mouse");

/// The module's manifest: `PointerHook` gates the hook, and `InjectInput` covers the input its engine sends.
pub static MANIFEST: Manifest = Manifest {
    id: ID,
    version: env!("CARGO_PKG_VERSION"),
    requires: &[PointerHookKey::ID],
    provides: &[],
    capabilities: &[Capability::PointerHook, Capability::InjectInput],
    settings: Some(&SPEC),
};

#[cfg(test)]
mod tests {
    use super::*;
    use kx_module_api::Event;
    use kx_module_api::ports::{AssistToggle, LatchesChanged};

    #[test]
    fn it_needs_only_the_pointer_hook() {
        assert_eq!(MANIFEST.requires, [PointerHookKey::ID]);
    }

    #[test]
    fn it_provides_nothing() {
        assert!(MANIFEST.provides.is_empty());
    }

    #[test]
    fn it_holds_the_hooks_gate_and_the_gate_of_the_input_it_sends() {
        let want = [Capability::PointerHook, Capability::InjectInput];
        assert_eq!(MANIFEST.capabilities, want);
        assert_eq!(PointerHookKey::CAPABILITY, Some(Capability::PointerHook));
        let heard = [AssistToggle::CAPABILITY, LatchesChanged::CAPABILITY];
        assert!(heard.iter().all(|&c| c == Some(Capability::InjectInput)));
    }

    #[test]
    fn it_owns_the_mouse_section() {
        assert_eq!(MANIFEST.id.as_str(), "mouse");
        assert!(MANIFEST.settings.is_some_and(|s| s.version == 1));
    }
}
