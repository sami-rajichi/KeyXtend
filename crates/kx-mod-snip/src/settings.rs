//! The `[snip]` settings (spec "Settings `[snip]`"): the capture, the Save folder, a video's countdown, rate, length
//! and sounds, the panel's steps, the copied videos' days, the caps, the jobs' waits, Undo, a stroke's points, Copy's
//! clipboard wait, a tick's and a note's time, the wait for the keyboard's step aside, a frame's smallest side, and the
//! editor's place and maximised work area. The ports' ranges are theirs.

mod jobs;
mod place;

pub use jobs::{ENCODE_IDLE_S, ENCODE_MB_MS, ENCODE_MS, ENCODE_RESTART_MS, EncodeWaits};
pub(crate) use place::Written;

use std::ops::RangeInclusive;
use std::time::Duration;

use kx_module_api::ports::{ClipWait, RecordAsk, Rect, SHOT_CAP_MB, ShotCap, SnipKeyboard};
use kx_module_api::{Settings, SettingsError, SettingsSpec, refuse_if, validate_as, within};
use serde::{Deserialize, Serialize};

/// The seconds a video counts down before it records; 0 records at once.
pub const COUNTDOWN_S: RangeInclusive<u8> = 0..=10;
/// The countdown's step on the Snip panel, in seconds.
pub const COUNTDOWN_STEP_S: RangeInclusive<u8> = 1..=5;
/// The longest video's step on the Snip panel, in minutes.
pub const MAX_MIN_STEP: RangeInclusive<u16> = 1..=30;
/// The days a copied video stays in the Snips folder's unsaved part.
pub const COPIED_DAYS: RangeInclusive<u8> = 1..=7;
/// The most steps the editor's Undo takes back.
pub const UNDO_MAX: RangeInclusive<u16> = 10..=200;
/// The most points one pen or highlighter stroke keeps; a longer one keeps its first points.
pub const STROKE_MAX: RangeInclusive<u32> = 100..=100_000;
/// The longest Copy waits for another app to let go of the clipboard, in ms, as voice typing's Copy.
pub const COPY_MS: RangeInclusive<u32> = 10..=2_000;
/// How often Copy tries again within that wait, in ms.
pub const COPY_POLL_MS: RangeInclusive<u32> = 1..=100;
/// How many times Copy tries a busy clipboard again before its note.
pub const COPY_AGAIN: RangeInclusive<u8> = 0..=5;
/// The pause before each new try, in ms.
pub const COPY_AGAIN_MS: RangeInclusive<u32> = 100..=5_000;
/// How long a note shows, in seconds.
pub const NOTE_S: RangeInclusive<u8> = 2..=30;
/// How long the tick of a Copy or Save shows, in ms.
pub const SHOWN_MS: RangeInclusive<u32> = 300..=5_000;
/// How long the freeze waits for the ui to say the keyboard stepped aside, in ms.
pub const ASIDE_MS: RangeInclusive<u32> = 100..=2_000;
/// The smallest side a frame may keep, in physical pixels.
pub const FRAME_MIN_PX: RangeInclusive<u16> = 1..=64;

/// The keys the module writes back itself, from the Snip panel and the editor's place.
pub(crate) mod keys {
    /// `keyboard`.
    pub(crate) const KEYBOARD: &str = "keyboard";
    /// `save_dir`.
    pub(crate) const SAVE_DIR: &str = "save_dir";
    /// `countdown_s`.
    pub(crate) const COUNTDOWN_S: &str = "countdown_s";
    /// `fps`.
    pub(crate) const FPS: &str = "fps";
    /// `max_min`.
    pub(crate) const MAX_MIN: &str = "max_min";
    /// `mic_on`.
    pub(crate) const MIC_ON: &str = "mic_on";
    /// `sound_on`.
    pub(crate) const SOUND_ON: &str = "sound_on";
    /// `editor`.
    pub(crate) const EDITOR: &str = "editor";
    /// `editor_full`.
    pub(crate) const EDITOR_FULL: &str = "editor_full";
}

