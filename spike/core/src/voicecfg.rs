//! Voice settings (`spike.toml [voice]`): the worker, the local server, the opt-in cloud engine and the caption bar.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Bits of a LANGID that name the language without its region.
const PRIMARY_MASK: u16 = 0x03FF;

/// Where speech turns into text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Engine {
    /// whisper.cpp's server on this PC.
    Local,
    /// The opt-in cloud API.
    Cloud,
}

/// Voice settings.
#[derive(Debug, Clone, Deserialize)]
pub struct VoiceConfig {
    /// The worker program, next to the face.
    pub worker: String,
    /// Switch that runs the local server in weak mode.
    pub weak_arg: String,
    /// Switch that allows the cloud engine; without it the worker refuses cloud.
    pub cloud_arg: String,
    /// Engine the face uses.
    pub engine: Engine,
    /// Whether the face starts the worker with the cloud switch.
    pub cloud_on: bool,
    /// Longest recording, in seconds.
    pub record_max_s: u64,
    /// Sample rate the engines take, in Hz.
    pub rate_hz: u32,
    /// Primary language of the keyboard layout (low 10 bits of its LANGID, 2 hex digits) to spoken language.
    pub languages: BTreeMap<String, String>,
    /// Language for a layout not in the table.
    pub default_lang: String,
    /// Typed after each transcript, so sentences do not run together.
    pub type_suffix: String,
    /// Gap between typed characters, in ms; Notepad drops most of a sentence typed in one batch.
    pub type_gap_ms: u64,
    /// Switch, followed by a folder, that makes the worker keep each clip there for the bench.
    pub keep_arg: String,
    /// Longest JSON line between face and worker, in bytes; longer lines are skipped.
    pub line_max_bytes: usize,
    /// The local server.
    pub local: LocalConfig,
    /// The cloud engine.
    pub cloud: CloudConfig,
    /// Limits of weak mode.
    pub weak: WeakConfig,
    /// The caption bar.
    pub caption: CaptionConfig,
}

/// whisper.cpp's server, started once with the worker.
#[derive(Debug, Clone, Deserialize)]
pub struct LocalConfig {
    /// Server program.
    pub server: String,
    /// Model file.
    pub model: String,
    /// Address it listens on; this PC only.
    pub host: String,
    /// Port it listens on.
    pub port: u16,
    /// Threads it computes with.
    pub threads: u32,
    /// Request path for a clip.
    pub path: String,
    /// Path that answers 200 once the model is loaded.
    pub health_path: String,
    /// Longest wait for it to answer after start, in ms.
    pub ready_wait_ms: u64,
    /// Pause between readiness checks, in ms.
    pub ready_poll_ms: u64,
    /// Longest wait for one answer, in ms.
    pub timeout_ms: u64,
}

/// The opt-in cloud engine (OpenAI-style transcription API).
#[derive(Debug, Clone, Deserialize)]
pub struct CloudConfig {
    /// Host, reached over HTTPS.
    pub host: String,
    /// Request path.
    pub path: String,
    /// Model name.
    pub model: String,
    /// Environment variable that holds the API key.
    pub key_env: String,
    /// Longest wait for one answer, in ms.
    pub timeout_ms: u64,
}

/// Weak mode: the local server runs in a job with these caps.
#[derive(Debug, Clone, Deserialize)]
pub struct WeakConfig {
    /// Cores it may use.
    pub cores: u32,
    /// CPU share, in percent of the whole machine.
    pub rate_pct: u32,
    /// Memory cap, in MiB.
    pub mem_mb: u64,
}

/// The caption bar at the bottom centre of the screen.
#[derive(Debug, Clone, Deserialize)]
pub struct CaptionConfig {
    /// Window title, so the harness can find it.
    pub title: String,
    /// Width, height and gap from the bottom of the work area, in logical pixels.
    pub px: [f32; 3],
    /// Shown while recording.
    pub listening: String,
    /// Shown while the engine works.
    pub transcribing: String,
    /// Shown when the engine heard no words.
    pub nothing_heard: String,
    /// How long the text stays after it arrives, in ms.
    pub hide_ms: u64,
}

impl VoiceConfig {
    /// The spoken language for keyboard layout language `lang_id`; regional layouts share their language.
    pub fn language(&self, lang_id: u16) -> &str {
        self.languages
            .get(&format!("{:02X}", lang_id & PRIMARY_MASK))
            .unwrap_or(&self.default_lang)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spike_toml_loads_the_voice_settings() {
        let v = crate::config::load().expect("spike.toml loads").voice;
        assert_eq!(
            v.engine,
            Engine::Local,
            "the face starts on the local engine"
        );
        assert!(!v.cloud_on, "cloud is off unless turned on");
        assert!(v.record_max_s > 0 && v.rate_hz > 0);
    }

    #[test]
    fn the_language_follows_the_keyboard_layout() {
        let v = crate::config::load().expect("spike.toml loads").voice;
        assert_eq!(v.language(0x0409), "en");
        assert_eq!(v.language(0x040C), "fr");
        assert_eq!(v.language(0x1C01), "ar");
        assert_eq!(v.language(0x0407), v.default_lang);
    }

    #[test]
    fn regional_layouts_share_their_language() {
        let v = crate::config::load().expect("spike.toml loads").voice;
        for (id, lang) in [
            (0x0401, "ar"),
            (0x3801, "ar"),
            (0x0809, "en"),
            (0x0C0C, "fr"),
        ] {
            assert_eq!(v.language(id), lang, "{id:04X}");
        }
    }
}
