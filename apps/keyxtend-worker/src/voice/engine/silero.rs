//! The shipped Silero detector through the engine, on the processor: the file loads once per job and each run gets a
//! fresh stream of its own, one probability a frame of 32 ms.

use super::transcribe;
use crate::voice::cut::{Detector, Detectors};
use crate::voice::detect::{Probe, SILERO, Voiced};
use kx_ipc::{SOUND_HZ, VoiceFault};
use std::path::Path;
use transcribe_cpp::{Backend, Model, ModelOptions, VadSession, VadSessionOptions};

/// Processor threads for the detector: it is small, and the model's run needs the rest.
const THREADS: i32 = 1;

/// Opens detectors from the setup's file, keeping the last file loaded.
#[derive(Default)]
pub struct SileroFiles {
    kept: Option<(String, Model)>,
}

impl Detectors for SileroFiles {
    fn open(&mut self, path: &str) -> Result<Option<Box<dyn Detector>>, VoiceFault> {
        if !transcribe::start() {
            return Err(VoiceFault::Runner);
        }
        let probe = Silero::new(self.model(path)?)?;
        Ok(Some(Box::new(Voiced::new(Box::new(probe), &SILERO))))
    }
}

impl SileroFiles {
    /// The detector model in `path`, loaded on the processor unless kept.
    fn model(&mut self, path: &str) -> Result<&Model, VoiceFault> {
        if self.kept.as_ref().is_none_or(|(p, _)| p != path) {
            if !Path::new(path).is_file() {
                return Err(VoiceFault::NoFile);
            }
            let options = ModelOptions {
                backend: Backend::Cpu,
                ..ModelOptions::default()
            };
            let model = Model::load_with(path, &options).map_err(|_| VoiceFault::BadFile)?;
            self.kept = Some((path.to_owned(), model));
        }
        self.kept
            .as_ref()
            .map(|(_, m)| m)
            .ok_or(VoiceFault::Internal)
    }
}

/// One run's stream of Silero probabilities.
struct Silero {
    session: VadSession,
    frame: usize,
}

impl Silero {
    /// A fresh stream over `model`; `BadFile` unless it is a voice detector of the job's rate.
    fn new(model: &Model) -> Result<Self, VoiceFault> {
        let info = model.vad_info().map_err(|_| VoiceFault::BadFile)?;
        let rate = usize::try_from(info.sample_rate).ok();
        let frame = usize::try_from(info.frame_samples).unwrap_or(0);
        if rate != Some(SOUND_HZ) || frame == 0 {
            return Err(VoiceFault::BadFile);
        }
        let options = VadSessionOptions { n_threads: THREADS };
        let session = model
            .vad_session_with(&options)
            .map_err(|_| VoiceFault::BadFile)?;
        Ok(Self { session, frame })
    }
}

impl Probe for Silero {
    fn frame(&self) -> usize {
        self.frame
    }

    fn prob(&mut self, frame: &[f32]) -> Option<f32> {
        let scored = self.session.stream_feed(frame).ok()?;
        scored.probs.last().copied()
    }
}
