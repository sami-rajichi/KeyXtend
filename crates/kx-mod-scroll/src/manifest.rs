//! What the scroll module declares: it moves other apps' windows only through the platform's scroll target.

use crate::settings::SPEC;
use kx_module_api::ports::ScrollTargetKey;
use kx_module_api::{Capability, Manifest, ModuleId, ServiceKey};

/// The module's id, which also names its settings section.
pub const ID: ModuleId = ModuleId::new("scroll");

/// The module's manifest: `Scroll` gates the scroll target, its only service.
pub static MANIFEST: Manifest = Manifest {
    id: ID,
    version: env!("CARGO_PKG_VERSION"),
    requires: &[ScrollTargetKey::ID],
    provides: &[],
    capabilities: &[Capability::Scroll],
    settings: Some(&SPEC),
};

#[cfg(test)]
mod tests {
    use super::*;
    use kx_module_api::Event;
    use kx_module_api::ports::{
        PointerPressed, ScrollAimed, ScrollAsk, ScrollChanged, ScrollPage, ScrollPeek, ScrollStop,
        ScrollTap,
    };

    #[test]
    fn it_needs_only_the_scroll_target() {
        assert_eq!(MANIFEST.requires, [ScrollTargetKey::ID]);
    }

    #[test]
    fn it_provides_nothing() {
        assert!(MANIFEST.provides.is_empty());
    }

    #[test]
    fn it_holds_only_the_scroll_targets_gate() {
        assert_eq!(MANIFEST.capabilities, [Capability::Scroll]);
        assert_eq!(ScrollTargetKey::CAPABILITY, Some(Capability::Scroll));
    }

    #[test]
    fn the_events_it_hears_and_sends_need_no_capability() {
        let gates = [
            ScrollTap::CAPABILITY,
            ScrollStop::CAPABILITY,
            ScrollPeek::CAPABILITY,
            ScrollAsk::CAPABILITY,
            PointerPressed::CAPABILITY,
            ScrollChanged::CAPABILITY,
            ScrollAimed::CAPABILITY,
            ScrollPage::CAPABILITY,
        ];
        assert!(gates.iter().all(Option::is_none));
    }

    #[test]
    fn it_owns_the_scroll_section() {
        assert_eq!(MANIFEST.id.as_str(), "scroll");
        assert!(MANIFEST.settings.is_some_and(|s| s.version == 1));
    }
}
