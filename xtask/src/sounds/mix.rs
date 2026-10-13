//! Joining two sounds in the shipping format, the second starting a set time after the first.

use kx_theme::sounds::wav::samples as shipped_samples;

use super::recipe::Book;
use super::units::samples;
use super::{FULL_GAIN_DB, Sound, SoundError, finish, wav};

/// The samples of the WAV file `bytes`, named `name`, which must be a sound kx-ui's build takes in `book`'s format.
pub(crate) fn shipping(book: &Book, name: &str, bytes: &[u8]) -> Result<Vec<f64>, SoundError> {
    let audio = wav::read(bytes)?;
    match shipped_samples(bytes, &book.format) {
        Ok(_) => Ok(audio.samples),
        Err(_) => Err(SoundError::NotShipping(name.to_string())),
    }
}

/// Mixes `second` into `first` from `at_ms`, then fades and peaks the result as an import does.
pub(crate) fn join(
    book: &Book,
    first: &[f64],
    second: &[f64],
    at_ms: u32,
) -> Result<Sound, SoundError> {
    let at = samples(f64::from(at_ms), f64::from(book.format.rate));
    let len = first.len().max(at.saturating_add(second.len()));
    if len > book.format.max_frames() {
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
