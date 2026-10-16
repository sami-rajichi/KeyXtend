//! The `[ui]` settings: the look, the language, tips, timings, paging and sounds, with every number's limits in one
//! place.

use crate::harden::QT_POOL_BYTES;
use crate::i18n::{AUTO, find};
use kx_module_api::{Settings, SettingsError, SettingsSpec, validate_as, within};
use kx_platform::ui::BackdropKind;
use kx_theme::contrast::MIN_CONTRAST;
use kx_theme::{LookChoice, ModeChoice, Themes};
use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;
use std::sync::OnceLock;

/// How long the pointer may rest before a tip shows, in ms.
pub(crate) const TIP_MS: RangeInclusive<u64> = 1..=10_000;
/// How often a move or resize, or the pointer overlay, may follow the pointer, in ms.
pub(crate) const POLL_MS: RangeInclusive<u64> = 1..=1_000;
/// How long the window guard may wait before each try, in ms.
pub(crate) const GUARD_DELAY_MS: RangeInclusive<u64> = 1..=10_000;
/// How many times the window guard may try.
pub(crate) const GUARD_TRIES: RangeInclusive<u32> = 1..=100;
/// How many times a second copy may try to reach the running one.
pub(crate) const SEND_TRIES: RangeInclusive<u32> = 1..=100;
/// How long a second copy may wait between tries, in ms.
pub(crate) const SEND_WAIT_MS: RangeInclusive<u64> = 1..=10_000;
/// How long the tray menu may ignore a new front window after it opens, in ms; 0 turns that off.
pub(crate) const TRAY_GRACE_MS: RangeInclusive<u64> = 0..=5_000;
/// How many times one run may make the windows again after the graphics device is lost; 0 never does.
pub(crate) const DEVICE_REBUILDS: RangeInclusive<u32> = 0..=10;
/// How long one may wait before trying again at a graphics device that could not be made, in ms.
pub(crate) const DEVICE_RETRY_MS: RangeInclusive<u64> = 0..=5_000;
/// How many frames Qt lets queue before it waits at a frame's start; 0 drops that wait, not Direct3D's own queue.
pub(crate) const FRAME_LATENCY: RangeInclusive<u32> = 0..=3;
/// How long Qt's update timer may wait before a frame, in ms; 0 starts it at once.
pub(crate) const UPDATE_IDLE_MS: RangeInclusive<u64> = 0..=50;
/// How many bytes of each buffer kind a window's renderer may keep to reuse: none, up to four times Qt's own.
pub(crate) const BUFFER_POOL_BYTES: RangeInclusive<u32> = 0..=4 * QT_POOL_BYTES;
/// How long a press may wait for its frame before it types anyway, in ms.
pub(crate) const PRESS_FRAME_MS: RangeInclusive<u64> = 1..=1_000;
/// How many rows a list page may show; every list is paged (ADR-0008).
pub(crate) const PAGE_ROWS: RangeInclusive<usize> = 1..=10;
/// Our volume, in percent of each sound's own level; the end plays it as made, and 0 is silent.
pub(crate) const SOUND_VOLUME: RangeInclusive<u8> = 0..=100;
/// How long the speaker stays ready after the last sound, in ms. The floor is twice the sum of the longest sound
/// `sounds.toml` allows and about 100 ms of device latency, as 500 ms would only just equal that sum.
pub(crate) const SOUND_IDLE_MS: RangeInclusive<u32> = 1_000..=60_000;

/// Why `language` was refused.
const NO_LANGUAGE: &str = "language must be auto or a language the keyboard speaks";

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

/// When a pressed key types, against the frame that shows it pressed (ADR-0024 keys act at the press).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PressFrame {
    /// It types at once, and the press shows in the next frame after.
    After,
    /// It types once the frame showing the press is out, or after `press_frame_ms`.
    First,
    /// The frame showing the press is drawn at once, then it types.
    Now,
}

