//! The default microphone (WASAPI through cpal): records into memory until stopped, full or lost.

use std::fmt::Display;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, ErrorKind, FromSample, SampleFormat, SizedSample, Stream, StreamConfig};

use crate::resample::{mono, pcm16, to_rate};
use crate::wav;

/// Why the mic stopped by itself.
#[derive(Debug, PartialEq, Eq)]
pub enum Stop {
    /// The recording reached its cap.
    Full,
    /// The mic stream failed, for example because the device was unplugged.
    Lost(String),
}

/// Called from the audio thread, at most once per reason.
pub type OnStop = Arc<dyn Fn(Stop) + Send + Sync>;
/// Samples shared with the audio thread.
type Buf = Arc<Mutex<Vec<f32>>>;

/// A recording in progress.
pub struct Mic {
    stream: Stream,
    buf: Buf,
    channels: u16,
    rate: u32,
}

/// A finished recording: interleaved samples as the mic gave them.
#[derive(Default)]
pub struct Clip {
    samples: Vec<f32>,
    channels: u16,
    rate: u32,
}

/// Passes each stop reason on at most once, whichever audio thread sees it.
struct Reporter {
    on_stop: OnStop,
    full: AtomicBool,
    lost: AtomicBool,
}

impl Mic {
    /// Starts recording; after `max_s` seconds it stops taking sound and calls `on_stop` with `Full`.
    pub fn start(max_s: u64, on_stop: OnStop) -> Result<Mic, String> {
        let dev = cpal::default_host()
            .default_input_device()
            .ok_or("no microphone found")?;
        let sup = dev
            .default_input_config()
            .map_err(|e| format!("microphone: {e}"))?;
        let cfg = sup.config();
        let per_s = u64::from(cfg.sample_rate) * u64::from(cfg.channels);
        let (samples, cap) = reserve(max_s, per_s)?;
        let buf: Buf = Arc::new(Mutex::new(samples));
        let r = Arc::new(Reporter::new(on_stop));
        let stream = match sup.sample_format() {
            SampleFormat::F32 => build::<f32>(&dev, &cfg, &buf, cap, r),
            SampleFormat::I16 => build::<i16>(&dev, &cfg, &buf, cap, r),
            SampleFormat::I32 => build::<i32>(&dev, &cfg, &buf, cap, r),
            SampleFormat::U16 => build::<u16>(&dev, &cfg, &buf, cap, r),
            other => Err(format!("microphone sample format {other} is not handled")),
        }?;
        stream.play().map_err(|e| format!("microphone: {e}"))?;
        Ok(Mic {
            stream,
            buf,
            channels: cfg.channels,
            rate: cfg.sample_rate,
        })
    }

    /// Stops recording and hands over the sound.
    pub fn stop(self) -> Clip {
        let Mic {
            stream,
            buf,
            channels,
            rate,
        } = self;
        drop(stream);
        let samples = buf
            .lock()
            .map(|mut b| std::mem::take(&mut *b))
            .unwrap_or_default();
        Clip {
            samples,
            channels,
            rate,
        }
    }
}

/// An input stream that appends samples as floats up to `cap`, and reports a full or lost stream once.
fn build<T>(
    dev: &Device,
    cfg: &StreamConfig,
    buf: &Buf,
    cap: usize,
    report: Arc<Reporter>,
) -> Result<Stream, String>
where
    T: SizedSample,
    f32: FromSample<T>,
{
    let (buf, r) = (Arc::clone(buf), Arc::clone(&report));
    let data = move |data: &[T], _: &cpal::InputCallbackInfo| {
        let Ok(mut b) = buf.lock() else { return };
        if append(&mut b, data, cap) {
            r.full();
        }
    };
    let error = move |e: cpal::Error| {
        if is_lost(e.kind()) {
            report.lost(e);
        }
    };
    dev.build_input_stream(*cfg, data, error, None)
        .map_err(|e| format!("microphone: {e}"))
}