/// Why a placed editor was refused.
const BAD_EDITOR: &str = "editor must be empty or a rectangle with a width and a height";
/// Why a maximised editor's work area was refused.
const BAD_FULL: &str = "editor_full must be empty or a rectangle with a width and a height";
/// Why a maximised editor with no box to come back to was refused.
const BARE_FULL: &str = "editor_full needs an editor box to come back to";
/// Why Copy's poll was refused.
const BAD_POLL: &str = "copy_poll_ms must not pass copy_ms";

/// How the kernel reads the `[snip]` section: version 1, with the defaults embedded.
pub static SPEC: SettingsSpec = SettingsSpec {
    version: 1,
    defaults: include_str!("../defaults.toml"),
    migrations: &[],
    validate: validate_as::<SnipSettings>,
};

/// The module's settings; every value comes from `defaults.toml` or the user's file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnipSettings {
    /// What the keyboard does during a capture.
    pub keyboard: SnipKeyboard,
    /// The folder the Save dialog opens at, kept as typed and checked when used; empty means the Snips folder.
    pub save_dir: String,
    /// The seconds a video counts down before it records.
    pub countdown_s: u8,
    /// The countdown's step on the Snip panel, in seconds.
    pub countdown_step_s: u8,
    /// A video's frames a second, one of `RecordAsk::FPS`.
    pub fps: u8,
    /// The longest video, in minutes.
    pub max_min: u16,
    /// The longest video's step on the Snip panel, in minutes.
    pub max_min_step: u16,
    /// The microphone records from a video's start.
    pub mic_on: bool,
    /// What the computer plays records from a video's start.
    pub sound_on: bool,
    /// The days a copied video stays in the unsaved part.
    pub copied_days: u8,
    /// The room a video's drive keeps free, in MB.
    pub min_free_mb: u32,
    /// The most steps Undo takes back.
    pub undo_max: u16,
    /// The most points one stroke keeps.
    pub stroke_max: u32,
    /// The largest frozen copy of the screens, in MB.
    pub shot_cap_mb: u32,
    /// The recording job's memory cap, in MB.
    pub record_mb: u32,
    /// The longest a recording's job may take to say it started, in ms.
    pub record_start_ms: u32,
    /// The longest a recording may stay silent, in ms.
    pub record_tick_ms: u32,
    /// The longest a recording's job may take to end after Stop, or to quit, in ms.
    pub record_end_ms: u32,
    /// The longest a switch waits for the recording's answer, in ms.
    pub switch_ms: u32,
    /// A speed copy's least deadline, in ms.
    pub speed_ms: u32,
    /// A speed copy's more deadline for each MB of its video, in ms.
    pub speed_mb_ms: u32,
    /// The dialogs' job memory cap, in MB.
    pub pick_mb: u32,
    /// How long a dialog may stay open, in minutes.
    pub dialog_min: u16,
    /// How long the dialogs' job waits for the next dialog before it stops, in seconds.
    pub pick_idle_s: u16,
    /// The wait before a failed dialogs' job starts again, in ms.
    pub pick_restart_ms: u32,
    /// The longest such wait, in ms.
    pub pick_restart_max_ms: u32,
    /// The least time a picture's PNG may take in the media worker, in ms.
    pub encode_ms: u32,
    /// The more time a PNG may take for each MB of its pixels, in ms.
    pub encode_mb_ms: u32,
    /// How long the media worker may idle after a PNG, in seconds.
    pub encode_idle_s: u32,
    /// The wait before a failed media worker starts again, in ms.
    pub encode_restart_ms: u32,
    /// The longest Copy waits for the clipboard, in ms.
    pub copy_ms: u32,
    /// How often Copy tries again within that wait, in ms.
    pub copy_poll_ms: u32,
    /// How many times Copy tries a busy clipboard again before its note.
    pub copy_again: u8,
    /// The pause before each new try, in ms.
    pub copy_again_ms: u32,
    /// How long a note shows, in seconds.
    pub note_s: u8,
    /// How long the tick of a Copy or Save shows, in ms.
    pub shown_ms: u32,
    /// How long the freeze waits for the ui to say the keyboard stepped aside, in ms; then it freezes anyway.
    pub aside_ms: u32,
    /// The smallest side a frame may keep once a handle moves, in physical pixels.
    pub frame_min_px: u16,
    /// The editor's last size and place, in physical pixels; `None`, written as an empty table, until first placed.
    #[serde(with = "place")]
    pub editor: Option<Rect>,
    /// The work area the editor filled when last maximised, over `editor`; `None` when it is not maximised.
    #[serde(with = "place")]
    pub editor_full: Option<Rect>,
}

