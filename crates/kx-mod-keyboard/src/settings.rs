//! The `[keyboard]` settings: modifiers, the language key, key repeat, size, place, the bubble corner, the
//! shortcut pages and the text cap of a picked word.

pub use kx_module_api::ports::Corner;

use std::collections::HashSet;
use std::ops::RangeInclusive;

use crate::spec::BoardSpec;
use kx_module_api::ports::{MAX_PRESETS, Point, ShortcutId};
use kx_module_api::{Settings, SettingsError, SettingsSpec, validate_as, within};
use serde::{Deserialize, Serialize};

/// How often the labels may follow the app in front, in ms. The floor spares CPU time, and the ceiling keeps a
/// hand-edited value from stalling them.
pub(crate) const RELABEL_MS: RangeInclusive<u64> = 50..=10_000;
/// How long a key is held before it repeats, in ms: a slow click never repeats, and a held key never waits for long.
pub(crate) const REPEAT_DELAY_MS: RangeInclusive<u64> = 300..=3_000;
/// The time between repeats, in ms: from about 30 a second to one a second.
pub(crate) const REPEAT_EVERY_MS: RangeInclusive<u64> = 33..=1_000;
/// The most characters a picked word may erase or type: room for the longest word with chips and its gap (the app ties
/// it to `[predict] max_word`) or a long email, and never a page of text.
pub const TEXT_CAP: RangeInclusive<u16> = 128..=1_024;
/// The smallest `size_min` the settings take: the lowest size step where every key cell is still 24 px across
/// (the spacing rule of WCAG 2.2 SC 2.5.8), so a hand-edited size cannot make the keys too small to click.
pub(crate) const SIZE_FLOOR: f64 = 0.8;
/// The largest `size_max` the settings take, so a hand-edited size cannot open the keyboard far past every screen.
pub(crate) const SIZE_CEILING: f64 = 4.0;
/// The most shortcut pages: the Next key shows one dot per page, and they must fit one line inside its cap in every
/// theme, which the app's `agree` test checks.
pub const MAX_PAGES: usize = 6;

/// Why a size was refused: it is not a finite number.
const NOT_A_NUMBER: &str = "sizes must be numbers";
/// Why the limits were refused.
const BAD_RANGE: &str = "size_min must be above 0 and below size_max";
/// Why the step was refused.
const BAD_STEP: &str = "size_step must be above 0 and no wider than size_max - size_min";
/// Why the presets were refused.
const BAD_PRESETS: &str =
    "presets must rise, one to as many as the top bar names, between size_min and size_max";
/// Why the size was refused.
const BAD_SIZE: &str = "size must be between size_min and size_max";
/// Why an app name was refused.
const BLANK_APP: &str = "key_apps: an app name is blank";
/// Why the shortcut pages were refused: too many.
const BAD_PAGE_COUNT: &str = "shortcut_pages: more pages than the Next key's dots show";
/// Why a shortcut page was refused: it is empty.
const BAD_EMPTY_PAGE: &str = "shortcut_pages: a page is empty";
/// Why a shortcut page was refused: the side block has no room for it.
const BAD_FULL_PAGE: &str =
    "shortcut_pages: a page holds more shortcuts than the side block has cells";
/// Why the shortcut pages were refused: an id comes twice.
const BAD_TWICE: &str = "shortcut_pages: a shortcut is listed twice";

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
    /// Held typing keys repeat.
    pub repeat: bool,
    /// How long a key is held before it repeats, in ms.
    pub repeat_delay_ms: u64,
    /// The time between repeats, in ms.
    pub repeat_every_ms: u64,
    /// Apps that get text as key presses in their own layout, by file name.
    pub key_apps: Vec<String>,
    /// The most characters a picked word may erase or type; a longer request types nothing.
    pub text_cap: u16,
    /// The saved left edge, in physical pixels.
    pub x: i32,
    /// The saved top edge, in physical pixels.
    pub y: i32,
    /// The user has put the keyboard somewhere, so `x` and `y` hold.
    pub placed: bool,
    /// The bubble's corner.
    pub bubble_corner: Corner,
    /// The shortcuts layer's pages in order, by shortcut id.
    pub shortcut_pages: Vec<Vec<ShortcutId>>,
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
        if self.size_min < SIZE_FLOOR || self.size_max > SIZE_CEILING {
            let (low, high) = (SIZE_FLOOR, SIZE_CEILING);
            return Err(invalid(&format!(
                "size_min and size_max must be from {low} to {high}"
            )));
        }
        within("relabel_ms", self.relabel_ms, &RELABEL_MS)?;
        within("repeat_delay_ms", self.repeat_delay_ms, &REPEAT_DELAY_MS)?;
        within("repeat_every_ms", self.repeat_every_ms, &REPEAT_EVERY_MS)?;
        within("text_cap", self.text_cap, &TEXT_CAP)?;
        if self.key_apps.iter().any(|a| a.trim().is_empty()) {
            return Err(invalid(BLANK_APP));
        }
        self.check_pages().map_err(invalid)
    }
}

impl KeyboardSettings {
    /// Every size is a number, the limits make a range above zero, the step fits it, and the rest sits within it.
    fn check_sizes(&self) -> Result<(), &'static str> {
        let all = [self.size, self.size_step, self.size_min, self.size_max];
        if !all.iter().chain(&self.presets).all(|v| v.is_finite()) {
            return Err(NOT_A_NUMBER);
        }
        if !(self.size_min > 0.0 && self.size_min < self.size_max) {
            return Err(BAD_RANGE);
        }
        let step = self.size_step;
        if !(step > 0.0 && step <= self.size_max - self.size_min && step.recip().is_finite()) {
            return Err(BAD_STEP);
        }
        let within = |v: &f64| (self.size_min..=self.size_max).contains(v);
        let count = (1..=MAX_PRESETS).contains(&self.presets.len());
        let rising = self.presets.windows(2).all(|p| p[0] < p[1]);
        if !count || !rising || !self.presets.iter().all(within) {
            return Err(BAD_PRESETS);
        }
        if !within(&self.size) {
            return Err(BAD_SIZE);
        }
        Ok(())
    }

    /// At most `MAX_PAGES` pages, none empty or past the layer's room on the shipped board, and no id twice.
    /// A board that does not read has no room, though its own check stops the module first.
    fn check_pages(&self) -> Result<(), &'static str> {
        let pages = &self.shortcut_pages;
        if pages.len() > MAX_PAGES {
            return Err(BAD_PAGE_COUNT);
        }
        let room = BoardSpec::shipped().map_or(0, |b| b.layer_grid().capacity(pages.len()));
        let mut seen = HashSet::new();
        for page in pages {
            if page.is_empty() {
                return Err(BAD_EMPTY_PAGE);
            }
            if page.len() > room {
                return Err(BAD_FULL_PAGE);
            }
            if !page.iter().all(|id| seen.insert(id)) {
                return Err(BAD_TWICE);
            }
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
