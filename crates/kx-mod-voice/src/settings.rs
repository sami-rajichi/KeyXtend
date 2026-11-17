//! The `[voice]` settings (plan "Settings `[voice]`"): the switches, the folder and picks, the device, the listening
//! and typing numbers, the bar, history and recordings, the card's check, the worker's limits and the cloud services.
//! The worker's ranges are the ports' own, so a value is never checked twice; the rest are named consts here.

mod cloud;
mod drop;
mod names;
mod parts;

pub(crate) use cloud::datetime;
pub use cloud::{FACTS_DAYS, KEEP_START, ServiceOn};
pub use names::{INSERTS, Insert, PLACES, insert_named, place_name, place_named};

use std::collections::BTreeMap;
use std::ops::RangeInclusive;
use std::path::Path;

use kx_module_api::ports::{CardUse, DevicePick, FetchAsk, JobLimits, SpeechAsk, path_ok};
use kx_module_api::{
    Settings, SettingsError, SettingsSpec, is_hidden, refuse_if, validate_as, within,
};
use serde::{Deserialize, Serialize};

/// Seconds without speech before the bar hints that nothing is heard.
pub const NOTHING_S: RangeInclusive<u8> = 2..=60;
/// The wait within which a second click on the key is ignored, in ms.
pub const DEBOUNCE_MS: RangeInclusive<u16> = 100..=2_000;
/// The most characters of what follows the typed text.
pub const TYPE_SUFFIX_CHARS: usize = 8;
/// The most characters of what goes between History's texts.
pub const HISTORY_JOIN_CHARS: usize = 8;
/// The wait between typed pieces, in ms.
pub const TYPE_GAP_MS: RangeInclusive<u16> = 0..=200;
/// How long the bar shows the text, in ms.
pub const DONE_MS: RangeInclusive<u32> = 2_000..=60_000;
/// The wave's smoothing, in ms.
pub const WAVE_MS: RangeInclusive<u16> = 0..=2_000;
/// The most texts the history keeps.
pub const HISTORY_MAX: RangeInclusive<u16> = 1..=500;
/// The most the kept recordings take, in MB.
pub const VOICE_MAX_MB: RangeInclusive<u32> = 64..=65_536;
/// How many times faster than the processor a card must be.
pub const CARD_GAIN: RangeInclusive<f64> = 1.0..=10.0;
/// The most words a card's text may differ by.
pub const CARD_DIFF: RangeInclusive<u8> = 0..=5;
/// The longest a look at the focused box may take, in ms; the module reads its kind alone.
pub const READ_MS: RangeInclusive<u32> = 50..=1_000;
/// The longest Copy waits for another app to let go of the clipboard, in ms.
pub const COPY_MS: RangeInclusive<u32> = 10..=2_000;
/// How often Copy tries again within that wait, in ms.
pub const COPY_POLL_MS: RangeInclusive<u32> = 1..=100;
/// The longest wait for the clipboard history to paste a text, in ms.
pub const PASTE_MS: RangeInclusive<u32> = 500..=10_000;

/// Why a value or a pair was refused.
const BAD_RESTART: &str = "restart_ms must not pass restart_max_ms";
const BAD_POLL: &str = "copy_poll_ms must not pass copy_ms";
const BAD_DEVICE: &str = "device must be auto, cpu or a card's id";
const BAD_PLACE: &str = "place must be above, bottom or top";
const BAD_INSERT: &str = "insert must be type or paste";
const BAD_DIR: &str = "models_dir must be empty or an absolute drive path";
const BAD_SUFFIX: &str =
    "type_suffix must stay within its length and hold only characters that show";
const BAD_JOIN: &str =
    "history_join must stay within its length and hold only characters that show";

/// The picks' key in the section, which also names a dropped pick in the settings notice.
pub(crate) const PICKS: &str = "picks";
/// The switched-on services' key in the section, which also names a dropped entry in the settings notice.
pub(crate) const SERVICES: &str = "services";
/// The device's key in the section, which the Device page's Use writes.
pub(crate) const DEVICE: &str = "device";
/// The models folder's key in the section, which the first download's folder question writes.
pub(crate) const MODELS_DIR: &str = "models_dir";

/// How the kernel reads the `[voice]` section: version 1, with the defaults embedded.
pub static SPEC: SettingsSpec = SettingsSpec {
    version: 1,
    defaults: include_str!("../defaults.toml"),
    migrations: &[],
    validate: validate_as::<VoiceSettings>,
};

