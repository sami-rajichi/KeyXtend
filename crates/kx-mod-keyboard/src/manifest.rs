//! What the keyboard module declares: typing needs the injector, the layout source, the layouts service and the
//! shortcuts, it provides the `KeyboardFace` the UI draws, behind `KeyboardUi`, it tells and types words behind
//! `ReadFocusedText`, it sends the clipboard panel's keys and pastes behind the clipboard gates, Quick-fill's keys
//! and fills behind `Vault`, the selection chips' edit keys behind `Selection`, dictated texts behind `Voice`, and the
//! Snip key and its windows' keys behind `Snip`.

use crate::settings::SPEC;
use kx_module_api::ports::{
    InputInjectorKey, KeyboardFaceKey, LayoutSourceKey, LayoutsKey, ShortcutsKey,
};
use kx_module_api::{Capability, Manifest, ModuleId, ServiceKey};

/// The module's id, which also names its settings section.
pub const ID: ModuleId = ModuleId::new("keyboard");

/// The module's manifest: `InjectInput` gates all four services it needs, `KeyboardUi` the face it provides,
/// `ReadFocusedText` the typed pieces, the typing mode and the picked words, and `ReadClipboard` and `WriteClipboard`
/// the panel's state and keys and the paste requests, `Vault` Quick-fill's, `Selection` the chips' edit requests,
/// `Voice` the Dictation key and its texts and `Snip` the Snip key and its windows' keys; it needs no clipboard, vault,
/// selection, speech or capture service.
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
        Capability::ReadClipboard,
        Capability::WriteClipboard,
        // An event gate only, as `ReadClipboard` is for the clipboard: Quick-fill's state, keys and fills need it.
        Capability::Vault,
        // An event gate only too: the chips' edit requests and their answers need it (ADR-0041 decision 3).
        Capability::Selection,
        // An event gate only too: the Dictation key, its light and the texts to type need it (ADR-0042 decision 7).
        Capability::Voice,
        // An event gate only too: the Snip key, its light and the snip windows' line need it (ADR-0056).
        Capability::Snip,
    ],
    settings: Some(&SPEC),
};

#[cfg(test)]
mod tests {
    use super::*;
    use kx_module_api::Event;
    use kx_module_api::ports::{
        AimedLine, BoxesKey, ClipKey, ClipLine, ClipPanel, ClipboardKey, ClipboardWriterKey,
        EditAsk, EditDone, FillAsk, FillClose, FillKey, FillLine, FillLive, FillOpen, FillPanel,
        FillType, FillTyped, PanelPressesWanted, PasteAsk, Pasted, PlacesKey, PressesAsk,
        ScreenFreezeKey, SnipAsk, SnipLine, SnipLineKey, SnipLit, SnipStart, SpeechKey, TextAsk,
        TextDone, TypeWord, Typed, TypingMode, TypingModeAsk, VaultStoreKey, VoiceAsk, VoiceClose,
        VoiceKey, VoiceLine, VoiceLineKey, VoiceLit,
    };

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
    fn it_holds_the_gates_of_typing_the_face_words_and_the_clipboard_panel() {
        let want = [
            Capability::InjectInput,
            Capability::KeyboardUi,
            Capability::ReadFocusedText,
            Capability::ReadClipboard,
            Capability::WriteClipboard,
            Capability::Vault,
            Capability::Selection,
            Capability::Voice,
            Capability::Snip,
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
            AimedLine::CAPABILITY,
        ];
        assert!(
            words
                .iter()
                .all(|&c| c == Some(Capability::ReadFocusedText))
        );
    }

    #[test]
    fn the_clipboard_gates_cover_the_panel_and_its_keys_and_the_paste_and_no_clipboard_service() {
        let panel = [
            ClipPanel::CAPABILITY,
            ClipKey::CAPABILITY,
            ClipLine::CAPABILITY,
        ];
        assert_eq!(panel, [Some(Capability::ReadClipboard); 3]);
        let paste = [PasteAsk::CAPABILITY, Pasted::CAPABILITY];
        assert_eq!(paste, [Some(Capability::WriteClipboard); 2]);
        let clipboard = [ClipboardKey::ID, ClipboardWriterKey::ID];
        assert!(MANIFEST.requires.iter().all(|s| !clipboard.contains(s)));
    }

    #[test]
    fn the_vault_gate_covers_quick_fill_s_state_keys_and_fills_and_no_vault_service() {
        assert!(MANIFEST.capabilities.contains(&Capability::Vault));
        let fill = [
            FillLive::CAPABILITY,
            FillPanel::CAPABILITY,
            FillKey::CAPABILITY,
            FillLine::CAPABILITY,
            FillType::CAPABILITY,
            FillTyped::CAPABILITY,
        ];
        assert_eq!(fill, [Some(Capability::Vault); 6]);
        let free = [
            FillOpen::CAPABILITY,
            FillClose::CAPABILITY,
            FillAsk::CAPABILITY,
        ];
        assert_eq!(free, [None; 3]);
        let vault = [VaultStoreKey::ID, BoxesKey::ID, PlacesKey::ID];
        assert!(MANIFEST.requires.iter().all(|s| !vault.contains(s)));
        assert!(
            !MANIFEST.capabilities.contains(&Capability::VaultData),
            "ADR-0039"
        );
    }

    #[test]
    fn the_selection_gate_covers_the_edit_asks_and_their_answers() {
        assert!(MANIFEST.capabilities.contains(&Capability::Selection));
        let edits = [EditAsk::CAPABILITY, EditDone::CAPABILITY];
        assert_eq!(edits, [Some(Capability::Selection); 2]);
    }

    #[test]
    fn the_voice_gate_covers_the_key_its_light_the_texts_and_the_panels_line_and_no_microphone() {
        assert!(MANIFEST.capabilities.contains(&Capability::Voice));
        let voice = [
            VoiceKey::CAPABILITY,
            VoiceLit::CAPABILITY,
            VoiceAsk::CAPABILITY,
            TextAsk::CAPABILITY,
            TextDone::CAPABILITY,
            VoiceLine::CAPABILITY,
            VoiceLineKey::CAPABILITY,
            VoiceClose::CAPABILITY,
        ];
        assert_eq!(voice, [Some(Capability::Voice); 8]);
        assert!(!MANIFEST.requires.contains(&SpeechKey::ID));
        assert!(!MANIFEST.capabilities.contains(&Capability::Microphone));
    }

    #[test]
    fn the_snip_gate_covers_the_key_its_light_the_line_and_the_ask_and_no_capture() {
        assert!(MANIFEST.capabilities.contains(&Capability::Snip));
        let snip = [
            SnipStart::CAPABILITY,
            SnipLit::CAPABILITY,
            SnipLine::CAPABILITY,
            SnipLineKey::CAPABILITY,
            SnipAsk::CAPABILITY,
        ];
        assert_eq!(snip, [Some(Capability::Snip); 5]);
        assert!(!MANIFEST.requires.contains(&ScreenFreezeKey::ID));
        assert!(!MANIFEST.capabilities.contains(&Capability::ScreenCapture));
    }

    #[test]
    fn the_wish_for_presses_and_its_ask_need_no_gate() {
        let free = [PanelPressesWanted::CAPABILITY, PressesAsk::CAPABILITY];
        assert_eq!(free, [None; 2]);
    }

    #[test]
    fn it_owns_the_keyboard_section() {
        assert_eq!(MANIFEST.id.as_str(), "keyboard");
        assert!(MANIFEST.settings.is_some_and(|s| s.version == 1));
    }
}
