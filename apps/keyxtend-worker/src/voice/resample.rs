//! Microphone samples to what the job keeps, chunk by chunk: interleaved channels averaged to mono, the rate changed to
//! `SOUND_HZ`, and floats made 16-bit. Going down averages each output's window and going up draws straight lines
//! between samples, as the spike did; a stream cut into chunks gives the same samples as one whole buffer.

use super::wipe::Buf;
use kx_ipc::VoiceFault;

/// Averages interleaved frames to one channel, keeping a frame cut between chunks for the next one.
pub struct Mixer {
    channels: u16,
    rest: Buf<f32>,
}

impl Mixer {
    /// A mixer of `channels` interleaved channels, at least 1.
    pub fn new(channels: u16) -> Self {
        Self {
            channels: channels.max(1),
            rest: Buf::new(),
        }
    }

    /// The mono samples of the whole frames `x` completes.
    ///
    /// # Errors
    /// `Memory` when a buffer cannot grow.
    pub fn take(&mut self, mut x: &[f32]) -> Result<Buf<f32>, VoiceFault> {
        let n = usize::from(self.channels);
        let mut out = Buf::new();
        out.reserve((self.rest.len() + x.len()) / n)?;
        if !self.rest.is_empty() {
            let (head, tail) = x.split_at((n - self.rest.len()).min(x.len()));
            self.rest.extend(head)?;
            x = tail;
            if self.rest.len() < n {
                return Ok(out);
            }
            out.push(self.mean(&self.rest))?;
            self.rest.clear();
        }
        let mut frames = x.chunks_exact(n);
        for frame in &mut frames {
            out.push(self.mean(frame))?;
        }
        self.rest.extend(frames.remainder())?;
        Ok(out)
    }

    /// One frame's channels averaged.
    fn mean(&self, frame: &[f32]) -> f32 {
        frame.iter().sum::<f32>() / f32::from(self.channels)
    }
}

/// Changes the rate of a mono stream from `from` to `to` samples a second as chunks arrive.
pub struct Resampler {
    from: u64,
    to: u64,
    /// The inputs still needed, the first being input number `base`.
    held: Buf<f32>,
    base: u64,
    /// Inputs taken so far.
    seen: u64,
    /// The next output's number.
    next: u64,
}

impl Resampler {
    /// A resampler from `from` to `to` samples a second, each at least 1.
    pub fn new(from: u32, to: usize) -> Self {
        Self {
            from: u64::from(from.max(1)),
            to: u64::try_from(to.max(1)).unwrap_or(u64::MAX),
            held: Buf::new(),
            base: 0,
            seen: 0,
            next: 0,
        }
    }

    /// The outputs `x` completes.
    ///
    /// # Errors
    /// `Memory` when a buffer cannot grow.
    pub fn push(&mut self, x: &[f32]) -> Result<Buf<f32>, VoiceFault> {
        self.held.extend(x)?;
        self.seen += x.len() as u64;
        self.run(false)
    }

    /// The last outputs, the final inputs standing in for the ones that never came.
    ///
    /// # Errors
    /// `Memory` when a buffer cannot grow.
    pub fn flush(&mut self) -> Result<Buf<f32>, VoiceFault> {
        self.run(true)
    }

    /// Every output ready now, or at the `end` every one left; then drops the inputs no output needs.
    fn run(&mut self, end: bool) -> Result<Buf<f32>, VoiceFault> {
        let mut out = Buf::new();
        while let Some(y) = self.one(self.next, end) {
            out.push(y)?;
            self.next += 1;
        }
        let first = (self.next * self.from / self.to).min(self.seen);
        let used = usize::try_from(first - self.base).unwrap_or(usize::MAX);
        self.held.drain_front(used);
        self.base = first;
        Ok(out)
    }

    /// Output `i`, or `None` until its inputs came; past the last input only at the `end`.
    #[allow(
        clippy::cast_precision_loss,
        reason = "window sizes and the step's fraction are far below f32's exact range"
    )]
    fn one(&self, i: u64, end: bool) -> Option<f32> {
        if i >= self.seen * self.to / self.from {
            return None;
        }
        let a = i * self.from / self.to;
        if self.from > self.to {
            let b = (i + 1) * self.from / self.to;
            if b > self.seen && !end {
                return None;
            }
            let b = b.min(self.seen).max(a + 1);
            let window = self.slice(a, b);
            return Some(window.iter().sum::<f32>() / window.len() as f32);
        }
        if self.from == self.to {
            return Some(self.at(a));
        }
        if a + 1 >= self.seen && !end {
            return None;
        }
        let (y0, y1) = (self.at(a), self.at((a + 1).min(self.seen - 1)));
        let t = ((i * self.from) % self.to) as f32 / self.to as f32;
        Some(y0 + (y1 - y0) * t)
    }

    /// Input number `n`, which the outputs still need.
    fn at(&self, n: u64) -> f32 {
        self.slice(n, n + 1).first().copied().unwrap_or_default()
    }

    /// Inputs `a` to `b`, which the outputs still need.
    fn slice(&self, a: u64, b: u64) -> &[f32] {
        let at = |n: u64| usize::try_from(n - self.base).unwrap_or(usize::MAX);
        self.held.get(at(a)..at(b)).unwrap_or_default()
    }
}

/// Floats in -1 to 1 as 16-bit samples, rounded and clamped.
///
/// # Errors
/// `Memory` when the buffer cannot be had.
pub fn pcm16(x: &[f32]) -> Result<Buf<i16>, VoiceFault> {
    let mut out = Buf::new();
    out.reserve(x.len())?;
    for &v in x {
        out.push(sample(v))?;
    }
    Ok(out)
}

/// 16-bit samples as floats in -1 to 1, the way back from `pcm16`, for the detector and the engine.
///
/// # Errors
/// `Memory` when the buffer cannot be had.
#[cfg(any(test, feature = "engine"))]
pub fn floats(x: &[i16]) -> Result<Buf<f32>, VoiceFault> {
    let full = f32::from(i16::MAX);
    Buf::collect(x.iter().map(|&s| (f32::from(s) / full).max(-1.0)))
}

/// One float as a 16-bit sample, full scale being the largest one.
#[allow(
    clippy::cast_possible_truncation,
    reason = "the value is clamped to the 16-bit range first; NaN becomes 0"
)]
fn sample(v: f32) -> i16 {
    (v.clamp(-1.0, 1.0) * f32::from(i16::MAX)).round() as i16
}

#[cfg(test)]
mod tests;
