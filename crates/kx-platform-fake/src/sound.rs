//! A fake speaker: it records each prepare and each play in order, and plays nothing.

use crate::lock;
use kx_platform::ui::{Clip, ClipId, Speaker, SpeakerConfig};
use std::sync::Mutex;

/// One `prepare`: the clips' ids in order, and the config.
pub type Prepared = (Vec<ClipId>, SpeakerConfig);

/// A speaker that records what it was asked.
#[derive(Debug, Default)]
pub struct FakeSpeaker {
    prepared: Mutex<Vec<Prepared>>,
    plays: Mutex<Vec<(ClipId, f32)>>,
}

impl FakeSpeaker {
    /// A speaker that was asked nothing yet.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Every `prepare` so far, in order.
    #[must_use]
    pub fn prepared(&self) -> Vec<Prepared> {
        lock(&self.prepared).clone()
    }

    /// Every play so far, as the clip and its gain, in order.
    #[must_use]
    pub fn plays(&self) -> Vec<(ClipId, f32)> {
        lock(&self.plays).clone()
    }

    /// The plays so far, which it forgets.
    pub fn take_plays(&self) -> Vec<(ClipId, f32)> {
        std::mem::take(&mut *lock(&self.plays))
    }
}

impl Speaker for FakeSpeaker {
    fn prepare(&self, clips: Vec<Clip>, config: SpeakerConfig) {
        let ids = clips.iter().map(|c| c.id).collect();
        lock(&self.prepared).push((ids, config));
    }

    fn play(&self, id: ClipId, gain: f32) {
        lock(&self.plays).push((id, gain));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: ClipId = ClipId::new(1);
    const DROP: ClipId = ClipId::new(2);
    const CONFIG: SpeakerConfig = SpeakerConfig {
        voices: 3,
        idle_ms: 3_000,
    };

    fn clip(id: ClipId) -> Clip {
        Clip {
            id,
            rate: 48_000,
            samples: &[],
        }
    }

    #[test]
    fn it_records_each_prepare_and_each_play_in_order() {
        let speaker = FakeSpeaker::new();
        speaker.prepare(vec![clip(KEY), clip(DROP)], CONFIG);
        speaker.play(DROP, 0.5);
        speaker.play(KEY, 1.0);
        assert_eq!(speaker.prepared(), [(vec![KEY, DROP], CONFIG)]);
        assert_eq!(speaker.plays(), [(DROP, 0.5), (KEY, 1.0)]);
        assert_eq!(speaker.take_plays(), [(DROP, 0.5), (KEY, 1.0)]);
        assert!(speaker.plays().is_empty(), "taken plays are forgotten");
        assert_eq!(speaker.prepared().len(), 1, "prepares stay");
    }
}
