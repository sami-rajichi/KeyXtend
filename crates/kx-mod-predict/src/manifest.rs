//! What the prediction module declares: it reads the focused box, opens the word packs and keeps its learned words
//! sealed, and never types itself (ADR-0033).

use crate::settings::SPEC;
use kx_module_api::ports::{FieldWatchKey, SealedStoreKey, WordPacksKey};
use kx_module_api::{Capability, Manifest, ModuleId, ServiceKey};

/// The module's id, which also names its settings section.
pub const ID: ModuleId = ModuleId::new("predict");

/// The module's manifest: `ReadFocusedText` gates the box and the typed-text events, `SecretStore` the sealed store,
/// and `Vault` only the notebook's emails for the `@@` list, as an event gate (ADR-0038, the ADR-0035 pattern).
pub static MANIFEST: Manifest = Manifest {
    id: ID,
    version: env!("CARGO_PKG_VERSION"),
    requires: &[FieldWatchKey::ID, WordPacksKey::ID, SealedStoreKey::ID],
    provides: &[],
    capabilities: &[
        Capability::ReadFocusedText,
        Capability::SecretStore,
        Capability::Vault,
    ],
    settings: Some(&SPEC),
};

#[cfg(test)]
mod tests {
    use super::*;
    use kx_module_api::ports::{
        AimedLine, BoxesKey, ChipPicked, LayoutAsk, LayoutChanged, PlacesKey, PointerPressed,
        PressesAsk, PressesWanted, Suggested, TypeWord, Typed, TypingMode, TypingModeAsk,
        VaultEmails, VaultStoreKey, WordsAsk,
    };
    use kx_module_api::{Event, Notice};

    #[test]
    fn it_needs_only_the_box_the_packs_and_the_sealed_store() {
        let want = [FieldWatchKey::ID, WordPacksKey::ID, SealedStoreKey::ID];
        assert_eq!(MANIFEST.requires, want);
        assert!(MANIFEST.provides.is_empty());
    }

    #[test]
    fn it_holds_only_its_three_gates_and_never_inject_input() {
        let want = [
            Capability::ReadFocusedText,
            Capability::SecretStore,
            Capability::Vault,
        ];
        assert_eq!(MANIFEST.capabilities, want);
        assert!(!MANIFEST.capabilities.contains(&Capability::InjectInput));
        let gates = [
            FieldWatchKey::CAPABILITY,
            WordPacksKey::CAPABILITY,
            SealedStoreKey::CAPABILITY,
        ];
        assert_eq!(gates, [want.first().copied(), None, want.get(1).copied()]);
    }

    #[test]
    fn the_typed_text_events_need_the_gate_it_holds_and_the_rest_none() {
        let read = Some(Capability::ReadFocusedText);
        let gated = [
            Typed::CAPABILITY,
            AimedLine::CAPABILITY,
            TypingMode::CAPABILITY,
            TypingModeAsk::CAPABILITY,
            TypeWord::CAPABILITY,
            Suggested::CAPABILITY,
            ChipPicked::CAPABILITY,
            WordsAsk::CAPABILITY,
        ];
        assert!(gated.iter().all(|g| *g == read));
        let open = [
            LayoutChanged::CAPABILITY,
            LayoutAsk::CAPABILITY,
            PointerPressed::CAPABILITY,
            PressesWanted::CAPABILITY,
            PressesAsk::CAPABILITY,
            Notice::CAPABILITY,
        ];
        assert!(open.iter().all(Option::is_none));
    }

    #[test]
    fn vault_is_held_only_to_hear_the_notebooks_emails() {
        assert_eq!(VaultEmails::CAPABILITY, Some(Capability::Vault));
        let vault = [VaultStoreKey::ID, BoxesKey::ID, PlacesKey::ID];
        assert!(
            MANIFEST.requires.iter().all(|id| !vault.contains(id)),
            "no vault service"
        );
    }

    #[test]
    fn it_owns_the_predict_section() {
        assert_eq!(MANIFEST.id.as_str(), "predict");
        assert!(MANIFEST.settings.is_some_and(|s| s.version == 1));
    }
}
