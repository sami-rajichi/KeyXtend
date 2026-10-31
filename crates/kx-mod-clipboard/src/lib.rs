//! Clipboard history (spec §7, ADR-0034): the copies kept, newest first, within the owner's limits. These parts are
//! pure: entries and their kinds, the content hash, the order, the limits and sweeps, and follow-the-clear.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod entry;
pub mod follow;
pub mod hash;
pub mod keep;
pub mod order;

#[cfg(test)]
mod fixture;