impl SnipSettings {
    /// The settings in the embedded `defaults.toml`.
    ///
    /// # Errors
    /// `SettingsError::Parse` when the defaults do not parse, or the error from `check`.
    pub fn from_defaults() -> Result<Self, SettingsError> {
        SPEC.defaults_as()
    }

    /// The freeze's cap as the port's checked type; `None` only for a `shot_cap_mb` that `check` refuses.
    #[must_use]
    pub fn shot_cap(&self) -> Option<ShotCap> {
        ShotCap::new(self.shot_cap_mb)
    }

    /// How long Copy waits for the clipboard.
    #[must_use]
    pub fn copy_wait(&self) -> ClipWait {
        ClipWait {
            open_ms: self.copy_ms,
            poll_ms: self.copy_poll_ms,
        }
    }

    /// How long the tick of a Copy or Save shows.
    #[must_use]
    pub fn shown(&self) -> Duration {
        Duration::from_millis(u64::from(self.shown_ms))
    }

    /// The countdown, the frame rate, the longest video and the panel's steps for them.
    fn check_video(&self) -> Result<(), SettingsError> {
        within("countdown_s", self.countdown_s, &COUNTDOWN_S)?;
        within("countdown_step_s", self.countdown_step_s, &COUNTDOWN_STEP_S)?;
        if !RecordAsk::FPS.contains(&self.fps) {
            let why = format!("fps must be one of {:?}", RecordAsk::FPS);
            return Err(SettingsError::Invalid(why));
        }
        within("max_min", self.max_min, &RecordAsk::MAX_MIN)?;
        within("max_min_step", self.max_min_step, &MAX_MIN_STEP)
    }

    /// The editor's box and a maximised editor's work area each have a width and a height, and that work area lies over
    /// a box.
    fn check_editor(&self) -> Result<(), SettingsError> {
        let flat = |place: Option<Rect>| place.is_some_and(|r| r.w <= 0 || r.h <= 0);
        refuse_if(flat(self.editor), BAD_EDITOR)?;
        refuse_if(flat(self.editor_full), BAD_FULL)?;
        refuse_if(
            self.editor_full.is_some() && self.editor.is_none(),
            BARE_FULL,
        )
    }
}

impl Settings for SnipSettings {
    /// Each range, the frame rate's choices, then the editor's place.
    fn check(&self) -> Result<(), SettingsError> {
        self.check_video()?;
        within("copied_days", self.copied_days, &COPIED_DAYS)?;
        within("min_free_mb", self.min_free_mb, &RecordAsk::MIN_FREE_MB)?;
        within("undo_max", self.undo_max, &UNDO_MAX)?;
        within("stroke_max", self.stroke_max, &STROKE_MAX)?;
        within("shot_cap_mb", self.shot_cap_mb, &SHOT_CAP_MB)?;
        within("record_mb", self.record_mb, &RecordAsk::JOB_MB)?;
        within("copy_ms", self.copy_ms, &COPY_MS)?;
        within("copy_poll_ms", self.copy_poll_ms, &COPY_POLL_MS)?;
        within("copy_again", self.copy_again, &COPY_AGAIN)?;
        within("copy_again_ms", self.copy_again_ms, &COPY_AGAIN_MS)?;
        within("note_s", self.note_s, &NOTE_S)?;
        within("shown_ms", self.shown_ms, &SHOWN_MS)?;
        within("aside_ms", self.aside_ms, &ASIDE_MS)?;
        within("frame_min_px", self.frame_min_px, &FRAME_MIN_PX)?;
        refuse_if(self.copy_poll_ms > self.copy_ms, BAD_POLL)?;
        self.check_jobs()?;
        self.check_editor()
    }
}

#[cfg(test)]
mod tests;
