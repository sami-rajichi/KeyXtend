//! The `[keyboard]` settings: modifiers, the language key, size, place and the bubble corner.

pub use kx_module_api::ports::Corner;

use std::ops::RangeInclusive;

use kx_module_api::ports::{MAX_PRESETS, Point};
use kx_module_api::{Settings, SettingsError, SettingsSpec, validate_as};
use serde::{Deserialize, Serialize};

/// How often the labels may follow the app in front, in ms. The floor spares CPU time, and the ceiling keeps a
/// hand-edited value from stalling them.
pub(crate) const RELABEL_MS: RangeInclusive<u64> = 50..=10_000;

/// Why a size was refused: it is not a finite number.
const NOT_A_NUMBER: &str = "sizes must be numbers";
/// Why the limits were refused.
const BAD_RANGE: &str = "size_min must be above 0 and below size_max";
/// Why the step was refused.
const BAD_STEP: &str = "size_step must be above 0";
/// Why the presets were refused.
const BAD_PRESETS: &str = "presets must be at least one size, no more than the top bar names, between size_min and size_max";
/// Why the size was refused.
const BAD_SIZE: &str = "size must be between size_min and size_max";
/// Why an app name was refused.
const BLANK_APP: &str = "key_apps: an app name is blank";

/// How the kernel reads the `[keyboard]` section: version 1, with the defaults embedded.
pub static SPEC: SettingsSpec = SettingsSpec {
    version: 1,
    defaults: include_str!("../defaults.toml"),
    migrations: &[],
    validate: validate_as::<KeyboardSettings>,
};

/// When latched modifiers let go.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Modifiers {
    /// After the next key.
    Release,
    /// Until the modifier is clicked again.
    Stay,
}

/// How the language key shows the layouts.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LangKey {
    /// The layouts before, now and after, turning like a wheel.
    Carousel,
    /// The current layout, stepping to the next on each click.
    Cycle,
    /// A list of every layout to pick from.
    List,
}

/// The module's settings; every value comes from `defaults.toml` or the user's file.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyboardSettings {
    /// When latched modifiers let go.
    pub modifiers: Modifiers,
    /// How the language key shows the layouts.
    pub lang_key: LangKey,
    /// The size, as a factor of the size-1 board.
    pub size: f64,
    /// The S, M and L sizes.
    pub presets: Vec<f64>,
    /// Change per smaller or bigger click.
    pub size_step: f64,
    /// The smallest size.
    pub size_min: f64,
    /// The largest size.
    pub size_max: f64,
    /// How often the labels follow the layout of the app in front, in ms.
    pub relabel_ms: u64,
    /// Apps that get text as key presses in their own layout, by file name.
    pub key_apps: Vec<String>,
    /// The saved left edge, in physical pixels.
    pub x: i32,
    /// The saved top edge, in physical pixels.
    pub y: i32,
    /// The user has put the keyboard somewhere, so `x` and `y` hold.
    pub placed: bool,
    /// The bubble's corner.
    pub bubble_corner: Corner,
}

impl KeyboardSettings {
    /// The settings in the embedded `defaults.toml`.
    ///
    /// # Errors
    /// `SettingsError::Parse` when the defaults do not parse, or the error from `check`.
    pub fn from_defaults() -> Result<Self, SettingsError> {
        SPEC.defaults_as()
    }

    /// Where the user last put the keyboard, if anywhere.
    #[must_use]
    pub fn place(&self) -> Option<Point> {
        self.placed.then_some(Point {
            x: self.x,
            y: self.y,
        })
    }
}

impl Settings for KeyboardSettings {
    fn check(&self) -> Result<(), SettingsError> {
        self.check_sizes().map_err(invalid)?;
        if !RELABEL_MS.contains(&self.relabel_ms) {
            let (low, high) = (RELABEL_MS.start(), RELABEL_MS.end());
            return Err(invalid(&format!("relabel_ms must be from {low} to {high}")));
        }
        if self.key_apps.iter().any(|a| a.trim().is_empty()) {
            return Err(invalid(BLANK_APP));
        }
        Ok(())
    }
}

impl KeyboardSettings {
    /// Every size is a number, the limits make a range above zero, and the rest sits within it.
    fn check_sizes(&self) -> Result<(), &'static str> {
        let all = [self.size, self.size_step, self.size_min, self.size_max];
        if !all.iter().chain(&self.presets).all(|v| v.is_finite()) {
            return Err(NOT_A_NUMBER);
        }
        if !(self.size_min > 0.0 && self.size_min < self.size_max) {
            return Err(BAD_RANGE);
        }
        if self.size_step <= 0.0 {
            return Err(BAD_STEP);
        }
        let within = |v: &f64| (self.size_min..=self.size_max).contains(v);
        let count = (1..=MAX_PRESETS).contains(&self.presets.len());
        if !count || !self.presets.iter().all(within) {
            return Err(BAD_PRESETS);
        }
        if !within(&self.size) {
            return Err(BAD_SIZE);
        }
        Ok(())
    }
}

/// A broken rule as a settings error.
fn invalid(why: &str) -> SettingsError {
    SettingsError::Invalid(why.to_owned())
}

#[cfg(test)]
mod tests;
