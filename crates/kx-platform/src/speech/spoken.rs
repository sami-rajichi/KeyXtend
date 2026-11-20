//! A listen-only run's speech as the job hands it over at Stop: taken only in order up to the piece marked last and
//! within the longest recording, and given out as fresh copies once the run's end came. Anything else wipes it and
//! marks the job at fault.

use super::router::secret;
use kx_ipc::{MAX_RECORD_BYTES, MAX_SPEECH_PIECES, SoundPiece};
use kx_module_api::SecretBytes;
use kx_module_api::ports::SpeechError;

/// A run's speech so far.
#[derive(Default)]
pub(super) struct Spoken {
    pieces: Vec<SecretBytes>,
    bytes: usize,
    /// The run's end came.
    done: bool,
    /// The piece marked last came.
    whole: bool,
    /// A piece came out of order, after the last or the end, or past a bound.
    broken: bool,
}

impl Spoken {
    /// Takes `piece` when it is the next in order and fits; anything else wipes the speech and breaks it.
    pub(super) fn take(&mut self, piece: SoundPiece<'static>) {
        let len = piece.bytes.len();
        let next = usize::from(piece.index) == self.pieces.len();
        let fits = self.pieces.len() < MAX_SPEECH_PIECES && self.bytes + len <= MAX_RECORD_BYTES;
        if self.broken || self.done || self.whole || !next || !fits {
            *self = Self {
                done: self.done,
                broken: true,
                ..Self::default()
            };
            return;
        }
        (self.bytes, self.whole) = (self.bytes + len, piece.last);
        self.pieces.push(secret(piece.bytes));
    }

    /// Marks the run's end, after which the speech is whole.
    pub(super) const fn end(&mut self) {
        self.done = true;
    }

    /// Fresh copies of the whole speech.
    ///
    /// # Errors
    /// `NoSound` before the run's end, `Internal` once broken or when the piece marked last never came.
    pub(super) fn give(&self) -> Result<Vec<SecretBytes>, SpeechError> {
        let cut = !self.whole && !self.pieces.is_empty();
        match (self.done, self.broken || cut) {
            (false, _) => Err(SpeechError::NoSound),
            (true, true) => Err(SpeechError::Internal),
            (true, false) => Ok(self.pieces.clone()),
        }
    }
}
