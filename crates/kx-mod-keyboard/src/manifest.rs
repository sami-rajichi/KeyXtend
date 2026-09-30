//! What the keyboard module declares; the face it provides and the services it needs arrive with typing.

use crate::settings::SPEC;
use kx_module_api::{Manifest, ModuleId};

/// The module's id, which also names its settings section.
pub const ID: ModuleId = ModuleId::new("keyboard");

/// The module's manifest: for now it only reads its settings and its board.
pub static MANIFEST: Manifest = Manifest {
    id: ID,
    version: env!("CARGO_PKG_VERSION"),
    requires: &[],
    provides: &[],
    capabilities: &[],
    settings: Some(&SPEC),
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_asks_for_nothing_yet_and_owns_the_keyboard_section() {
        assert_eq!(MANIFEST.id.as_str(), "keyboard");
        assert!(MANIFEST.capabilities.is_empty());
        assert!(MANIFEST.requires.is_empty() && MANIFEST.provides.is_empty());
        assert!(MANIFEST.settings.is_some_and(|s| s.version == 1));
    }
}
