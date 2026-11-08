//! The configs the parts take from `[selection]`, each in its own units.

use std::time::Duration;

use kx_module_api::ports::SelConfig;

use super::SelSettings;
use crate::chips::Rules;
use crate::strip::StripConfig;

impl SelSettings {
    /// How the platform watches the focused box.
    #[must_use]
    pub fn sel_config(&self) -> SelConfig {
        SelConfig {
            settle_ms: self.settle_ms,
            read_ms: self.read_ms,
            looks_per_s: self.looks_per_s,
        }
    }

    /// Which keycaps may show; Translate only when `link_ok`, as a bad site table turns it off.
    #[must_use]
    pub fn rules(&self, link_ok: bool) -> Rules {
        Rules {
            on: self.on,
            copy: self.copy,
            cut: self.cut,
            paste: self.paste,
            translate: self.translate && link_ok,
            caret_paste: self.caret_paste,
        }
    }

    /// What the strip's rules and its view take; Translate only when `link_ok`.
    #[must_use]
    pub(crate) fn strip_config(&self, link_ok: bool) -> StripConfig {
        StripConfig {
            rules: self.rules(link_ok),
            minimised: self.minimised,
            caret: self.caret(),
            look: self.look,
        }
    }

    /// How long the caret's Paste stays.
    #[must_use]
    pub fn caret(&self) -> Duration {
        Duration::from_secs(self.caret_s.into())
    }

    /// How long a Translate question waits for its answer.
    #[must_use]
    pub fn consent(&self) -> Duration {
        Duration::from_secs(self.consent_s.into())
    }

    /// How long an edit or a Translate open may go unanswered.
    #[must_use]
    pub fn edit(&self) -> Duration {
        Duration::from_millis(self.edit_ms.into())
    }
}