/// An empty buffer with room for `max_s` seconds of `per_s` samples a second, and that sample count.
fn reserve(max_s: u64, per_s: u64) -> Result<(Vec<f32>, usize), String> {
    let too_long = || format!("a {max_s} s recording does not fit in memory");
    let cap = max_s
        .checked_mul(per_s)
        .and_then(|n| usize::try_from(n).ok())
        .ok_or_else(too_long)?;
    let mut v = Vec::new();
    v.try_reserve_exact(cap).map_err(|_| too_long())?;
    Ok((v, cap))
}

impl Reporter {
    /// A reporter that has told nothing yet.
    fn new(on_stop: OnStop) -> Self {
        Reporter {
            on_stop,
            full: AtomicBool::new(false),
            lost: AtomicBool::new(false),
        }
    }

    /// The recording is full.
    fn full(&self) {
        if !self.full.swap(true, Ordering::Relaxed) {
            (self.on_stop)(Stop::Full);
        }
    }

    /// The stream failed with `e`.
    fn lost(&self, e: impl Display) {
        if !self.lost.swap(true, Ordering::Relaxed) {
            (self.on_stop)(Stop::Lost(format!("microphone: {e}")));
        }
    }
}

/// Appends `data` to `b` without passing `cap`; true once `b` is full.
fn append<T>(b: &mut Vec<f32>, data: &[T], cap: usize) -> bool
where
    T: SizedSample,
    f32: FromSample<T>,
{
    let room = cap.saturating_sub(b.len());
    b.extend(data.iter().take(room).map(|s| s.to_sample::<f32>()));
    b.len() >= cap
}

/// A stream error that ends the recording; a glitch (xrun) does not.
fn is_lost(k: ErrorKind) -> bool {
    k != ErrorKind::Xrun
}

impl Clip {
    /// Nothing was recorded.
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// The clip as a mono 16-bit WAV at `rate_hz`.
    pub fn wav(&self, rate_hz: u32) -> Vec<u8> {
        let one = mono(&self.samples, self.channels);
        wav::encode(&pcm16(&to_rate(&one, self.rate, rate_hz)), rate_hz)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stereo_48k_clip_becomes_a_16k_mono_wav_of_the_same_length() {
        let clip = Clip {
            samples: vec![0.25; 2 * 4800],
            channels: 2,
            rate: 48000,
        };
        assert_eq!(wav::audio_ms(&clip.wav(16000)), Ok(100));
        assert!(!clip.is_empty());
    }

    #[test]
    fn samples_stop_at_the_cap() {
        let mut b = Vec::new();
        assert!(!append(&mut b, &[0.5f32; 3], 5));
        assert!(append(&mut b, &[0.5f32; 3], 5));
        assert_eq!(b.len(), 5);
        assert!(append(&mut b, &[0.5f32; 3], 5), "still full");
        assert_eq!(b.len(), 5);
    }

    #[test]
    fn a_glitch_keeps_recording_but_a_lost_device_stops_it() {
        assert!(!is_lost(ErrorKind::Xrun));
        assert!(is_lost(ErrorKind::DeviceNotAvailable));
        assert!(is_lost(ErrorKind::StreamInvalidated));
    }

    #[test]
    fn an_absurd_recording_length_is_an_error_not_a_panic() {
        assert!(reserve(u64::MAX, 2).is_err(), "the sample count overflows");
        assert!(reserve(u64::MAX / 2, 1).is_err(), "the bytes overflow");
        let (v, cap) = reserve(2, 8).expect("fits");
        assert!(cap == 16 && v.is_empty() && v.capacity() >= cap);
    }

    #[test]
    fn each_stop_reason_is_reported_once() {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let log = Arc::clone(&seen);
        let r = Reporter::new(Arc::new(move |s: Stop| log.lock().expect("lock").push(s)));
        r.full();
        r.lost("unplugged");
        r.full();
        r.lost("again");
        let want = [Stop::Full, Stop::Lost("microphone: unplugged".into())];
        assert_eq!(*seen.lock().expect("lock"), want);
    }
}
