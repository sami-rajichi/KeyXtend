//! The speaker port: short clips that play at once, so the UI never waits on audio.

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

/// A sound compiled into the app: mono PCM16 little-endian samples, 2-byte aligned, never copied or parsed.
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
    /// Keeps `clips` and `config` for later plays; it starts nothing.
    fn prepare(&self, clips: Vec<Clip>, config: SpeakerConfig);

    /// Plays clip `id` at linear `gain`, from 0.0 to 1.0, under the system volume; it returns at once.
    fn play(&self, id: ClipId, gain: f32);
}

#[cfg(test)]
mod tests {
    use super::*;

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
