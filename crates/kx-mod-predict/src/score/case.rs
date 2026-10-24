//! Capitals (spec §6 "Matching"): guesses follow what was typed, and a word the pack spells with a capital keeps it.

use kx_module_api::ports::TypingMode;

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

impl Case {
    /// The case for `prefix` typed in `mode`: Caps Lock or two typed capitals give all capitals; a typed capital
    /// first, or Shift before any letter, gives a first capital.
    pub(super) fn of(prefix: &str, mode: TypingMode) -> Self {
        let capitals = prefix.chars().filter(|c| c.is_uppercase()).count();
        let first = prefix.chars().next().is_some_and(char::is_uppercase);
        if mode.caps || capitals > 1 {
            Self::All
        } else if first || (prefix.is_empty() && mode.shift) {
            Self::First
        } else {
            Self::Keep
        }
    }

    /// `word` in this case; capitals already in it stay.
    pub(super) fn apply(self, word: &str) -> String {
        match self {
            Self::Keep => word.to_owned(),
            Self::All => word.to_uppercase(),
            Self::First => {
                let mut chars = word.chars();
                let first = chars.next().into_iter().flat_map(char::to_uppercase);
                first.chain(chars).collect()
            }
        }
    }
}
