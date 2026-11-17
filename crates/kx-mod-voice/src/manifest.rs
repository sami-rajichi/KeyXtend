//! What the voice module declares (ADR-0042 decision 8): the microphone through the speech port, downloads the user
//! asks for, its sealed store on its own shelf, the focused box, Copy, the calendar's dates and its models folder
//! shown (ADR-0046), a card's link and the key page's Paste (ADR-0047); it never types.

use kx_module_api::ports::{
    CalendarKey, ClipboardWriterKey, FieldWatchKey, FolderRevealKey, ModelFilesKey,
    SessionWatchKey, SpeechKey, VoiceShelfKey, WebLinkKey,
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
        VoiceShelfKey::ID,
        CalendarKey::ID,
        WebLinkKey::ID,
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
        Capability::OpenLink,
        Capability::ReadClipboard,
    ],
    settings: Some(&SPEC),
};

#[cfg(test)]
mod tests {
    use super::*;
    use kx_module_api::Event;
    use kx_module_api::ports::{
        ClipTake, ClipTaken, LayoutAsk, LayoutChanged, PasteText, PointerPressed, PressesAsk,
        PressesWanted, SealedShelfKey, TextAsk, TextDone, TextPasted, Typed, VoiceAct, VoiceAsk,
        VoiceKey, VoiceLit, VoiceOpen, VoiceView,
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
            Capability::OpenLink,
            Capability::ReadClipboard,
        ];
        assert_eq!(MANIFEST.capabilities, want);
        assert!(!MANIFEST.capabilities.contains(&Capability::InjectInput));
    }

    #[test]
    fn it_shows_its_models_folder_through_the_folder_reveal_and_a_cards_link_through_the_web_link()
    {
        assert!(MANIFEST.requires.contains(&FolderRevealKey::ID));
        assert_eq!(FolderRevealKey::CAPABILITY, Some(Capability::OpenFolder));
        // The app provides the web link; a run whose host lacks it still dictates, and Link then logs why.
        assert!(MANIFEST.requires.contains(&WebLinkKey::ID));
        assert_eq!(WebLinkKey::CAPABILITY, Some(Capability::OpenLink));
        assert!(MANIFEST.declares(Capability::OpenLink));
    }

    #[test]
    fn it_keeps_its_store_on_its_own_shelf_and_dates_by_the_calendar() {
        assert!(MANIFEST.requires.contains(&VoiceShelfKey::ID));
        assert!(!MANIFEST.requires.contains(&SealedShelfKey::ID));
        assert!(MANIFEST.requires.contains(&CalendarKey::ID));
    }

    #[test]
    fn the_events_it_hears_from_others_need_only_gates_it_holds() {
        let heard = [
            Typed::CAPABILITY,
            PointerPressed::CAPABILITY,
            PressesWanted::CAPABILITY,
            PressesAsk::CAPABILITY,
            LayoutChanged::CAPABILITY,
            LayoutAsk::CAPABILITY,
            TextPasted::CAPABILITY,
            ClipTaken::CAPABILITY,
        ];
        assert!(heard.into_iter().flatten().all(|c| MANIFEST.declares(c)));
    }

    // Paste at once asks the clipboard history to paste through the gate Copy holds; the key page's Paste reads the
    // clipboard's text through `ClipTake`, as Quick-fill's does, under `ReadClipboard`.
    #[test]
    fn paste_at_once_needs_the_write_gate_and_the_key_pages_paste_the_read_gate() {
        let write = Some(Capability::WriteClipboard);
        assert_eq!([PasteText::CAPABILITY, TextPasted::CAPABILITY], [write; 2]);
        let read = Some(Capability::ReadClipboard);
        assert_eq!([ClipTake::CAPABILITY, ClipTaken::CAPABILITY], [read; 2]);
        assert!(MANIFEST.declares(Capability::ReadClipboard));
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
            VoiceShelfKey::CAPABILITY,
            CalendarKey::CAPABILITY,
            WebLinkKey::CAPABILITY,
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
            VoiceAsk::CAPABILITY,
            TextAsk::CAPABILITY,
            TextDone::CAPABILITY,
        ];
        assert_eq!(events, [voice; 8]);
    }

    #[test]
    fn it_owns_the_voice_section() {
        assert_eq!(MANIFEST.id.as_str(), "voice");
        assert!(MANIFEST.settings.is_some_and(|s| s.version == 1));
    }
}
