//! Word prediction (spec §6, ADR-0032): folding and casing, the word splitting the pack builder shares, the
//! word-pack format, and the engine: the word tracker, the scorer, the learner and email suggestions. The module comes
//! later in P7.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod casing;
pub mod email;
pub mod fold;
pub mod learn;
pub mod pack;
pub mod score;
pub mod script;
pub mod split;
pub mod track;

pub use casing::Casing;
pub use fold::fold;
pub use pack::{Flags, Pack, PackFormatError, WordId};
#[cfg(feature = "build")]
pub use pack::{PackWriter, WriteError};
