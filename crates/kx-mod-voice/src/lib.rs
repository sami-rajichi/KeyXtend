//! The voice typing module (P11, ADR-0042 to ADR-0044, ADR-0047): the compiled-in models and services tables, the cloud
//! rules and sealed keys, Automatic's device rule, the scorer Compare and a card's check count words with, the
//! dictation flow, History with its recordings, the counts, the sealed store, the `[voice]` settings, and the actor.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod actor;
pub mod cloud;
pub mod compare;
pub mod counts;
pub mod device;
pub mod fetch;
pub mod flow;
pub mod folder;
pub mod history;
pub mod keys;
mod manifest;
pub mod models;
mod module;
pub mod panel;
pub mod score;
pub mod services;
mod settings;
pub mod store;
pub mod view;

pub use manifest::{ID, MANIFEST};
pub use module::VoiceModule;
pub use settings::{
    CARD_DIFF, CARD_GAIN, DEBOUNCE_MS, DONE_MS, FACTS_DAYS, HISTORY_JOIN_CHARS, HISTORY_MAX,
    INSERTS, Insert, KEEP_START, NOTHING_S, PASTE_MS, PLACES, SPEC, ServiceOn, TAKE_MS,
    TYPE_GAP_MS, TYPE_SUFFIX_CHARS, VOICE_MAX_MB, VoiceSettings, WAVE_MS, place_name,
};