/// The module's settings; every value comes from `defaults.toml` or the user's file.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "Each flag is one on-or-off setting of the flat [voice] section the user reads and writes."
)]
pub struct VoiceSettings {
    /// The whole feature.
    pub on: bool,
    /// The models folder; empty until the first download asks for it.
    pub models_dir: String,
    /// The user's model per language, by row id or file name, or a service's id and model joined by `PICK_MARK`.
    pub picks: BTreeMap<String, String>,
    /// Cloud services may be switched on; off, none is offered and every language uses its local model.
    pub cloud: bool,
    /// The services the user switched on, by row id.
    pub services: BTreeMap<String, ServiceOn>,
    /// The longest wait for one cloud request's answer, in seconds.
    pub net_wait_s: u16,
    /// Tries per piece before a cloud note.
    pub net_tries: u8,
    /// Days after which a card's facts show as old.
    pub facts_days: u16,
    /// Where models run: `auto`, `cpu` or a card's id.
    pub device: String,
    /// Processor threads; 0 means the real cores.
    pub threads: u8,
    /// Minutes a model stays after the last dictation.
    pub idle_min: u16,
    /// The longest dictation, in seconds.
    pub record_max_s: u16,
    /// A piece longer than this, in seconds, is cut at the next pause.
    pub piece_s: u8,
    /// The silence kept around speech, in ms.
    pub pad_ms: u16,
    /// Seconds before the nothing-heard hint.
    pub nothing_s: u8,
    /// Silence after speech stops listening.
    pub silence_stop: bool,
    /// The silence that stops listening, in seconds.
    pub silence_stop_s: u8,
    /// A second click this soon, in ms, is ignored.
    pub debounce_ms: u16,
    /// What follows the typed text.
    pub type_suffix: String,
    /// What goes between History's texts when Copy or Insert takes more than one.
    pub history_join: String,
    /// The wait between typed pieces, in ms.
    pub type_gap_ms: u16,
    /// How a text goes in, by its name in `INSERTS`.
    pub insert: String,
    /// The longest wait for the clipboard history to paste a text, in ms; then it is typed.
    pub paste_ms: u32,
    /// How long the bar shows the text, in ms.
    pub done_ms: u32,
    /// The wave's smoothing, in ms.
    pub wave_ms: u16,
    /// Levels told per second.
    pub level_hz: u8,
    /// Where the bar sits, by its name in `PLACES`.
    pub place: String,
    /// The start and stop sounds.
    pub sounds: bool,
    /// The history of texts.
    pub history: bool,
    /// The most texts the history keeps.
    pub history_max: u16,
    /// Each recording is kept with its text.
    pub keep_voice: bool,
    /// The most the kept recordings take, in MB.
    pub voice_max_mb: u32,
    /// The room a card keeps free beside a model, in MB.
    pub card_margin_mb: u32,
    /// How many times faster a card must be.
    pub card_gain: f64,
    /// The most words a card's text may differ by.
    pub card_diff: u8,
    /// The room the models drive keeps free, in MB.
    pub min_free_mb: u32,
    /// A job's memory cap without a model, in MB.
    pub job_mb: u32,
    /// The longest wait for the worker's answer, in ms.
    pub ask_ms: u32,
    /// The longest load, read or start of a fresh job, in seconds.
    pub load_s: u16,
    /// The longest warm-up, in seconds.
    pub warm_s: u16,
    /// The longest one piece may take, in seconds.
    pub piece_wait_s: u16,
    /// A download with no byte this long fails, in seconds.
    pub stall_s: u16,
    /// The first wait before a failed worker starts again, in ms.
    pub restart_ms: u32,
    /// The longest wait before a failed worker starts again, in ms.
    pub restart_max_ms: u32,
    /// The longest a look at the focused box may take, in ms.
    pub read_ms: u32,
    /// The longest Copy waits for the clipboard, in ms.
    pub copy_ms: u32,
    /// How often Copy tries again within that wait, in ms.
    pub copy_poll_ms: u32,
}

impl VoiceSettings {
    /// The settings in the embedded `defaults.toml`.
    ///
    /// # Errors
    /// `SettingsError::Parse` when the defaults do not parse, or the error from `check`.
    pub fn from_defaults() -> Result<Self, SettingsError> {
        SPEC.defaults_as()
    }

