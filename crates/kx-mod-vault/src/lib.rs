//! Quick-fill (design "Part 2", ADR-0038): `VaultModule` reads `[vault]` (`settings`) and keeps the notebook, sealed
//! with the account's key. Its pure parts: the `kinds` and labels tables, the `entry` rules, the `book` within its
//! limits, its `store`, search (`find`), the list's `order` and By label, and `make` for Make one.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod actor;
pub mod backup;
pub mod book;
pub mod entry;
pub mod fill;
pub mod find;
pub mod import;
pub mod kinds;
pub mod make;
mod manifest;
mod module;
pub mod order;
pub mod panel;
pub mod paste;
pub mod settings;
pub mod store;
pub mod view;

#[cfg(test)]
mod fixture;

pub use manifest::{ID, MANIFEST};
pub use module::VaultModule;
pub use settings::{SPEC, VaultSettings};