/// The ui module's settings; every value comes from `defaults.toml` or the user's file.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "Each flag is one on-or-off setting of the flat [ui] section the user reads and writes."
)]
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
    /// The language that pins the keyboard's texts, or `auto` to follow the layout in front (ADR-0020).
    pub language: String,
    /// How long the pointer rests before a tip shows, in ms.
    pub tip_ms: u64,
    /// How often a move or resize follows the pointer, and the least time between the pointer overlay's reads, in ms.
    pub poll_ms: u64,
    /// How long the window guard waits before each try, in ms.
    pub guard_delay_ms: u64,
    /// How many times the window guard tries.
    pub guard_tries: u32,
    /// How many times a second copy tries to hand its command to the running one.
    pub send_tries: u32,
    /// How long a second copy waits between tries, in ms.
    pub send_wait_ms: u64,
    /// How long the tray menu ignores a new front window after it opens, in ms, as the hidden icons' pop-up closing
    /// can bring one.
    pub tray_grace_ms: u64,
    /// How many times one run makes the windows again on a new graphics device after the device is lost (ADR-0021).
    pub device_rebuilds: u32,
    /// How long to wait before trying again at a new graphics device that could not be made, as a driver resets, in ms.
    pub device_retry_ms: u64,
    /// How many frames Qt lets queue before it waits at a frame's start (`QT_D3D_MAX_FRAME_LATENCY`); 0 drops that wait.
    pub frame_latency: u32,
    /// Frames start on the screen's refresh (Qt's vblank thread); false starts them on Qt's update timer.
    pub vblank_updates: bool,
    /// Qt's update timer, in ms, for windows whose frames do not follow the refresh (`QT_QPA_UPDATE_IDLE_TIME`).
    pub update_idle_ms: u64,
    /// Each frame shows at a screen refresh (swap interval 1); false shows it at once (0).
    pub present_vsync: bool,
    /// How many bytes of freed vertex buffers, and as many of index buffers, each window's renderer keeps to reuse
    /// (`QSG_RENDERER_BUFFER_POOL_LIMIT`, ADR-0028); 0 keeps none.
    pub buffer_pool_bytes: u32,
    /// When a pressed key types, against the frame that shows it pressed.
    pub press_frame: PressFrame,
    /// The longest `PressFrame::First` waits for that frame, in ms.
    pub press_frame_ms: u64,
    /// Each press and frame logs its times, never a key or text (`FrameLog`).
    pub frame_log: bool,
    /// Rows per page in every list.
    pub page_rows: usize,
    /// The contrast a legend keeps when its key takes the accent.
    pub min_contrast: f32,
    /// A key press sounds.
    pub sound_key: bool,
    /// A hold's right-click sounds.
    pub sound_rclick: bool,
    /// Grab's pick-up sounds.
    pub sound_pick: bool,
    /// Grab's drop sounds.
    pub sound_drop: bool,
    /// Our volume for every sound, in percent of its own level, under the Windows volume.
    pub sound_volume: u8,
    /// How long the speaker stays ready after the last sound, in ms; the next sound after that wakes it.
    pub sound_idle_ms: u32,
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
        within("send_tries", self.send_tries, &SEND_TRIES)?;
        within("send_wait_ms", self.send_wait_ms, &SEND_WAIT_MS)?;
        within("tray_grace_ms", self.tray_grace_ms, &TRAY_GRACE_MS)?;
        within("device_rebuilds", self.device_rebuilds, &DEVICE_REBUILDS)?;
        within("device_retry_ms", self.device_retry_ms, &DEVICE_RETRY_MS)?;
        within("frame_latency", self.frame_latency, &FRAME_LATENCY)?;
        within("update_idle_ms", self.update_idle_ms, &UPDATE_IDLE_MS)?;
        within(
            "buffer_pool_bytes",
            self.buffer_pool_bytes,
            &BUFFER_POOL_BYTES,
        )?;
        within("press_frame_ms", self.press_frame_ms, &PRESS_FRAME_MS)?;
        within("sound_volume", self.sound_volume, &SOUND_VOLUME)?;
        within("sound_idle_ms", self.sound_idle_ms, &SOUND_IDLE_MS)?;
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
