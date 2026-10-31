//! What the clipboard module declares (ADR-0034 decision 1): it watches and sets the clipboard, keeps the history
//! sealed, draws pictures in the worker and reads the calendar; it never types, as the keyboard pastes for it.

use kx_module_api::ports::{
    CalendarKey, ClipboardKey, ClipboardWriterKey, PicturesKey, SealedShelfKey,
};
use kx_module_api::{Capability, Manifest, ModuleId, ServiceKey};

use crate::settings::SPEC;

/// The module's id, which also names its settings section.
pub const ID: ModuleId = ModuleId::new("clipboard");

/// The module's manifest: `ReadClipboard` gates the watch and the panel's events, `WriteClipboard` the writer and the
/// paste request, `SecretStore` the shelf.
pub static MANIFEST: Manifest = Manifest {
    id: ID,
    version: env!("CARGO_PKG_VERSION"),
    requires: &[
        ClipboardKey::ID,
        ClipboardWriterKey::ID,
        SealedShelfKey::ID,
        PicturesKey::ID,
        CalendarKey::ID,
    ],
    provides: &[],
    capabilities: &[
        Capability::ReadClipboard,
        Capability::WriteClipboard,
        Capability::SecretStore,
    ],
    settings: Some(&SPEC),
};

#[cfg(test)]
mod tests {
    use super::*;
    use kx_module_api::ports::{
        ClipAct, ClipKey, ClipOpen, ClipPanel, ClipView, HistoryAsk, HistoryWanted, PasteAsk,
        Pasted,
    };
    use kx_module_api::{Event, Notice};

    #[test]
    fn it_needs_the_clipboard_the_writer_the_shelf_pictures_and_the_calendar() {
        let want = [
            ClipboardKey::ID,
            ClipboardWriterKey::ID,
            SealedShelfKey::ID,
            PicturesKey::ID,
            CalendarKey::ID,
        ];
        assert_eq!(MANIFEST.requires, want);
        assert!(MANIFEST.provides.is_empty());
    }

    #[test]
    fn it_holds_the_three_gates_and_never_injects_input() {
        let want = [
            Capability::ReadClipboard,
            Capability::WriteClipboard,
            Capability::SecretStore,
        ];
        assert_eq!(MANIFEST.capabilities, want);
        assert!(!MANIFEST.capabilities.contains(&Capability::InjectInput));
        let gates = [
            ClipboardKey::CAPABILITY,
            ClipboardWriterKey::CAPABILITY,
            SealedShelfKey::CAPABILITY,
        ];
        assert_eq!(gates, want.map(Some));
        assert_eq!(
            (PicturesKey::CAPABILITY, CalendarKey::CAPABILITY),
            (None, None)
        );
    }

    #[test]
    fn the_events_it_uses_need_only_the_gates_it_holds() {
        let read = Some(Capability::ReadClipboard);
        let write = Some(Capability::WriteClipboard);
        assert!(
            [
                ClipKey::CAPABILITY,
                ClipView::CAPABILITY,
                ClipAct::CAPABILITY
            ]
            .iter()
            .all(|g| *g == read)
        );
        assert_eq!([PasteAsk::CAPABILITY, Pasted::CAPABILITY], [write, write]);
        let open = [
            ClipOpen::CAPABILITY,
            ClipPanel::CAPABILITY,
            HistoryWanted::CAPABILITY,
            HistoryAsk::CAPABILITY,
            Notice::CAPABILITY,
        ];
        assert!(open.iter().all(Option::is_none));
    }

    #[test]
    fn it_owns_the_clipboard_section() {
        assert_eq!(MANIFEST.id.as_str(), "clipboard");
        assert!(MANIFEST.settings.is_some_and(|s| s.version == 1));
    }
}
