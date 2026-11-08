//! The selection chips module (P10, ADR-0041): Copy, Cut, Paste and Translate keycaps by the focused box's selection
//! or caret. Its rules are pure; its actor joins them to the watch, the keyboard's edits and the browser on the bus.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod actor;
mod chips;
mod link;
mod manifest;
mod module;
mod settings;
mod strip;

pub use chips::{At, Rules, chips};
pub use link::{Site, SiteError, Sites, TAG_CHARS, sites, tag_ok};
pub use manifest::{ID, MANIFEST};
pub use module::SelectionModule;
pub use settings::{
    CARET_S, CONSENT_S, LOOKS_PER_S, READ_MS, SETTLE_MS, SPEC, SelSettings, TRANSLATE_CHARS,
    TRANSLATE_OK_KEY,
};
pub use strip::{Board, Input, Pick, Strip, StripConfig};
