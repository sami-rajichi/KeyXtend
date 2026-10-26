//! What the keyboard module declares: typing needs the injector, the layout source, the layouts service and the
//! shortcuts, it provides the `KeyboardFace` the UI draws, behind `KeyboardUi`, and it tells and types words behind
//! `ReadFocusedText`.

use crate::settings::SPEC;
use kx_module_api::ports::{
    InputInjectorKey, KeyboardFaceKey, LayoutSourceKey, LayoutsKey, ShortcutsKey,
};
use kx_module_api::{Capability, Manifest, ModuleId, ServiceKey};

/// The module's id, which also names its settings section.
pub const ID: ModuleId = ModuleId::new("keyboard");

/// The module's manifest: `InjectInput` gates all four services it needs, `KeyboardUi` the face it provides, and
/// `ReadFocusedText` the typed pieces, the typing mode and the picked words.
pub static MANIFEST: Manifest = Manifest {
    id: ID,
    version: env!("CARGO_PKG_VERSION"),
    requires: &[
        InputInjectorKey::ID,
        LayoutSourceKey::ID,
        LayoutsKey::ID,
        ShortcutsKey::ID,
    ],
    provides: &[KeyboardFaceKey::ID],
    capabilities: &[
        Capability::InjectInput,
        Capability::KeyboardUi,
        Capability::ReadFocusedText,
    ],
    settings: Some(&SPEC),
};

#[cfg(test)]
mod tests {
    use super::*;
    use kx_module_api::Event;
    use kx_module_api::ports::{TypeWord, Typed, TypingMode, TypingModeAsk};

    #[test]
    fn it_types_through_the_injector_the_layouts_and_the_shortcuts() {
        let want = [
            InputInjectorKey::ID,
            LayoutSourceKey::ID,
            LayoutsKey::ID,
            ShortcutsKey::ID,
        ];
        assert_eq!(MANIFEST.requires, want);
    }

    #[test]
    fn it_provides_the_face_behind_keyboard_ui() {
        assert_eq!(MANIFEST.provides, [KeyboardFaceKey::ID]);
        assert_eq!(KeyboardFaceKey::CAPABILITY, Some(Capability::KeyboardUi));
    }

    #[test]
    fn it_holds_the_gates_of_typing_the_face_and_words() {
        let want = [
            Capability::InjectInput,
            Capability::KeyboardUi,
            Capability::ReadFocusedText,
        ];
        assert_eq!(MANIFEST.capabilities, want);
        let gates = [
            InputInjectorKey::CAPABILITY,
            LayoutSourceKey::CAPABILITY,
            LayoutsKey::CAPABILITY,
            ShortcutsKey::CAPABILITY,
        ];
        assert!(gates.iter().all(|&c| c == Some(Capability::InjectInput)));
        let words = [
            Typed::CAPABILITY,
            TypingMode::CAPABILITY,
            TypingModeAsk::CAPABILITY,
            TypeWord::CAPABILITY,
        ];
        assert!(
            words
                .iter()
                .all(|&c| c == Some(Capability::ReadFocusedText))
        );
    }

    #[test]
    fn it_owns_the_keyboard_section() {
        assert_eq!(MANIFEST.id.as_str(), "keyboard");
        assert!(MANIFEST.settings.is_some_and(|s| s.version == 1));
    }
}
