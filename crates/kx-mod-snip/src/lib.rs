//! Snip and screen recording (P12, ADR-0055, ADR-0056): pictures and short videos of a part, a window or the screen.
//! This crate holds the module's manifest, its `[snip]` settings, the capture flow, the picking rules, where the strip
//! goes, the picture editor's model, and the actor that carries the flow out on the ports.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod actor;
pub mod edit;
pub mod flow;
pub mod frame;
mod manifest;
mod module;
pub mod part;
mod settings;
pub mod strip;

pub use manifest::{ID, MANIFEST};
pub use module::SnipModule;
pub use settings::{
    ASIDE_MS, COPIED_DAYS, COPY_AGAIN, COPY_AGAIN_MS, COPY_MS, COPY_POLL_MS, COUNTDOWN_S,
    COUNTDOWN_STEP_S, ENCODE_IDLE_S, ENCODE_MB_MS, ENCODE_MS, ENCODE_RESTART_MS, EncodeWaits,
    FRAME_MIN_PX, MAX_MIN_STEP, NOTE_S, SHOWN_MS, SPEC, STROKE_MAX, SnipSettings, UNDO_MAX,
};
