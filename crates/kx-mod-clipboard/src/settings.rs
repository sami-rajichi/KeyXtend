//! The `[clipboard]` settings (spec §7, plan Task 14, plus the insert and worker waits the build added): the limits,
//! the insert's joins and waits, the history shortcut, the texts' lengths, the media worker and the clipboard waits.
//! Each range is a named const.

mod parts;

use std::fmt;
use std::ops::RangeInclusive;

use kx_module_api::ports::{MB, RGBA_BYTES, WORKER_DECODE_BYTES, WORKER_READ_TIMES};
use kx_module_api::{Redacted, Settings, SettingsError, SettingsSpec, validate_as, within};
use serde::{Deserialize, Serialize};

/// Calendar days an entry, or a media entry, is kept.
pub const KEEP_DAYS: RangeInclusive<u8> = 1..=7;
/// The most entries.
pub const MAX_ENTRIES: RangeInclusive<usize> = 1..=5_000;
/// The whole history's cap, in MB.
pub const MAX_MB: RangeInclusive<u32> = 64..=16_384;
/// The free space the drive keeps, in MB.
pub const MIN_FREE_MB: RangeInclusive<u32> = 256..=65_536;
/// One entry's cap, in MB; the worker's pipe carries at most the top.
pub const MAX_ENTRY_MB: RangeInclusive<u32> = 1..=64;
/// Seconds a clear follows a copy.
pub const CLEAR_FOLLOW_S: RangeInclusive<u32> = 10..=600;
/// How many skipped apps may be listed.
pub const SKIP_APPS: RangeInclusive<usize> = 0..=100;
/// The characters of one skipped app's id: a file name's most.
pub const APP_CHARS: RangeInclusive<usize> = 1..=255;
/// Line breaks between two inserted texts.
pub const JOIN_BREAKS: RangeInclusive<u8> = 1..=3;
/// The most characters of the separator line.
pub const JOIN_RULE_CHARS: usize = 80;
/// The wait after each pasted step, in ms.
pub const PASTE_GAP_MS: RangeInclusive<u32> = 20..=2_000;
/// The wait for the keyboard's paste, in ms.
pub const PASTE_WAIT_MS: RangeInclusive<u32> = 100..=5_000;
/// The wait for an insert's file check, in ms.
pub const CHECK_MS: RangeInclusive<u32> = 1_000..=30_000;
/// The characters search reads.
pub const SEARCH_CHARS: RangeInclusive<usize> = 100..=2_000;
/// The characters a title keeps.
pub const TITLE_CHARS: RangeInclusive<usize> = 40..=600;
/// The characters the preview shows.
pub const PREVIEW_CHARS: RangeInclusive<usize> = 1_000..=100_000;
/// A thumbnail's longest side, in pixels.
pub const THUMB_PX: RangeInclusive<u32> = 32..=512;
/// The preview image's longest side, in pixels, up to the worker's pipe's longest side.
pub const PREVIEW_PX: RangeInclusive<u32> = 256..=4_096;
/// The most pixels an image may hold; the top fits `worker_mb`'s default as the worker scales while reading.
pub const MAX_PIXELS: RangeInclusive<u64> = 1_000_000..=100_000_000;
/// The longest one picture may take, in ms.
pub const MEDIA_MS: RangeInclusive<u32> = 500..=30_000;
/// Seconds the worker may sit idle.
pub const WORKER_IDLE_S: RangeInclusive<u32> = 5..=3_600;
/// The worker's memory cap, in MB.
pub const WORKER_MB: RangeInclusive<u32> = 64..=4_096;
/// The first wait before a failed worker starts again, in ms.
pub const RESTART_MS: RangeInclusive<u32> = 100..=30_000;
/// The longest wait before a failed worker starts again, in ms.
pub const RESTART_MAX_MS: RangeInclusive<u32> = 1_000..=600_000;
/// The longest wait for the clipboard, in ms.
pub const OPEN_MS: RangeInclusive<u32> = 10..=2_000;
/// How often the clipboard is tried within that wait, in ms.
pub const POLL_MS: RangeInclusive<u32> = 1..=100;
/// The most copies that wait for the history to keep them.
pub const QUEUE_COPIES: RangeInclusive<usize> = 1..=64;
/// The save's wait after a first change, in ms.
pub const SAVE_MS: RangeInclusive<u32> = 100..=60_000;
/// The worker's memory beside its decoder, in MB: its code, stacks and pipe buffers (`keyxtend-worker.md`).
pub const WORKER_OWN_MB: u64 = 16;
/// The worker's own memory, in MB: its decoder's budget, rounded up, and the rest of it.
pub const WORKER_BASE_MB: u64 = (WORKER_DECODE_BYTES as u64).div_ceil(MB) + WORKER_OWN_MB;
/// How many times one entry's bytes the worker holds while it reads them, as kx-ipc's reader grows.
const READ: u64 = WORKER_READ_TIMES as u64;

