//! Capitals (spec §6 "Matching"): guesses follow what was typed, and a word the pack spells with a capital keeps it.

use kx_module_api::ports::TypingMode;

use crate::casing::Casing;

/// How a guess is capitalised.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Case {
    /// As the pack or the learner spells it.
    Keep,
    /// Its first letter in capitals.
    First,
    /// Every letter in capitals.
    All,
}

/// The typed capitals that ask for all capitals when no lower-case letter was typed.
const ALL_FROM: usize = 2;

impl Case {
    /// The case for `prefix` typed in `mode`, by the Unicode case of each typed letter: with no lower-case letter,
    /// Caps Lock or two capitals give all capitals; a typed capital first, or Shift before any letter, gives a first
    /// capital.
    pub(super) fn of(prefix: &str, mode: TypingMode) -> Self {
        let capitals = prefix.chars().filter(|c| c.is_uppercase()).count();
        let lower = prefix.chars().any(char::is_lowercase);
        let first = prefix.chars().next().is_some_and(char::is_uppercase);
        if !lower && (mode.caps || capitals >= ALL_FROM) {
            Self::All
        } else if first || (prefix.is_empty() && mode.shift) {
            Self::First
        } else {
            Self::Keep
        }
    }

    /// `word` in this case, as `casing` writes capitals; capitals already in it stay.
    pub(super) fn apply(self, word: &str, casing: &Casing) -> String {
        match self {
            Self::Keep => word.to_owned(),
            Self::All => casing.upper(word),
            Self::First => casing.capital(word),
        }
    }
}
