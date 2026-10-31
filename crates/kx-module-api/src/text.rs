//! Text folding and casing by language (ADR-0034 decision 6): word prediction and clipboard search fold with one
//! table, so `ecole` finds `école` and `ان` finds `أن` in both.

mod casing;
mod fold;

pub use casing::Casing;
pub use fold::{apostrophe, fold, is_apostrophe};

/// `text` folded for search in any language: Unicode's default lower case, then the shared table, so accents,
/// harakat, tatweel and the alef forms go and ى reads as ي.
#[must_use]
pub fn fold_any(text: &str) -> String {
    fold(text, Casing::root())
}

#[cfg(test)]
mod tests;