/// The line and paragraph separators, which `char::is_control` lets through.
const BREAKS: [char; 2] = ['\u{2028}', '\u{2029}'];

/// Why a list or a pair was refused.
const BAD_APP: &str = "skip_apps must each be an app id on one line";
const BAD_RULE: &str = "join_rule must be one line within its length";
const BAD_POLL: &str = "poll_ms must not pass open_ms";
const BAD_RESTART: &str = "restart_ms must not pass restart_max_ms";
const BAD_WORKER: &str =
    "worker_mb must hold the largest entry as it is read, a preview or its DIB, and the worker";

/// How the kernel reads the `[clipboard]` section: version 1, with the defaults embedded.
pub static SPEC: SettingsSpec = SettingsSpec {
    version: 1,
    defaults: include_str!("../defaults.toml"),
    migrations: &[],
    validate: validate_as::<ClipSettings>,
};

/// The module's settings; every value comes from `defaults.toml` or the user's file.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClipSettings {
    /// Calendar days an entry is kept.
    pub keep_days: u8,
    /// Calendar days a video or audio entry is kept.
    pub media_days: u8,
    /// The most entries kept.
    pub max_entries: usize,
    /// The most the history keeps, in MB.
    pub max_mb: u32,
    /// The free space the drive keeps, in MB.
    pub min_free_mb: u32,
    /// The most one entry keeps, in MB.
    pub max_entry_mb: u32,
    /// Seconds after a copy within which its app's clear deletes it.
    pub clear_follow_s: u32,
    /// The ids of apps whose copies are never kept.
    pub skip_apps: Vec<String>,
    /// Line breaks pasted between two inserted texts.
    pub join_breaks: u8,
    /// A separator line pasted between two inserted texts; empty for none.
    pub join_rule: String,
    /// The wait after each pasted step, in ms.
    pub paste_gap_ms: u32,
    /// How long an insert waits for the keyboard to paste, in ms.
    pub paste_wait_ms: u32,
    /// How long an insert waits for its files to be checked, in ms.
    pub check_ms: u32,
    /// The system's clipboard-history shortcut opens our panel.
    pub history_key: bool,
    /// The characters of each text search reads.
    pub search_chars: usize,
    /// The characters of each text a title keeps.
    pub title_chars: usize,
    /// The characters of a text the preview shows.
    pub preview_chars: usize,
    /// A thumbnail's longest side, in pixels.
    pub thumb_px: u32,
    /// The preview image's longest side, in pixels.
    pub preview_px: u32,
    /// The most pixels an image may hold to be drawn.
    pub max_pixels: u64,
    /// The longest one picture may take, in ms.
    pub media_ms: u32,
    /// Seconds the worker may sit idle.
    pub worker_idle_s: u32,
    /// The worker's memory cap, in MB.
    pub worker_mb: u32,
    /// The first wait before a failed worker starts again, in ms.
    pub restart_ms: u32,
    /// The longest wait before a failed worker starts again, in ms.
    pub restart_max_ms: u32,
    /// The longest wait for the clipboard, in ms.
    pub open_ms: u32,
    /// How often the clipboard is tried within that wait, in ms.
    pub poll_ms: u32,
    /// The most copies that wait for the history to keep them.
    pub queue_copies: usize,
    /// The save's wait after a first change, in ms.
    pub save_ms: u32,
}

impl ClipSettings {
    /// The settings in the embedded `defaults.toml`.
    ///
    /// # Errors
    /// `SettingsError::Parse` when the defaults do not parse, or the error from `check`.
    pub fn from_defaults() -> Result<Self, SettingsError> {
        SPEC.defaults_as()
    }

    /// The history's ranges.
    fn check_history(&self) -> Result<(), SettingsError> {
        within("keep_days", self.keep_days, &KEEP_DAYS)?;
        within("media_days", self.media_days, &KEEP_DAYS)?;
        within("max_entries", self.max_entries, &MAX_ENTRIES)?;
        within("max_mb", self.max_mb, &MAX_MB)?;
        within("min_free_mb", self.min_free_mb, &MIN_FREE_MB)?;
        within("max_entry_mb", self.max_entry_mb, &MAX_ENTRY_MB)?;
        within("clear_follow_s", self.clear_follow_s, &CLEAR_FOLLOW_S)?;
        within("join_breaks", self.join_breaks, &JOIN_BREAKS)?;
        within("paste_gap_ms", self.paste_gap_ms, &PASTE_GAP_MS)?;
        within("paste_wait_ms", self.paste_wait_ms, &PASTE_WAIT_MS)?;
        within("check_ms", self.check_ms, &CHECK_MS)?;
        within("search_chars", self.search_chars, &SEARCH_CHARS)?;
        within("title_chars", self.title_chars, &TITLE_CHARS)?;
        within("preview_chars", self.preview_chars, &PREVIEW_CHARS)?;
        within("save_ms", self.save_ms, &SAVE_MS)
    }

