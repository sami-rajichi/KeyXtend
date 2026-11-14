//! The voice detector behind `Detector`: a speech probability per frame, judged as Silero's own segmenter does. Speech
//! starts at `threshold`, ends after `min_silence_ms` under the lower mark, and counts only once `min_speech_ms` long,
//! so the first frames of speech are told late and the cutter takes them back through `pending`.

use super::cut::Detector;
use super::resample::floats;
use kx_ipc::SOUND_HZ;

/// How speech probabilities become speech and silence.
pub struct Tune {
    /// A frame at or above it starts or keeps speech.
    pub threshold: f32,
    /// The lower mark sits this far under `threshold`, at least `floor`.
    pub fall: f32,
    /// The lowest the lower mark goes.
    pub floor: f32,
    /// Silence under the lower mark this long ends speech.
    pub min_silence_ms: u32,
    /// Speech shorter than this is dropped.
    pub min_speech_ms: u32,
}

/// Silero's own defaults, which the trial settled on the owner's six clips (P11 build proofs, 2026-11-14).
pub const SILERO: Tune = Tune {
    threshold: 0.5,
    fall: 0.15,
    floor: 0.01,
    min_silence_ms: 100,
    min_speech_ms: 250,
};

/// Milliseconds in a second.
const MS_PER_S: usize = 1000;
/// The probability a frame the model could not score counts as: speech, so no sound is lost.
const UNSCORED: f32 = 1.0;

/// A speech probability per frame of 16 kHz floats: Silero through the engine, or a fake in tests.
pub trait Probe {
    /// Samples in one frame, at least 1.
    fn frame(&self) -> usize;

    /// The probability that `frame`, exactly `frame()` samples, holds speech; `None` when it could not be scored.
    fn prob(&mut self, frame: &[f32]) -> Option<f32>;
}

/// Where the judge stands.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum State {
    /// No speech.
    Quiet,
    /// Speech may have started this many frames ago; it is told once long enough.
    Onset(usize),
    /// Speech.
    Speech,
}

/// Judges each frame's probability with hysteresis, as Silero's segmenter does over a whole clip.
pub struct Judge {
    up: f32,
    down: f32,
    /// Frames of silence that end speech, and of speech that count.
    silence: usize,
    speech: usize,
    state: State,
    /// Frames since the probability fell under the lower mark, while it stays under `up`.
    dip: Option<usize>,
}

impl Judge {
    /// A judge of frames of `frame` samples, tuned by `tune`.
    pub fn new(tune: &Tune, frame: usize) -> Self {
        let frame = frame.max(1);
        let samples = |ms: u32| usize::try_from(ms).unwrap_or(usize::MAX) * SOUND_HZ / MS_PER_S;
        Self {
            up: tune.threshold,
            down: (tune.threshold - tune.fall).max(tune.floor),
            silence: samples(tune.min_silence_ms).div_ceil(frame).max(1),
            speech: samples(tune.min_speech_ms) / frame + 1,
            state: State::Quiet,
            dip: None,
        }
    }

    /// True when the frame of probability `p` is told as speech.
    pub fn take(&mut self, p: f32) -> bool {
        if self.state == State::Quiet {
            if p < self.up {
                return false;
            }
            (self.state, self.dip) = (State::Onset(0), None);
        }
        let quiet = self.dip(p) >= self.silence;
        self.state = match self.state {
            _ if quiet => State::Quiet,
            State::Onset(n) if (n + 1).saturating_sub(self.dip.unwrap_or(0)) >= self.speech => {
                State::Speech
            }
            State::Onset(n) => State::Onset(n + 1),
            other => other,
        };
        if quiet {
            self.dip = None;
        }
        self.state == State::Speech
    }

    /// The frames, ending with the last one judged, told as silence that turn into speech if it goes on.
    pub fn pending(&self) -> usize {
        match self.state {
            State::Onset(n) => n,
            State::Quiet | State::Speech => 0,
        }
    }

    /// Counts the dip with the frame of probability `p`, and gives its length.
    fn dip(&mut self, p: f32) -> usize {
        self.dip = match self.dip {
            _ if p >= self.up => None,
            Some(n) => Some(n + 1),
            None if p < self.down => Some(1),
            None => None,
        };
        self.dip.unwrap_or(0)
    }
}

/// A detector over a probe and a judge.
pub struct Voiced {
    probe: Box<dyn Probe>,
    judge: Judge,
}

impl Voiced {
    /// A detector over `probe`, tuned by `tune`.
    pub fn new(probe: Box<dyn Probe>, tune: &Tune) -> Self {
        let judge = Judge::new(tune, probe.frame());
        Self { probe, judge }
    }
}

impl Detector for Voiced {
    fn frame(&self) -> usize {
        self.probe.frame().max(1)
    }

    fn speech(&mut self, frame: &[i16]) -> bool {
        let p = floats(frame)
            .ok()
            .and_then(|pcm| self.probe.prob(&pcm))
            .unwrap_or(UNSCORED);
        self.judge.take(p)
    }

    fn pending(&self) -> usize {
        self.judge.pending()
    }
}

#[cfg(test)]
mod tests;
