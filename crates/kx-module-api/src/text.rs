//! Text folding and casing by language (ADR-0034 decision 6): word prediction and clipboard search fold with one
//! table, so `ecole` finds `école` and `ان` finds `أن` in both.

mod casing;
mod fold;

pub use casing::Casing;
pub use fold::{apostrophe, fold, is_apostrophe};