    /// The media worker's, the clipboard waits' and the copy queue's ranges, their pairs' order, and the worker's room.
    fn check_platform(&self) -> Result<(), SettingsError> {
        within("thumb_px", self.thumb_px, &THUMB_PX)?;
        within("preview_px", self.preview_px, &PREVIEW_PX)?;
        within("max_pixels", self.max_pixels, &MAX_PIXELS)?;
        within("media_ms", self.media_ms, &MEDIA_MS)?;
        within("worker_idle_s", self.worker_idle_s, &WORKER_IDLE_S)?;
        within("worker_mb", self.worker_mb, &WORKER_MB)?;
        within("restart_ms", self.restart_ms, &RESTART_MS)?;
        within("restart_max_ms", self.restart_max_ms, &RESTART_MAX_MS)?;
        within("open_ms", self.open_ms, &OPEN_MS)?;
        within("poll_ms", self.poll_ms, &POLL_MS)?;
        within("queue_copies", self.queue_copies, &QUEUE_COPIES)?;
        refuse_if(self.poll_ms > self.open_ms, BAD_POLL)?;
        refuse_if(self.restart_ms > self.restart_max_ms, BAD_RESTART)?;
        refuse_if(
            self.worker_need_mb() > u64::from(self.worker_mb),
            BAD_WORKER,
        )
    }

    /// The memory one ask may need, in MB: `READ` times the largest entry, the larger of a preview's RGBA (which covers
    /// a thumbnail) and the DIB an insert makes of that entry, and `WORKER_BASE_MB`.
    fn worker_need_mb(&self) -> u64 {
        let side = u64::from(self.preview_px);
        let rgba = u64::try_from(RGBA_BYTES).unwrap_or(u64::MAX);
        let preview = side.saturating_mul(side).saturating_mul(rgba).div_ceil(MB);
        let entry = u64::from(self.max_entry_mb);
        READ * entry + preview.max(entry) + WORKER_BASE_MB
    }

    /// The skipped apps and the separator line: each a line of text within its length.
    fn check_texts(&self) -> Result<(), SettingsError> {
        let count = self.skip_apps.len();
        let apps = SKIP_APPS.contains(&count)
            && (self.skip_apps.iter())
                .all(|a| APP_CHARS.contains(&a.chars().count()) && one_line(a));
        refuse_if(!apps, BAD_APP)?;
        let rule = self.join_rule.chars().count() <= JOIN_RULE_CHARS && one_line(&self.join_rule);
        refuse_if(!rule, BAD_RULE)
    }
}

/// Shows every value but the skipped apps, whose ids tell what the user works in.
impl fmt::Debug for ClipSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClipSettings")
            .field("keep_days", &self.keep_days)
            .field("media_days", &self.media_days)
            .field("max_entries", &self.max_entries)
            .field("max_mb", &self.max_mb)
            .field("min_free_mb", &self.min_free_mb)
            .field("max_entry_mb", &self.max_entry_mb)
            .field("clear_follow_s", &self.clear_follow_s)
            .field("skip_apps", &Redacted::new(&self.skip_apps))
            .field("join_breaks", &self.join_breaks)
            .field("join_rule", &self.join_rule)
            .field("paste_gap_ms", &self.paste_gap_ms)
            .field("paste_wait_ms", &self.paste_wait_ms)
            .field("check_ms", &self.check_ms)
            .field("history_key", &self.history_key)
            .field("search_chars", &self.search_chars)
            .field("title_chars", &self.title_chars)
            .field("preview_chars", &self.preview_chars)
            .field("thumb_px", &self.thumb_px)
            .field("preview_px", &self.preview_px)
            .field("max_pixels", &self.max_pixels)
            .field("media_ms", &self.media_ms)
            .field("worker_idle_s", &self.worker_idle_s)
            .field("worker_mb", &self.worker_mb)
            .field("restart_ms", &self.restart_ms)
            .field("restart_max_ms", &self.restart_max_ms)
            .field("open_ms", &self.open_ms)
            .field("poll_ms", &self.poll_ms)
            .field("queue_copies", &self.queue_copies)
            .field("save_ms", &self.save_ms)
            .finish()
    }
}

impl Settings for ClipSettings {
    /// Each range, the pairs' order, then the lists, so the parts never get a bad config.
    fn check(&self) -> Result<(), SettingsError> {
        self.check_history()?;
        self.check_platform()?;
        self.check_texts()
    }
}

/// True when `text` holds no line break, Unicode's separators included, or other control character.
fn one_line(text: &str) -> bool {
    !text.chars().any(|c| c.is_control() || BREAKS.contains(&c))
}

/// Refuses with `why` when `bad`.
fn refuse_if(bad: bool, why: &str) -> Result<(), SettingsError> {
    if bad {
        Err(SettingsError::Invalid(why.to_owned()))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests;
