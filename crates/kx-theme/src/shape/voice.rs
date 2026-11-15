//! The dictation bar's and the Voice panel's own sizes (design/voice-bar.html `--voice-*`); the bar's status sizes
//! live in `[caption]` and the panel's frame in `[panel]` and `[clip]`.

use serde::{Deserialize, Serialize};

/// The bar and the panel's own parts; both grow with the keyboard.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VoiceShape {
    /// The bar's width: while it listens or writes, then with a text or a note.
    pub bar_px: [f32; 2],
    /// The wave's bars: width, gap, height at rest and at full level.
    pub wave_px: [f32; 4],
    /// The lines a text or a note shows before it is cut.
    pub text_lines: u8,
    /// A download's progress bar height.
    pub progress_px: f32,
    /// An option's switch: width, height and knob.
    pub switch_px: [f32; 3],
    /// A stepper's value box width.
    pub value_px: f32,
    /// A number column of the Compare table.
    pub col_px: f32,
}

impl VoiceShape {
    /// Refuses, naming the size: no line, a size of 0 or less, a wave gap below 0, a wide bar narrower than the other,
    /// a wave whose rest is above its full height, or a knob larger than its switch.
    pub(super) fn check(&self) -> Result<(), String> {
        if self.text_lines == 0 {
            return Err("shape: voice.text_lines 0 must be 1 or more".to_owned());
        }
        let [w, gap, rest, full] = self.wave_px;
        let above = self.bar_px.iter().map(|v| ("voice.bar_px", *v));
        let above = above.chain(self.switch_px.iter().map(|v| ("voice.switch_px", *v)));
        let mut above = above.chain([
            ("voice.wave_px", w),
            ("voice.wave_px", rest),
            ("voice.wave_px", full),
            ("voice.progress_px", self.progress_px),
            ("voice.value_px", self.value_px),
            ("voice.col_px", self.col_px),
        ]);
        if let Some((n, v)) = above.find(|(_, v)| !(v.is_finite() && *v > 0.0)) {
            return Err(format!("shape: {n} {v} must be above 0"));
        }
        if !(gap.is_finite() && gap >= 0.0) {
            return Err(format!("shape: voice.wave_px gap {gap} must be 0 or more"));
        }
        self.check_order(rest, full)
    }

    /// The wide bar holds the narrow one, the wave rests below its full height, and the knob fits in its switch.
    fn check_order(&self, rest: f32, full: f32) -> Result<(), String> {
        let [narrow, wide] = self.bar_px;
        let [_, high, knob] = self.switch_px;
        let wrong = [
            (
                "voice.bar_px",
                wide < narrow,
                "the text's bar must be as wide as the other or wider",
            ),
            (
                "voice.wave_px",
                full < rest,
                "the full height must be the rest's or more",
            ),
            (
                "voice.switch_px",
                knob > high,
                "the knob must fit in its switch",
            ),
        ];
        match wrong.iter().find(|(_, bad, _)| *bad) {
            Some((n, _, why)) => Err(format!("shape: {n}: {why}")),
            None => Ok(()),
        }
    }
}
