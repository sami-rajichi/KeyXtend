//! `sounds.toml`: the format of every shipped sound file, how the speaker plays, and each event's files and balance.

pub mod wav;

use kx_platform::ui::Clip;
use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::icons::plain;
use crate::{SOUNDS, SOUNDS_FILE, ThemeError};

/// The type of every sound file.
pub const SOUND_EXT: &str = "wav";
/// The table that holds the format.
const FORMAT: &str = "format";
/// The table that holds how the speaker plays.
const PLAY: &str = "play";
/// Bits in a byte.
const BYTE_BITS: u16 = 8;
/// Milliseconds per second.
const MS_PER_S: f64 = 1000.0;

/// A moment the app plays a sound for.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Event {
    /// A key press.
    Key,
    /// A hold fires the right-click.
    Rclick,
    /// Grab picks a window up.
    Pick,
    /// Grab drops it.
    Drop,
}

impl Event {
    /// Every event, in the order the app lists them.
    pub const ALL: [Self; 4] = [Self::Key, Self::Rclick, Self::Pick, Self::Drop];

    /// Its table in `sounds.toml`.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Key => "key",
            Self::Rclick => "rclick",
            Self::Pick => "pick",
            Self::Drop => "drop",
        }
    }
}

/// The format every shipped sound file is in.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Format {
    /// Samples per second.
    pub rate: u32,
    /// Bits per sample.
    pub bits: u16,
    /// Channels.
    pub channels: u16,
    /// The highest peak, in dBFS.
    pub ceiling_db: f64,
    /// The fade at the end of every sound, in ms.
    pub fade_ms: f64,
    /// Import trims audio quieter than this, in dB below the peak.
    pub trim_db: f64,
    /// The longest sound, in ms.
    pub max_ms: f64,
}

impl Format {
    /// Bytes in one frame: one sample of each channel.
    #[must_use]
    pub fn frame_bytes(&self) -> usize {
        usize::from(self.channels) * usize::from(self.bits / BYTE_BITS)
    }

    /// The most frames a sound may hold: `max_ms` at `rate`, rounded.
    #[must_use]
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "Float-to-integer casts saturate, and NaN gives 0."
    )]
    pub fn max_frames(&self) -> usize {
        (self.max_ms * f64::from(self.rate) / MS_PER_S).round() as usize
    }

    /// Refuses a layout the speaker cannot play and values out of range; NaN fails each.
    fn check(&self) -> Result<(), String> {
        let (channels, bits) = (Clip::CHANNELS, Clip::SAMPLE_BYTES * BYTE_BITS);
        check(self.rate > 0, "rate must be above 0")?;
        let pcm16 = format!("bits must be {bits}, as the speaker plays");
        check(self.bits == bits, pcm16)?;
        let mono = format!("channels must be {channels}, as the speaker plays");
        check(self.channels == channels, mono)?;
        let ceiling = self.ceiling_db.is_finite() && self.ceiling_db <= 0.0;
        check(ceiling, "ceiling_db must be 0 or less")?;
        let trim = self.trim_db.is_finite() && self.trim_db < 0.0;
        check(trim, "trim_db must be below 0")?;
        let max = self.max_ms.is_finite() && self.max_ms > 0.0;
        check(max, "max_ms must be above 0")?;
        let fade = self.fade_ms >= 0.0 && self.fade_ms <= self.max_ms;
        check(fade, "fade_ms must be from 0 to max_ms")
    }
}

/// How the speaker plays.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Play {
    /// Voices per clip, so quick plays of one clip overlap.
    pub voices: usize,
}

/// What one event plays: its files, in turn, and its balance.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cue {
    /// File names in `assets/sounds/`, played in turn.
    pub files: Vec<String>,
    /// The event's balance, in dB (0 or less).
    pub gain_db: f64,
}

impl Cue {
    /// Refuses a cue with no file, a name that is not a plain `.wav` name, or a gain above 0.
    fn check(&self) -> Result<(), String> {
        check(!self.files.is_empty(), "files must name a sound")?;
        if let Some(f) = self.files.iter().find(|f| !sound_name(f)) {
            return Err(format!(
                "{f:?} is not lower-case letters, digits and - then .{SOUND_EXT}"
            ));
        }
        check(
            self.gain_db.is_finite() && self.gain_db <= 0.0,
            "gain_db must be 0 or less",
        )
    }
}

/// The checked `sounds.toml`.
#[derive(Debug, Clone, PartialEq)]
pub struct Sounds {
    /// The format every sound file is in.
    pub format: Format,
    /// How the speaker plays.
    pub play: Play,
    /// Each event's cue, in `Event::ALL` order.
    pub cues: Vec<(Event, Cue)>,
}

impl Sounds {
    /// The shipped tokens, checked.
    ///
    /// # Errors
    ///
    /// `sounds.toml` does not parse or breaks a rule, named with the reason.
    pub fn shipped() -> Result<Self, ThemeError> {
        Self::parse(SOUNDS).map_err(|why| ThemeError {
            file: SOUNDS_FILE,
            why,
        })
    }

    /// The checked sound tokens in `text`; the sound tool reads the shipped file through it too.
    ///
    /// # Errors
    ///
    /// An unknown or missing key or table, or a value out of range, named.
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut table: toml::Table = toml::from_str(text).map_err(|e| e.to_string())?;
        let format: Format = take(&mut table, FORMAT)?;
        format.check().map_err(|why| format!("[{FORMAT}] {why}"))?;
        let play: Play = take(&mut table, PLAY)?;
        check(
            play.voices > 0,
            format!("[{PLAY}] voices must be 1 or more"),
        )?;
        let mut cues = Vec::with_capacity(Event::ALL.len());
        for event in Event::ALL {
            let cue: Cue = take(&mut table, event.name())?;
            cue.check()
                .map_err(|why| format!("[{}] {why}", event.name()))?;
            cues.push((event, cue));
        }
        match table.keys().next() {
            Some(key) => Err(format!("[{key}] is not a table sounds.toml holds")),
            None => Ok(Self { format, play, cues }),
        }
    }

    /// Each file the cues name, once, in the order first named.
    #[must_use]
    pub fn files(&self) -> Vec<&str> {
        let mut files: Vec<&str> = Vec::new();
        for file in self.cues.iter().flat_map(|(_, c)| &c.files) {
            if !files.contains(&file.as_str()) {
                files.push(file);
            }
        }
        files
    }
}

/// `Err(why)` unless `ok`.
fn check(ok: bool, why: impl Into<String>) -> Result<(), String> {
    if ok { Ok(()) } else { Err(why.into()) }
}

/// True for a plain sound file name: lower-case letters, digits and `-`, then `.wav`.
fn sound_name(name: &str) -> bool {
    let stem = name
        .strip_suffix(SOUND_EXT)
        .and_then(|s| s.strip_suffix('.'));
    stem.is_some_and(plain)
}

/// Removes table `key` from `table` and reads it as a `T`.
fn take<T: DeserializeOwned>(table: &mut toml::Table, key: &str) -> Result<T, String> {
    let value = table
        .remove(key)
        .ok_or_else(|| format!("[{key}] is missing"))?;
    value.try_into().map_err(|e| format!("[{key}] {e}"))
}
