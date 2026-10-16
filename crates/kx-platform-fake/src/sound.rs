//! A fake speaker: it keeps clips and plays as the `Speaker` contract says, records them in order, and plays nothing.

use crate::lock;
use kx_platform::ui::{Clip, ClipId, Speaker, SpeakerConfig, level};
use std::sync::Mutex;

/// One `prepare`: the ids of the clips it kept, in order, and the config.
pub type Prepared = (Vec<ClipId>, SpeakerConfig);

/// A speaker that records what it kept and what it would play.
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

    /// Every play so far, as the clip and its clamped gain, in order; a play the contract drops is not here.
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
        let ids = clips.iter().filter(|c| c.playable()).map(|c| c.id);
        lock(&self.prepared).push((ids.collect(), config));
    }

    fn play(&self, id: ClipId, gain: f32) {
        let kept = lock(&self.prepared)
            .last()
            .is_some_and(|(ids, _)| ids.contains(&id));
        if let Some(gain) = level(gain).filter(|_| kept) {
            lock(&self.plays).push((id, gain));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: ClipId = ClipId::new(1);
    const DROP: ClipId = ClipId::new(2);
    const BAD: ClipId = ClipId::new(3);
    const CONFIG: SpeakerConfig = SpeakerConfig {
        voices: 3,
        idle_ms: 3_000,
    };
    /// Two whole samples.
    static SAMPLES: [u8; 4] = [0; 4];

    fn clip(id: ClipId) -> Clip {
        Clip {
            id,
            rate: *Clip::RATES.start(),
            samples: &SAMPLES,
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

    #[test]
    fn it_skips_a_clip_no_speaker_plays_and_clamps_or_drops_a_play_as_the_contract_says() {
        let speaker = FakeSpeaker::new();
        speaker.play(KEY, 1.0);
        assert!(speaker.plays().is_empty(), "nothing before a prepare");
        let odd = Clip {
            samples: &SAMPLES[..3],
            ..clip(BAD)
        };
        let slow = Clip {
            rate: Clip::RATES.start() - 1,
            ..clip(BAD)
        };
        speaker.prepare(vec![odd, clip(KEY), slow], CONFIG);
        assert_eq!(speaker.prepared(), [(vec![KEY], CONFIG)], "skipped");
        for (id, gain) in [(KEY, 2.0), (KEY, -1.0), (KEY, f32::NAN), (BAD, 1.0)] {
            speaker.play(id, gain);
        }
        assert_eq!(speaker.take_plays(), [(KEY, 1.0), (KEY, 0.0)]);
        speaker.prepare(vec![clip(DROP)], CONFIG);
        speaker.play(KEY, 1.0);
        speaker.play(DROP, 0.5);
        assert_eq!(speaker.take_plays(), [(DROP, 0.5)], "replaced");
    }
}
