//! The values the parts take from `[voice]`, each in its own units: the job's limits, a dictation's ask, the device
//! pick, the bar's place, the card check's rules and the flow's waits.

use std::time::Duration;

use kx_module_api::ports::{
    BarPlace, CardUse, DevicePick, Hundredths, JobLimits, SpeechAsk, SpeechModel,
};

use super::{VoiceSettings, place_named};
use crate::device::CardRules;

impl VoiceSettings {
    /// A voice or fetch job's memory cap and waits.
    #[must_use]
    pub fn job(&self) -> JobLimits {
        JobLimits {
            job_mb: self.job_mb,
            idle_min: self.idle_min,
            ask_ms: self.ask_ms,
            load_s: self.load_s,
            warm_s: self.warm_s,
            piece_wait_s: self.piece_wait_s,
            restart_ms: self.restart_ms,
            restart_max_ms: self.restart_max_ms,
        }
    }

    /// A dictation with `model` on `card`; no model listens only, and no card runs on the processor.
    #[must_use]
    pub fn ask(&self, model: Option<SpeechModel>, card: Option<CardUse>) -> SpeechAsk {
        SpeechAsk {
            job: self.job(),
            model,
            card,
            threads: self.threads,
            piece_s: self.piece_s,
            pad_ms: self.pad_ms,
            level_hz: self.level_hz,
            record_max_s: self.record_max_s,
            silence_stop_s: self.silence_stop.then_some(self.silence_stop_s),
        }
    }

    /// Where models run; Automatic for a name `check` would have refused.
    #[must_use]
    pub fn device(&self) -> DevicePick {
        DevicePick::parse(&self.device).unwrap_or(DevicePick::Auto)
    }

    /// Where the bar sits; above the keyboard for a name `check` would have refused.
    #[must_use]
    pub fn bar_place(&self) -> BarPlace {
        place_named(&self.place).unwrap_or(BarPlace::Above)
    }

    /// What a card's check must meet: `card_gain` in hundredths and `card_diff`.
    #[must_use]
    pub fn card_rules(&self) -> CardRules {
        let per_one = f64::from(Hundredths::PER_ONE);
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "card_gain is within CARD_GAIN, so its hundredths are a small positive whole number"
        )]
        let gain = (self.card_gain * per_one).round() as u32;
        CardRules {
            gain: Hundredths(gain),
            diff: self.card_diff,
        }
    }

    /// The wait within which a second click is ignored.
    #[must_use]
    pub fn debounce(&self) -> Duration {
        Duration::from_millis(self.debounce_ms.into())
    }

    /// The wait without speech before the nothing-heard hint.
    #[must_use]
    pub fn nothing(&self) -> Duration {
        Duration::from_secs(self.nothing_s.into())
    }

    /// How long the bar shows the text.
    #[must_use]
    pub fn done(&self) -> Duration {
        Duration::from_millis(self.done_ms.into())
    }

    /// The wave's smoothing.
    #[must_use]
    pub fn wave(&self) -> Duration {
        Duration::from_millis(self.wave_ms.into())
    }
}
