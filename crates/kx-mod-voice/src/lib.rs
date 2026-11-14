//! The voice typing module (P11, ADR-0042 to ADR-0044): the compiled-in models table and its orders per language,
//! Automatic's device rule, the scorer Compare and a card's check count words with, the dictation flow, History with
//! its recordings, the counts, the sealed store, and the `[voice]` settings.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod compare;
pub mod counts;
pub mod device;
pub mod fetch;
pub mod flow;
pub mod folder;
pub mod history;
mod manifest;
pub mod models;
mod module;
pub mod panel;
pub mod score;
mod settings;
pub mod store;
pub mod view;

pub use manifest::{ID, MANIFEST};
pub use module::VoiceModule;
pub use settings::{
    CARD_DIFF, CARD_GAIN, DEBOUNCE_MS, DONE_MS, HISTORY_MAX, NOTHING_S, PLACES, SPEC, TYPE_GAP_MS,
    TYPE_SUFFIX_CHARS, VOICE_MAX_MB, VoiceSettings, WAVE_MS, place_name,
};
