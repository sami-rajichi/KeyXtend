//! The speaker port: short clips that play at once, so the UI never waits on audio.

use std::ops::RangeInclusive;

/// Names one clip; the app picks the values.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ClipId(u16);

impl ClipId {
    /// Wraps the app's raw clip number.
    #[must_use]
    pub const fn new(raw: u16) -> Self {
        Self(raw)
    }

    /// The raw clip number.
    #[must_use]
    pub const fn raw(self) -> u16 {
        self.0
    }
}

/// A sound compiled into the app: mono PCM16 little-endian samples, never copied or parsed.
#[derive(Copy, Clone)]
pub struct Clip {
    /// Its name for `play`.
    pub id: ClipId,
    /// Samples per second.
    pub rate: u32,
    /// The samples' bytes.
    pub samples: &'static [u8],
}

impl Clip {
    /// Channels in every clip: mono.
    pub const CHANNELS: u16 = 1;
    /// Bytes in one sample: 16-bit PCM.
    pub const SAMPLE_BYTES: u16 = 2;
    /// The rates, in samples per second, that every speaker adapter must play.
    pub const RATES: RangeInclusive<u32> = 8_000..=192_000;

    /// True if the samples are one or more whole samples, with no partial one at the end.
    #[must_use]
    pub fn whole(&self) -> bool {
        let sample = usize::from(Self::CHANNELS * Self::SAMPLE_BYTES);
        !self.samples.is_empty() && self.samples.len().is_multiple_of(sample)
    }

    /// True if every speaker plays the clip: whole samples at a rate in `RATES`.
    #[must_use]
    pub fn playable(&self) -> bool {
        self.whole() && Self::RATES.contains(&self.rate)
    }
}

/// The lowest and highest gain a play takes.
const SILENT: f32 = 0.0;
const FULL: f32 = 1.0;

/// The gain a speaker plays `gain` at: clamped to 0.0–1.0, or `None` for NaN, which plays nothing.
#[must_use]
pub fn level(gain: f32) -> Option<f32> {
    (!gain.is_nan()).then(|| gain.clamp(SILENT, FULL))
}

/// Shows the clip's size, never its samples.
impl std::fmt::Debug for Clip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Clip")
            .field("id", &self.id)
            .field("rate", &self.rate)
            .field("bytes", &self.samples.len())
            .finish()
    }
}

/// How the speaker plays.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SpeakerConfig {
    /// Voices per clip, so quick plays of one clip overlap.
    pub voices: usize,
    /// Silence, in milliseconds, after which the audio engine stops.
    pub idle_ms: u32,
}

/// Plays short clips; it never blocks or fails the caller.
pub trait Speaker: Send + Sync {
    /// Keeps `clips` and `config` for later plays, in place of any earlier ones; it starts nothing.
    /// A clip that is not [`Clip::playable`] (empty, a partial sample, or a rate outside [`Clip::RATES`]) is skipped.
    fn prepare(&self, clips: Vec<Clip>, config: SpeakerConfig);

    /// Plays clip `id` at linear `gain` under the system volume, clamped to 0.0–1.0 by [`level`]; it returns at once.
    /// NaN, or an id the last `prepare` did not keep, plays nothing.
    fn play(&self, id: ClipId, gain: f32);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_adapter_plays_the_common_rates() {
        for rate in [8_000, 16_000, 44_100, 48_000, 96_000, 192_000] {
            assert!(Clip::RATES.contains(&rate), "{rate} Hz");
        }
    }

    #[test]
    fn only_whole_samples_at_a_listed_rate_play() {
        static TWO: [u8; 4] = [0; 4];
        let clip = |samples: &'static [u8], rate| Clip {
            id: ClipId::new(1),
            rate,
            samples,
        };
        let (low, high) = (*Clip::RATES.start(), *Clip::RATES.end());
        for rate in [low, high] {
            assert!(clip(&TWO, rate).playable(), "{rate} Hz");
        }
        for rate in [0, low - 1, high + 1] {
            let slow = clip(&TWO, rate);
            assert!(slow.whole() && !slow.playable(), "{rate} Hz");
        }
        for samples in [&TWO[..0], &TWO[..3]] {
            assert!(!clip(samples, low).whole(), "{} bytes", samples.len());
            assert!(!clip(samples, low).playable());
        }
    }

    #[test]
    fn a_gain_out_of_range_is_clamped_and_nan_plays_nothing() {
        assert_eq!(level(2.0), Some(1.0));
        assert_eq!(level(-0.5), Some(0.0));
        assert_eq!(level(0.25), Some(0.25));
        assert_eq!(level(f32::INFINITY), Some(1.0));
        assert_eq!(level(f32::NEG_INFINITY), Some(0.0));
        assert_eq!(level(f32::NAN), None);
    }

    #[test]
    fn a_clip_id_keeps_its_raw_value() {
        assert_eq!(ClipId::new(u16::MAX).raw(), u16::MAX);
    }

    #[test]
    fn a_clip_shows_its_size_but_not_its_samples() {
        static SAMPLES: [u8; 4] = [0xAB, 0xCD, 0x12, 0x34];
        let clip = Clip {
            id: ClipId::new(7),
            rate: 48_000,
            samples: &SAMPLES,
        };
        let shown = format!("{clip:?}");
        assert!(
            shown.contains("bytes: 4") && shown.contains("48000"),
            "{shown}"
        );
        assert!(!shown.contains("171"), "no sample value: {shown}");
    }
}
