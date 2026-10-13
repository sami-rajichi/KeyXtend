//! Joining two sounds in the shipping format, the second starting a set time after the first.

use super::recipe::Book;
use super::units::samples;
use super::{FULL_GAIN_DB, Sound, SoundError, finish, wav};

/// The samples of the WAV file `bytes`, named `name`, which must already be in `book`'s shipping format.
pub(crate) fn shipping(book: &Book, name: &str, bytes: &[u8]) -> Result<Vec<f64>, SoundError> {
    let audio = wav::read(bytes)?;
    let format = &book.format;
    let layout = (audio.rate, audio.channels, audio.bits);
    if layout == (format.rate, format.channels, format.bits) {
        Ok(audio.samples)
    } else {
        Err(SoundError::NotShipping(name.to_string()))
    }
}

/// Mixes `second` into `first` from `at_ms`, then fades and peaks the result as an import does.
pub(crate) fn join(
    book: &Book,
    first: &[f64],
    second: &[f64],
    at_ms: u32,
) -> Result<Sound, SoundError> {
    let rate = f64::from(book.format.rate);
    let at = samples(f64::from(at_ms), rate);
    let len = first.len().max(at.saturating_add(second.len()));
    if len > samples(book.format.max_ms, rate) {
        return Err(SoundError::TooLong(book.format.max_ms));
    }
    let mut mix = vec![0.0; len];
    for (out, x) in mix.iter_mut().zip(first) {
        *out += x;
    }
    for (out, x) in mix.iter_mut().skip(at).zip(second) {
        *out += x;
    }
    finish::master(&mut mix, &book.format, FULL_GAIN_DB)
}

#[cfg(test)]
mod tests;
