//! The token rules: which wordfreq tokens may become words.

use kx_mod_predict::pack::layout::fits_spelling;
use kx_mod_predict::script;

/// True when `word` may enter a word list: it fits a pack spelling, holds a letter, no digit and none of
/// `drop_chars` (web addresses, emails), and its letters are of one script.
pub(crate) fn keeps(word: &str, drop_chars: &[char]) -> bool {
    fits_spelling(word.len())
        && word.chars().any(char::is_alphabetic)
        && !word
            .chars()
            .any(|c| c.is_numeric() || drop_chars.contains(&c))
        && script::single(word)
}
