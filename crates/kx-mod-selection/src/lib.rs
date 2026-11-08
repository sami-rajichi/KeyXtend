//! The selection module's rules (P10, ADR-0041): which keycaps show by a selection or a caret, when the strip shows,
//! moves and hides, and the Translate link. They are pure: no bus, no OS, and time comes in as `Mono` values.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod chips;
mod link;
mod manifest;
mod settings;
mod strip;

pub use chips::{At, Rules, chips};
pub use link::{Site, SiteError, Sites, TAG_CHARS, sites, tag_ok};
pub use manifest::{ID, MANIFEST};
pub use settings::{
    CARET_S, CONSENT_S, LOOKS_PER_S, Look, READ_MS, SETTLE_MS, SPEC, SelSettings, TRANSLATE_CHARS,
};
pub use strip::{Board, Input, Pick, Strip, StripConfig};
