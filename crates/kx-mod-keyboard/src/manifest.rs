//! What the keyboard module declares: typing needs the injector, the layout source and the layouts service,
//! and it provides the `KeyboardFace` the UI draws.

use crate::settings::SPEC;
use kx_module_api::ports::{InputInjectorKey, KeyboardFaceKey, LayoutSourceKey, LayoutsKey};
use kx_module_api::{Capability, Manifest, ModuleId, ServiceKey};

/// The module's id, which also names its settings section.
pub const ID: ModuleId = ModuleId::new("keyboard");

/// The module's manifest: `InjectInput` gates all three services it needs.
pub static MANIFEST: Manifest = Manifest {
    id: ID,
    version: env!("CARGO_PKG_VERSION"),
    requires: &[InputInjectorKey::ID, LayoutSourceKey::ID, LayoutsKey::ID],
    provides: &[KeyboardFaceKey::ID],
    capabilities: &[Capability::InjectInput],
    settings: Some(&SPEC),
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_types_through_the_injector_and_reads_the_layout_source_and_the_layouts() {
        let want = [InputInjectorKey::ID, LayoutSourceKey::ID, LayoutsKey::ID];
        assert_eq!(MANIFEST.requires, want);
    }

    #[test]
    fn it_provides_the_face_which_needs_no_capability() {
        assert_eq!(MANIFEST.provides, [KeyboardFaceKey::ID]);
        assert_eq!(KeyboardFaceKey::CAPABILITY, None);
    }

    #[test]
    fn it_needs_only_inject_input_the_gate_of_all_three() {
        assert_eq!(MANIFEST.capabilities, [Capability::InjectInput]);
        let gates = [
            InputInjectorKey::CAPABILITY,
            LayoutSourceKey::CAPABILITY,
            LayoutsKey::CAPABILITY,
        ];
        assert!(gates.iter().all(|&c| c == Some(Capability::InjectInput)));
    }

    #[test]
    fn it_owns_the_keyboard_section() {
        assert_eq!(MANIFEST.id.as_str(), "keyboard");
        assert!(MANIFEST.settings.is_some_and(|s| s.version == 1));
    }
}