    /// The listening ranges, which are the speech port's own.
    fn check_listening(&self) -> Result<(), SettingsError> {
        within("threads", self.threads, &SpeechAsk::THREADS)?;
        within("record_max_s", self.record_max_s, &SpeechAsk::RECORD_MAX_S)?;
        within("piece_s", self.piece_s, &SpeechAsk::PIECE_S)?;
        within("pad_ms", self.pad_ms, &SpeechAsk::PAD_MS)?;
        within(
            "silence_stop_s",
            self.silence_stop_s,
            &SpeechAsk::SILENCE_STOP_S,
        )?;
        within("level_hz", self.level_hz, &SpeechAsk::LEVEL_HZ)?;
        within("card_margin_mb", self.card_margin_mb, &CardUse::MARGIN_MB)?;
        within("min_free_mb", self.min_free_mb, &FetchAsk::MIN_FREE_MB)?;
        within("stall_s", self.stall_s, &FetchAsk::STALL_S)
    }

    /// The worker's limits, which are the job's own, and their order.
    fn check_job(&self) -> Result<(), SettingsError> {
        within("job_mb", self.job_mb, &JobLimits::JOB_MB)?;
        within("idle_min", self.idle_min, &JobLimits::IDLE_MIN)?;
        within("ask_ms", self.ask_ms, &JobLimits::ASK_MS)?;
        within("load_s", self.load_s, &JobLimits::LOAD_S)?;
        within("warm_s", self.warm_s, &JobLimits::WARM_S)?;
        within("piece_wait_s", self.piece_wait_s, &JobLimits::PIECE_WAIT_S)?;
        within("restart_ms", self.restart_ms, &JobLimits::RESTART_MS)?;
        within(
            "restart_max_ms",
            self.restart_max_ms,
            &JobLimits::RESTART_MAX_MS,
        )?;
        refuse_if(self.restart_ms > self.restart_max_ms, BAD_RESTART)
    }

    /// The module's own ranges.
    fn check_own(&self) -> Result<(), SettingsError> {
        within("nothing_s", self.nothing_s, &NOTHING_S)?;
        within("debounce_ms", self.debounce_ms, &DEBOUNCE_MS)?;
        within("type_gap_ms", self.type_gap_ms, &TYPE_GAP_MS)?;
        within("done_ms", self.done_ms, &DONE_MS)?;
        within("wave_ms", self.wave_ms, &WAVE_MS)?;
        within("history_max", self.history_max, &HISTORY_MAX)?;
        within("voice_max_mb", self.voice_max_mb, &VOICE_MAX_MB)?;
        within("card_gain", self.card_gain, &CARD_GAIN)?;
        within("card_diff", self.card_diff, &CARD_DIFF)?;
        within("read_ms", self.read_ms, &READ_MS)?;
        within("copy_ms", self.copy_ms, &COPY_MS)?;
        within("copy_poll_ms", self.copy_poll_ms, &COPY_POLL_MS)?;
        within("paste_ms", self.paste_ms, &PASTE_MS)?;
        refuse_if(self.copy_poll_ms > self.copy_ms, BAD_POLL)
    }

    /// The named values: the device, the place, the way a text goes in, the folder, the suffix and the join.
    fn check_names(&self) -> Result<(), SettingsError> {
        refuse_if(DevicePick::parse(&self.device).is_none(), BAD_DEVICE)?;
        refuse_if(place_named(&self.place).is_none(), BAD_PLACE)?;
        refuse_if(insert_named(&self.insert).is_none(), BAD_INSERT)?;
        let dir = &self.models_dir;
        refuse_if(!dir.is_empty() && !path_ok(Path::new(dir)), BAD_DIR)?;
        refuse_if(!shows(&self.type_suffix, TYPE_SUFFIX_CHARS), BAD_SUFFIX)?;
        refuse_if(!shows(&self.history_join, HISTORY_JOIN_CHARS), BAD_JOIN)
    }
}

impl Settings for VoiceSettings {
    /// Each range, the restart pair's order, then the named values; a bad pick or service entry is left for
    /// `drop_bad`, so the kernel never resets the other languages' picks or services with it.
    fn check(&self) -> Result<(), SettingsError> {
        self.check_listening()?;
        self.check_job()?;
        self.check_own()?;
        self.check_cloud()?;
        self.check_names()
    }
}

/// True when `text` is at most `chars` characters, each one that shows.
fn shows(text: &str, chars: usize) -> bool {
    text.chars().count() <= chars && !text.chars().any(is_hidden)
}

#[cfg(test)]
mod tests;
