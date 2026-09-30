//! The `[ui]` settings: the look, the language, tips, timings and paging, with every number's limits in one place.

use crate::i18n::{AUTO, find};
use kx_module_api::{Settings, SettingsError, SettingsSpec, validate_as};
use kx_platform::ui::BackdropKind;
use kx_theme::contrast::MIN_CONTRAST;
use kx_theme::{LookChoice, ModeChoice, Themes};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::ops::RangeInclusive;
use std::sync::OnceLock;

/// How long the pointer may rest before a tip shows, in ms.
pub(crate) const TIP_MS: RangeInclusive<u64> = 1..=10_000;
/// How often a move or resize may follow the pointer, in ms.
pub(crate) const POLL_MS: RangeInclusive<u64> = 1..=1_000;
/// How long the window guard may wait before each try, in ms.
pub(crate) const GUARD_DELAY_MS: RangeInclusive<u64> = 1..=10_000;
/// How many times the window guard may try.
pub(crate) const GUARD_TRIES: RangeInclusive<u32> = 1..=100;
/// How long the tray menu may ignore a new front window after it opens, in ms; 0 turns that off.
pub(crate) const TRAY_GRACE_MS: RangeInclusive<u64> = 0..=5_000;
/// How many rows a list page may show; every list is paged (ADR-0008).
pub(crate) const PAGE_ROWS: RangeInclusive<usize> = 1..=10;

/// Why `language` was refused.
const NO_LANGUAGE: &str = "language must be auto or a language the UI speaks";

/// How the kernel reads the `[ui]` section: version 1, with the defaults embedded.
pub static SPEC: SettingsSpec = SettingsSpec {
    version: 1,
    defaults: include_str!("../defaults.toml"),
    migrations: &[],
    validate: validate_as::<UiSettings>,
};

/// What sits behind the keyboard's plate.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Backdrop {
    /// A solid plate the UI draws.
    None,
    /// The system's Mica material.
    Mica,
    /// The system's frosted glass.
    Acrylic,
}

impl Backdrop {
    /// The port's name for this backdrop.
    #[must_use]
    pub fn kind(self) -> BackdropKind {
        match self {
            Self::None => BackdropKind::None,
            Self::Mica => BackdropKind::Mica,
            Self::Acrylic => BackdropKind::Acrylic,
        }
    }
}

/// The ui module's settings; every value comes from `defaults.toml` or the user's file.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiSettings {
    /// The theme, by `themes.toml` id.
    pub theme: String,
    /// Light, dark, or following the system.
    pub mode: ModeChoice,
    /// Themes that can take the system accent take it.
    pub follow_accent: bool,
    /// Every animation is off except the hold ring.
    pub reduced_motion: bool,
    /// What sits behind the plate.
    pub backdrop: Backdrop,
    /// The UI language tag, or `auto`.
    pub language: String,
    /// How long the pointer rests before a tip shows, in ms.
    pub tip_ms: u64,
    /// How often a move or resize follows the pointer, in ms.
    pub poll_ms: u64,
    /// How long the window guard waits before each try, in ms.
    pub guard_delay_ms: u64,
    /// How many times the window guard tries.
    pub guard_tries: u32,
    /// How long the tray menu ignores a new front window after it opens, in ms, as the hidden icons' pop-up closing
    /// can bring one.
    pub tray_grace_ms: u64,
    /// Rows per page in every list.
    pub page_rows: usize,
    /// The contrast a legend keeps when its key takes the accent.
    pub min_contrast: f32,
}

impl UiSettings {
    /// The settings in the embedded `defaults.toml`.
    ///
    /// # Errors
    /// `SettingsError::Parse` when the defaults do not parse, or the error from `check`.
    pub fn from_defaults() -> Result<Self, SettingsError> {
        SPEC.defaults_as()
    }

    /// The look view's input: theme, mode, accent, motion and contrast.
    #[must_use]
    pub fn choice(&self) -> LookChoice {
        LookChoice {
            theme: self.theme.clone(),
            mode: self.mode,
            follow_accent: self.follow_accent,
            reduced_motion: self.reduced_motion,
            min_contrast: self.min_contrast,
        }
    }

    /// Every timing and count lies in its range.
    fn check_numbers(&self) -> Result<(), SettingsError> {
        within("tip_ms", self.tip_ms, &TIP_MS)?;
        within("poll_ms", self.poll_ms, &POLL_MS)?;
        within("guard_delay_ms", self.guard_delay_ms, &GUARD_DELAY_MS)?;
        within("guard_tries", self.guard_tries, &GUARD_TRIES)?;
        within("tray_grace_ms", self.tray_grace_ms, &TRAY_GRACE_MS)?;
        within("page_rows", self.page_rows, &PAGE_ROWS)
    }
}

impl Settings for UiSettings {
    fn check(&self) -> Result<(), SettingsError> {
        self.check_numbers()?;
        // `contains` is false for NaN too.
        within("min_contrast", self.min_contrast, &MIN_CONTRAST)?;
        if self.language != AUTO && find(&self.language).is_none() {
            return Err(invalid(NO_LANGUAGE));
        }
        let ids = theme_ids().map_err(invalid)?;
        if !ids.contains(&self.theme) {
            return Err(invalid(&format!("theme must be one of {}", ids.join(", "))));
        }
        Ok(())
    }
}

/// Refuses `value` of setting `key` outside `range`.
fn within<T: Copy + PartialOrd + Display>(
    key: &str,
    value: T,
    range: &RangeInclusive<T>,
) -> Result<(), SettingsError> {
    if range.contains(&value) {
        return Ok(());
    }
    let (lo, hi) = (range.start(), range.end());
    Err(invalid(&format!("{key} must be from {lo} to {hi}")))
}

/// The shipped theme ids, read from the compiled-in tokens once per run; the error names what broke.
fn theme_ids() -> Result<&'static [String], &'static str> {
    static IDS: OnceLock<Result<Vec<String>, String>> = OnceLock::new();
    let ids = IDS.get_or_init(|| {
        let themes =
            Themes::shipped().map_err(|e| format!("the shipped themes do not load: {e}"))?;
        Ok(themes.list.into_iter().map(|t| t.look.id).collect())
    });
    ids.as_deref().map_err(String::as_str)
}

/// A broken rule as a settings error.
fn invalid(why: &str) -> SettingsError {
    SettingsError::Invalid(why.to_owned())
}

#[cfg(test)]
mod tests;
