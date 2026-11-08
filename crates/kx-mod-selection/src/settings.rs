//! The `[selection]` settings (P10 design "Settings"): the switches, the caret's Paste, the keycaps' look, Translate's
//! target, site and length, and the watch's waits. Each range is a named const.

mod parts;

use std::ops::RangeInclusive;

use kx_module_api::ports::Legend;
use kx_module_api::{Settings, SettingsError, SettingsSpec, refuse_if, validate_as, within};
use serde::{Deserialize, Serialize};

use crate::link::{Sites, sites, tag_ok};

/// Seconds the caret's Paste stays.
pub const CARET_S: RangeInclusive<u32> = 2..=30;
/// The most characters Translate sends; each site may take fewer.
pub const TRANSLATE_CHARS: RangeInclusive<u16> = 50..=5_000;
/// The wait after a change before a look, in ms.
pub const SETTLE_MS: RangeInclusive<u32> = 30..=1_000;
/// The longest one look or read may take, in ms.
pub const READ_MS: RangeInclusive<u32> = 50..=2_000;
/// The most looks in one second.
pub const LOOKS_PER_S: RangeInclusive<u16> = 1..=30;
/// Seconds a Translate question waits for its answer.
pub const CONSENT_S: RangeInclusive<u32> = 5..=120;
/// The key the module sets itself when the user says yes to Translate.
pub const TRANSLATE_OK_KEY: &str = "translate_ok";

/// Why a value or a pair was refused.
const BAD_TO: &str = "translate_to must be empty or a short language tag of letters and hyphens";
const BAD_SITE: &str = "translate_site must name a site in sites.toml";
const BAD_CHARS: &str = "translate_chars must not pass translate_site's limit";

/// How the kernel reads the `[selection]` section: version 1, with the defaults embedded.
pub static SPEC: SettingsSpec = SettingsSpec {
    version: 1,
    defaults: include_str!("../defaults.toml"),
    migrations: &[],
    validate: validate_as::<SelSettings>,
};

/// The module's settings; every value comes from `defaults.toml` or the user's file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "Each flag is one on-or-off setting of the flat [selection] section the user reads and writes."
)]
pub struct SelSettings {
    /// The whole helper.
    pub on: bool,
    /// The Copy keycap.
    pub copy: bool,
    /// The Cut keycap.
    pub cut: bool,
    /// The Paste keycap.
    pub paste: bool,
    /// The Translate keycap.
    pub translate: bool,
    /// A click into an editable box shows Paste by the caret.
    pub caret_paste: bool,
    /// Seconds the caret's Paste stays.
    pub caret_s: u32,
    /// What each keycap shows: its icon and word, or one of them; any other value goes back to `both`.
    pub look: Legend,
    /// The chips show while the keyboard is minimised.
    pub minimised: bool,
    /// The language Translate asks for; empty follows the system's display language.
    pub translate_to: String,
    /// The translate site, by its id in `sites.toml`.
    pub translate_site: String,
    /// The most characters Translate sends.
    pub translate_chars: u16,
    /// The user said yes to sending selected text to the site.
    pub translate_ok: bool,
    /// The wait after a change before a look, in ms.
    pub settle_ms: u32,
    /// The longest one look or read may take, in ms.
    pub read_ms: u32,
    /// The most looks in one second.
    pub looks_per_s: u16,
    /// Seconds a Translate question waits for its answer.
    pub consent_s: u32,
}

impl SelSettings {
    /// The settings in the embedded `defaults.toml`.
    ///
    /// # Errors
    /// `SettingsError::Parse` when the defaults do not parse, or the error from `check`.
    pub fn from_defaults() -> Result<Self, SettingsError> {
        SPEC.defaults_as()
    }

    /// The ranges.
    fn check_ranges(&self) -> Result<(), SettingsError> {
        within("caret_s", self.caret_s, &CARET_S)?;
        within("translate_chars", self.translate_chars, &TRANSLATE_CHARS)?;
        within("settle_ms", self.settle_ms, &SETTLE_MS)?;
        within("read_ms", self.read_ms, &READ_MS)?;
        within("looks_per_s", self.looks_per_s, &LOOKS_PER_S)?;
        within("consent_s", self.consent_s, &CONSENT_S)
    }

    /// The site is one of `sites`, and `translate_chars` stays within its limit.
    fn check_site(&self, sites: &Sites) -> Result<(), SettingsError> {
        let site = sites.get(&self.translate_site);
        refuse_if(site.is_none(), BAD_SITE)?;
        refuse_if(
            site.is_some_and(|s| self.translate_chars > s.limit),
            BAD_CHARS,
        )
    }
}

impl Settings for SelSettings {
    /// The ranges and the target; the site only when the table loads, as a bad one turns Translate off.
    fn check(&self) -> Result<(), SettingsError> {
        self.check_ranges()?;
        let to = &self.translate_to;
        refuse_if(!to.is_empty() && !tag_ok(to), BAD_TO)?;
        sites().map_or(Ok(()), |table| self.check_site(table))
    }
}

#[cfg(test)]
mod tests;
