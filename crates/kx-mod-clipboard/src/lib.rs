//! Clipboard history (spec §7, ADR-0034): `ClipboardModule` reads `[clipboard]` (`settings`), keeps the copies newest
//! first within the owner's limits, sealed on the shelf in day folders, and runs the panel and Insert through the
//! keyboard. Its pure parts run no thread: entries (`entry`), the `hash`, the `order`, the limits (`keep`),
//! follow-the-clear (`follow`), the `store`, search (`find`), the `panel`, its `view` and the `insert` plan; the
//! `kx-clip` actor runs them, with the `media` threads drawing pictures and checking files.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod actor;
pub mod entry;
pub mod find;
pub mod follow;
pub mod hash;
pub mod insert;
pub mod keep;
mod manifest;
pub mod media;
mod module;
pub mod order;
pub mod panel;
pub mod settings;
pub mod store;
pub mod view;

#[cfg(test)]
mod fixture;

pub use manifest::{ID, MANIFEST};
pub use module::ClipboardModule;
pub use settings::{ClipSettings, SPEC};
