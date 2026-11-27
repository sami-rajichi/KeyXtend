//! What the snip module declares (ADR-0056 decision 10): the freeze, the recorder with the microphone's pass, the Snips
//! folder, the Save and folder dialogs, Play, Copy and Open folder, with `Snip` gating its events; and, ungated, the
//! media worker that makes a picture's PNG and the calendar that names a snip.

use kx_module_api::ports::{
    CalendarKey, ClipboardWriterKey, FileAskKey, FolderRevealKey, MediaOpenKey, MicPassKey,
    PicturesKey, RecorderKey, ScreenFreezeKey, SnipFolderKey,
};
use kx_module_api::{Capability, Manifest, ModuleId, ServiceKey};

use crate::settings::SPEC;

/// The module's id, which also names its settings section.
pub const ID: ModuleId = ModuleId::new("snip");

/// The module's manifest: `Snip` gates its events, and each port's gate comes with the port.
pub static MANIFEST: Manifest = Manifest {
    id: ID,
    version: env!("CARGO_PKG_VERSION"),
    requires: &[
        ScreenFreezeKey::ID,
        RecorderKey::ID,
        MicPassKey::ID,
        SnipFolderKey::ID,
        FileAskKey::ID,
        MediaOpenKey::ID,
        ClipboardWriterKey::ID,
        FolderRevealKey::ID,
        PicturesKey::ID,
        CalendarKey::ID,
    ],
    provides: &[],
    capabilities: &[
        Capability::Snip,
        Capability::ScreenCapture,
        Capability::Microphone,
        Capability::WriteClipboard,
        Capability::OpenFolder,
        Capability::SaveFile,
    ],
    settings: Some(&SPEC),
};

#[cfg(test)]
mod tests {
    use super::*;
    use kx_module_api::ports::{
        SnipAct, SnipAside, SnipAsk, SnipLine, SnipLineKey, SnipLit, SnipStart, SnipView,
    };
    use kx_module_api::{Event, ServiceId};

    /// Each gated service the module needs, with its gate.
    const NEEDS: [(ServiceId, Option<Capability>); 8] = [
        (ScreenFreezeKey::ID, ScreenFreezeKey::CAPABILITY),
        (RecorderKey::ID, RecorderKey::CAPABILITY),
        (MicPassKey::ID, MicPassKey::CAPABILITY),
        (SnipFolderKey::ID, SnipFolderKey::CAPABILITY),
        (FileAskKey::ID, FileAskKey::CAPABILITY),
        (MediaOpenKey::ID, MediaOpenKey::CAPABILITY),
        (ClipboardWriterKey::ID, ClipboardWriterKey::CAPABILITY),
        (FolderRevealKey::ID, FolderRevealKey::CAPABILITY),
    ];
    /// Each service the module needs that no capability gates: the PNG maker and the calendar.
    const FREE: [(ServiceId, Option<Capability>); 2] = [
        (PicturesKey::ID, PicturesKey::CAPABILITY),
        (CalendarKey::ID, CalendarKey::CAPABILITY),
    ];

    #[test]
    fn it_holds_the_plans_capabilities_and_never_injects_input() {
        let want = [
            Capability::Snip,
            Capability::ScreenCapture,
            Capability::Microphone,
            Capability::WriteClipboard,
            Capability::OpenFolder,
            Capability::SaveFile,
        ];
        assert_eq!(MANIFEST.capabilities, want);
        assert!(!MANIFEST.declares(Capability::InjectInput));
    }

    #[test]
    fn it_needs_the_snip_ports_copy_open_folder_pictures_and_the_calendar_and_provides_none() {
        let ids: Vec<_> = NEEDS.iter().chain(&FREE).map(|(id, _)| *id).collect();
        assert_eq!(MANIFEST.requires, ids);
        assert!(
            FREE.iter().all(|(_, gate)| gate.is_none()),
            "no gate to grant"
        );
        assert!(MANIFEST.provides.is_empty());
    }

    #[test]
    fn each_capability_gates_a_service_it_needs_or_its_events() {
        let gates: Vec<_> = NEEDS.iter().filter_map(|(_, gate)| *gate).collect();
        assert_eq!(gates.len(), NEEDS.len(), "every port it needs is gated");
        assert!(gates.iter().all(|&cap| MANIFEST.declares(cap)));
        let used = |cap: &Capability| *cap == Capability::Snip || gates.contains(cap);
        assert!(MANIFEST.capabilities.iter().all(used), "the smallest list");
    }

    #[test]
    fn the_events_it_uses_need_the_snip_gate() {
        let snip = Some(Capability::Snip);
        let events = [
            SnipStart::CAPABILITY,
            SnipView::CAPABILITY,
            SnipAct::CAPABILITY,
            SnipAside::CAPABILITY,
            SnipLit::CAPABILITY,
            SnipLine::CAPABILITY,
            SnipLineKey::CAPABILITY,
            SnipAsk::CAPABILITY,
        ];
        assert_eq!(events, [snip; 8]);
        assert!(MANIFEST.declares(Capability::Snip));
    }

    #[test]
    fn it_owns_the_snip_section() {
        assert_eq!(MANIFEST.id.as_str(), "snip");
        assert!(MANIFEST.settings.is_some_and(|s| s.version == 1));
    }
}
