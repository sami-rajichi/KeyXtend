//! The engine seam: a run's writer of pieces, and the engine that loads a model and answers the devices, a file's facts
//! and a card's check, each by notes through `Tell`. Built without the `engine` feature the job has none, so a run with
//! a model and every engine ask answer `Runner`.

#[cfg(feature = "engine")]
mod silero;
#[cfg(feature = "engine")]
mod transcribe;

use super::Tell;
use super::cut::Detectors;
#[cfg(not(feature = "engine"))]
use super::cut::NoDetectors;
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

/// Loads a model for a run and answers the asks that need the engine; none of its calls waits for the engine.
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

    /// Lists the devices; a `Devices` note or a fault follows on `tell`.
    ///
    /// # Errors
    /// `Runner` when this build has no engine.
    fn devices(&mut self, id: u32, tell: Tell) -> Result<(), VoiceFault>;

    /// Reads the model file at `path`; a `Facts` note or a fault follows on `tell`.
    ///
    /// # Errors
    /// `Runner` when this build has no engine, `NoFile` when the file is missing.
    fn facts(&mut self, id: u32, path: &str, tell: Tell) -> Result<(), VoiceFault>;

    /// Checks `model`'s card with the clip at `clip`; `Loading`, `Warm` and a `Checked` note or a fault follow.
    ///
    /// # Errors
    /// `Runner`, `NoFile`, or `OutOfStep` for a model aimed at the processor.
    fn check(
        &mut self,
        id: u32,
        model: &Model<'_>,
        clip: &str,
        tell: Tell,
    ) -> Result<(), VoiceFault>;
}

/// The engine of a build without one.
#[cfg(any(test, not(feature = "engine")))]
pub struct NoEngine;

#[cfg(any(test, not(feature = "engine")))]
impl Engine for NoEngine {
    fn load(&mut self, _: u32, _: &Model<'_>, _: Tell) -> Result<Box<dyn Writer>, VoiceFault> {
        Err(VoiceFault::Runner)
    }

    fn devices(&mut self, _: u32, _: Tell) -> Result<(), VoiceFault> {
        Err(VoiceFault::Runner)
    }

    fn facts(&mut self, _: u32, _: &str, _: Tell) -> Result<(), VoiceFault> {
        Err(VoiceFault::Runner)
    }

    fn check(&mut self, _: u32, _: &Model<'_>, _: &str, _: Tell) -> Result<(), VoiceFault> {
        Err(VoiceFault::Runner)
    }
}

/// This build's engine: transcribe.cpp on the real cores, or none.
pub fn live() -> Box<dyn Engine> {
    #[cfg(feature = "engine")]
    {
        let cores = super::cards::cores();
        Box::new(super::run::Local::new(transcribe::Transcribe, cores))
    }
    #[cfg(not(feature = "engine"))]
    {
        Box::new(NoEngine)
    }
}

/// This build's voice detectors: Silero through the engine, or none, so a run listens without cutting.
pub fn detectors() -> Box<dyn Detectors> {
    #[cfg(feature = "engine")]
    {
        Box::new(silero::SileroFiles::default())
    }
    #[cfg(not(feature = "engine"))]
    {
        Box::new(NoDetectors)
    }
}
