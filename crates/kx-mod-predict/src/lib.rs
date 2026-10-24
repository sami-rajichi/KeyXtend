//! Word prediction (spec §6, ADR-0032): folding for loose matching, the word-pack format each language's seed ships
//! in, and the word splitting the pack builder and the tracker share. The module itself comes later in P7.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod fold;
pub mod pack;
pub mod script;
pub mod split;

pub use fold::fold;
pub use pack::{Flags, Pack, PackFormatError, WordId};
#[cfg(feature = "build")]
pub use pack::{PackWriter, WriteError};
