//! Where a finished piece goes: the engine seam that P11 task 12 fills with its runner. This build has no engine, so a
//! run with a model is `Runner`, and only listening without one works.

use super::Tell;
use super::wipe::Buf;
use kx_ipc::{Model, VoiceFault};

/// Writes one run's pieces in the background, telling each text as a `Piece` note in order, then `Done`. Dropping it
/// drops the pieces not yet written and tells nothing more.
pub trait Writer {
    /// Takes the next piece: its sound with the silence around its speech cut, `pad_ms` kept.
    fn piece(&mut self, sound: Buf<i16>);

    /// No piece follows; `heard_ms` of sound were heard in all. `Done` follows once each piece is written.
    fn end(&mut self, heard_ms: u32);
}

/// Loads a model for a run and gives the writer of its pieces.
pub trait Engine {
    /// A writer for run `id` on `model`, whose notes go to `tell`; the load goes on in the background.
    ///
    /// # Errors
    /// The fault that keeps the model from serving, such as `Runner`, `NoFile` or `OutOfStep` for a second model.
    fn load(
        &mut self,
        id: u32,
        model: &Model<'_>,
        tell: Tell,
    ) -> Result<Box<dyn Writer>, VoiceFault>;
}

/// This build's engine: none yet.
pub struct NoEngine;

impl Engine for NoEngine {
    fn load(&mut self, _: u32, _: &Model<'_>, _: Tell) -> Result<Box<dyn Writer>, VoiceFault> {
        Err(VoiceFault::Runner)
    }
}
