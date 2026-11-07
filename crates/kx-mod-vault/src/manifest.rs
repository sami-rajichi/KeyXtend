//! What the vault module declares (ADR-0038 decision 3, ADR-0039): it keeps the notebook in its own sealed store,
//! reads the calendar for its stamps, random bytes for ids and Make one, and the boxes and places for the backup.
//! `ReadClipboard` gates `ClipTake` and `ClipTaken`, as Paste hands the clipboard's text into a field.

use kx_module_api::ports::{BoxesKey, CalendarKey, PlacesKey, RandomKey, VaultStoreKey};
use kx_module_api::{Capability, Manifest, ModuleId, ServiceKey};

use crate::settings::SPEC;

/// The module's id, which also names its settings section.
pub const ID: ModuleId = ModuleId::new("vault");

/// The module's manifest: `VaultData` gates the notebook store, boxes and places, `Vault` the panel's events.
pub static MANIFEST: Manifest = Manifest {
    id: ID,
    version: env!("CARGO_PKG_VERSION"),
    requires: &[
        VaultStoreKey::ID,
        CalendarKey::ID,
        RandomKey::ID,
        BoxesKey::ID,
        PlacesKey::ID,
    ],
    provides: &[],
    capabilities: &[
        Capability::Vault,
        Capability::VaultData,
        Capability::ReadClipboard,
    ],
    settings: Some(&SPEC),
};

#[cfg(test)]
mod tests {
    use super::*;
    use kx_module_api::ports::{
        ClipTake, ClipTaken, FillAct, FillAsk, FillClose, FillKey, FillLine, FillLive, FillNames,
        FillOpen, FillPanel, FillType, FillTyped, FillView, VaultEmails,
    };
    use kx_module_api::{Event, Notice, NoticeAnswered};

    #[test]
    fn it_needs_the_notebook_store_the_calendar_random_bytes_the_boxes_and_the_places() {
        let want = [
            VaultStoreKey::ID,
            CalendarKey::ID,
            RandomKey::ID,
            BoxesKey::ID,
            PlacesKey::ID,
        ];
        assert_eq!(MANIFEST.requires, want);
        assert!(MANIFEST.provides.is_empty());
        assert_eq!(
            (CalendarKey::CAPABILITY, RandomKey::CAPABILITY),
            (None, None)
        );
    }

    #[test]
    fn it_holds_vault_vault_data_and_read_clipboard_and_never_injects_input() {
        let want = [
            Capability::Vault,
            Capability::VaultData,
            Capability::ReadClipboard,
        ];
        assert_eq!(MANIFEST.capabilities, want);
        let data = Some(Capability::VaultData);
        let keys = [
            VaultStoreKey::CAPABILITY,
            BoxesKey::CAPABILITY,
            PlacesKey::CAPABILITY,
        ];
        assert_eq!(keys, [data; 3], "ADR-0039");
        assert!(!MANIFEST.capabilities.contains(&Capability::InjectInput));
        assert!(!MANIFEST.capabilities.contains(&Capability::WriteClipboard));
    }

    #[test]
    fn the_events_it_uses_need_only_the_gates_it_holds() {
        let vault = Some(Capability::Vault);
        let gated = [
            FillLive::CAPABILITY,
            FillPanel::CAPABILITY,
            FillKey::CAPABILITY,
            FillLine::CAPABILITY,
            FillView::CAPABILITY,
            FillAct::CAPABILITY,
            FillType::CAPABILITY,
            FillTyped::CAPABILITY,
            VaultEmails::CAPABILITY,
            FillNames::CAPABILITY,
        ];
        assert!(gated.iter().all(|g| *g == vault));
        let read = Some(Capability::ReadClipboard);
        assert_eq!([ClipTake::CAPABILITY, ClipTaken::CAPABILITY], [read, read]);
        let open = [
            FillOpen::CAPABILITY,
            FillClose::CAPABILITY,
            FillAsk::CAPABILITY,
            Notice::CAPABILITY,
            NoticeAnswered::CAPABILITY,
        ];
        assert!(open.iter().all(Option::is_none));
    }

    #[test]
    fn it_owns_the_vault_section() {
        assert_eq!(MANIFEST.id.as_str(), "vault");
        assert!(MANIFEST.settings.is_some_and(|s| s.version == 1));
    }
}
