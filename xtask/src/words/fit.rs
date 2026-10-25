//! The words budget: the rarest words go until the pack without its chains fits, measured by writing it.

use kx_mod_predict::{Flags, PackWriter, WriteError};

use super::WordsError;
use super::list::Word;

/// How many of `words`, most frequent first, fit `budget` bytes as `iso`'s pack without chains.
pub(crate) fn fitting(iso: &str, words: &[Word], budget: usize) -> Result<usize, WordsError> {
    let fits =
        |n: usize| Ok::<_, WordsError>(size(iso, words.get(..n).unwrap_or(words))? <= budget);
    if fits(words.len())? {
        return Ok(words.len());
    }
    let (mut fit, mut over) = (0, words.len());
    while over - fit > 1 {
        let mid = fit + (over - fit) / 2;
        if fits(mid)? {
            fit = mid;
        } else {
            over = mid;
        }
    }
    Ok(fit)
}

/// The bytes of `iso`'s pack holding only `words`.
pub(crate) fn size(iso: &str, words: &[Word]) -> Result<usize, WordsError> {
    let refused = |e: WriteError| WordsError::Pack(format!("{iso}: {e}"));
    let mut writer = PackWriter::new(iso, Flags::default(), 0).map_err(refused)?;
    for word in words {
        writer
            .add_word(&word.spelling, word.freq)
            .map_err(refused)?;
    }
    Ok(writer.finish().map_err(refused)?.len())
}
