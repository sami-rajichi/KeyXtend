//! Clipboard history (spec §7, ADR-0034): the copies kept, newest first, within the owner's limits. These parts run no
//! thread and reach the disk only through the shelf port: entries and their kinds, the content hash, the order, the
//! limits and sweeps, follow-the-clear, and the store.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod entry;
pub mod find;
pub mod follow;
pub mod hash;
pub mod keep;
pub mod order;
pub mod panel;
pub mod store;
pub mod view;

#[cfg(test)]
mod fixture;
