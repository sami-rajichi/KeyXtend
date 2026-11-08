//! What the selection module declares (ADR-0041 decision 3): it watches the focused box's selection and hands links
//! to the browser; it never types, as the keyboard sends the edit keys for it.

use kx_module_api::ports::{PressAsker, SelectionWatchKey, WebLinkKey};
use kx_module_api::{Capability, Manifest, ModuleId, ServiceKey};

use crate::settings::SPEC;

/// The module's id, which also names its settings section.
pub const ID: ModuleId = ModuleId::new("selection");
/// Who the module is when it asks the mouse module for presses.
pub(crate) const ASKER: PressAsker = PressAsker::Selection;

/// The module's manifest: `ReadFocusedText` gates the watch, `Selection` the chips' events, `OpenLink` the link.
pub static MANIFEST: Manifest = Manifest {
    id: ID,
    version: env!("CARGO_PKG_VERSION"),
    requires: &[SelectionWatchKey::ID, WebLinkKey::ID],
    provides: &[],
    capabilities: &[
        Capability::ReadFocusedText,
        Capability::Selection,
        Capability::OpenLink,
    ],
    settings: Some(&SPEC),
};

#[cfg(test)]
mod tests {
    use super::*;
    use kx_module_api::ports::{
        AimedLine, AssistAsk, AssistChanged, BoardPlace, BoardPlaceAsk, ChipAct, ChipsView,
        EditAsk, EditDone, PointerPressed, PointerReleased, PressesAsk, PressesWanted, Typed,
        TypingModeAsk,
    };
    use kx_module_api::{Event, Notice, NoticeAnswered};

    #[test]
    fn it_needs_the_watch_and_the_link_and_provides_nothing() {
        assert_eq!(MANIFEST.requires, [SelectionWatchKey::ID, WebLinkKey::ID]);
        assert!(MANIFEST.provides.is_empty());
    }

    #[test]
    fn it_holds_the_three_gates_and_never_injects_input() {
        let want = [
            Capability::ReadFocusedText,
            Capability::Selection,
            Capability::OpenLink,
        ];
        assert_eq!(MANIFEST.capabilities, want);
        assert!(!MANIFEST.capabilities.contains(&Capability::InjectInput));
        let gates = [SelectionWatchKey::CAPABILITY, WebLinkKey::CAPABILITY];
        assert_eq!(gates, [want[0], want[2]].map(Some));
    }

    #[test]
    fn the_events_it_uses_need_only_the_gates_it_holds() {
        let sel = Some(Capability::Selection);
        let events = [
            ChipsView::CAPABILITY,
            ChipAct::CAPABILITY,
            EditAsk::CAPABILITY,
            EditDone::CAPABILITY,
        ];
        assert_eq!(events, [sel; 4]);
        let read = Some(Capability::ReadFocusedText);
        let typing = [
            AimedLine::CAPABILITY,
            Typed::CAPABILITY,
            TypingModeAsk::CAPABILITY,
        ];
        assert_eq!(typing, [read; 3]);
        let open = [
            Notice::CAPABILITY,
            NoticeAnswered::CAPABILITY,
            BoardPlace::CAPABILITY,
            BoardPlaceAsk::CAPABILITY,
            PressesWanted::CAPABILITY,
            PressesAsk::CAPABILITY,
            PointerPressed::CAPABILITY,
            PointerReleased::CAPABILITY,
            AssistChanged::CAPABILITY,
            AssistAsk::CAPABILITY,
        ];
        assert_eq!(open, [None; 10]);
    }

    #[test]
    fn it_owns_the_selection_section() {
        assert_eq!(MANIFEST.id.as_str(), "selection");
        assert!(MANIFEST.settings.is_some_and(|s| s.version == 1));
    }
}
