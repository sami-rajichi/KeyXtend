//! What the voice module declares (ADR-0042 decision 8): the microphone through the speech port, downloads the user
//! asks for, its sealed store, the focused box, Copy and its models folder shown (ADR-0046); it never types.

use kx_module_api::ports::{
    ClipboardWriterKey, FieldWatchKey, FolderRevealKey, ModelFilesKey, SessionWatchKey, SpeechKey,
};
use kx_module_api::{Capability, Manifest, ModuleId, ServiceKey};

use crate::settings::SPEC;

/// The module's id, which also names its settings section.
pub const ID: ModuleId = ModuleId::new("voice");

/// The module's manifest: `Voice` gates its events, and each port's gate comes with the port.
pub static MANIFEST: Manifest = Manifest {
    id: ID,
    version: env!("CARGO_PKG_VERSION"),
    requires: &[
        SpeechKey::ID,
        ModelFilesKey::ID,
        SessionWatchKey::ID,
        FieldWatchKey::ID,
        ClipboardWriterKey::ID,
        FolderRevealKey::ID,
    ],
    provides: &[],
    capabilities: &[
        Capability::Voice,
        Capability::Microphone,
        Capability::Network,
        Capability::SecretStore,
        Capability::ReadFocusedText,
        Capability::WriteClipboard,
        Capability::OpenFolder,
    ],
    settings: Some(&SPEC),
};

#[cfg(test)]
mod tests {
    use super::*;
    use kx_module_api::Event;
    use kx_module_api::ports::{
        TextAsk, TextDone, VoiceAct, VoiceKey, VoiceLit, VoiceOpen, VoiceView,
    };

    #[test]
    fn it_holds_the_plans_capabilities_and_never_injects_input() {
        let want = [
            Capability::Voice,
            Capability::Microphone,
            Capability::Network,
            Capability::SecretStore,
            Capability::ReadFocusedText,
            Capability::WriteClipboard,
            Capability::OpenFolder,
        ];
        assert_eq!(MANIFEST.capabilities, want);
        assert!(!MANIFEST.capabilities.contains(&Capability::InjectInput));
    }

    #[test]
    fn it_shows_its_models_folder_through_the_folder_reveal_alone() {
        assert!(MANIFEST.requires.contains(&FolderRevealKey::ID));
        assert_eq!(FolderRevealKey::CAPABILITY, Some(Capability::OpenFolder));
        assert!(!MANIFEST.capabilities.contains(&Capability::OpenLink));
    }

    #[test]
    fn every_service_it_needs_is_gated_by_a_capability_it_holds() {
        let gates = [
            SpeechKey::CAPABILITY,
            ModelFilesKey::CAPABILITY,
            SessionWatchKey::CAPABILITY,
            FieldWatchKey::CAPABILITY,
            ClipboardWriterKey::CAPABILITY,
            FolderRevealKey::CAPABILITY,
        ];
        assert_eq!(MANIFEST.requires.len(), gates.len());
        assert!(
            gates
                .into_iter()
                .flatten()
                .all(|cap| MANIFEST.declares(cap))
        );
        assert!(MANIFEST.provides.is_empty());
    }

    #[test]
    fn the_events_it_uses_need_the_voice_gate() {
        let voice = Some(Capability::Voice);
        let events = [
            VoiceKey::CAPABILITY,
            VoiceLit::CAPABILITY,
            VoiceOpen::CAPABILITY,
            VoiceView::CAPABILITY,
            VoiceAct::CAPABILITY,
            TextAsk::CAPABILITY,
            TextDone::CAPABILITY,
        ];
        assert_eq!(events, [voice; 7]);
    }

    #[test]
    fn it_owns_the_voice_section() {
        assert_eq!(MANIFEST.id.as_str(), "voice");
        assert!(MANIFEST.settings.is_some_and(|s| s.version == 1));
    }
}
