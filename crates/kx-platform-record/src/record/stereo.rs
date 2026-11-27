//! A source's frames made into the track's: every channel downmixed to stereo by the table's gains, each side scaled
//! so it never clips, then resampled to the track's rate. It keeps its place across chunks.

use super::{CHANNELS, RATE};
use crate::table::Table;

/// A left and right gain.
type Gain = [f32; 2];

/// Turns one source's interleaved frames into stereo frames at the track's rate.
pub struct Stereo {
    gains: Vec<Gain>,
    turn: Resample,
    frame: [f32; 2],
}

impl Stereo {
    /// For a source of `channels` at `rate` Hz, with `table`'s downmix.
    #[must_use]
    pub fn new(channels: u16, rate: u32, table: &Table) -> Self {
        let listed = table.gains(channels).map(<[Gain]>::to_vec);
        Self {
            gains: scaled(listed.unwrap_or_else(|| spread(channels))),
            turn: Resample::new(rate, RATE),
            frame: [0.0; 2],
        }
    }

    /// The stereo frames, interleaved at the track's rate, that `input`'s whole frames make.
    pub fn push(&mut self, input: &[f32]) -> Vec<f32> {
        let mut out = Vec::new();
        for chunk in input.chunks_exact(self.gains.len().max(1)) {
            self.frame = [0.0; 2];
            for (s, g) in chunk.iter().zip(&self.gains) {
                self.frame[0] += s * g[0];
                self.frame[1] += s * g[1];
            }
            self.turn.push(self.frame, &mut out);
        }
        out
    }
}

/// The gains of a count the table does not list: the first channel left, the second right, the rest to both.
fn spread(channels: u16) -> Vec<Gain> {
    let side = |n: u16| match n {
        0 => [1.0, 0.0],
        1 => [0.0, 1.0],
        _ => [1.0, 1.0],
    };
    (0..channels.max(1)).map(side).collect()
}

/// `gains` with each side divided by its sum, so a full-scale frame on every channel stays full scale.
fn scaled(mut gains: Vec<Gain>) -> Vec<Gain> {
    for side in 0..usize::from(CHANNELS) {
        let sum: f32 = gains.iter().map(|g| g[side]).sum();
        if sum > 0.0 {
            for g in &mut gains {
                g[side] /= sum;
            }
        }
    }
    gains
}

/// A linear resampler over stereo frames, kept across chunks.
struct Resample {
    /// Input frames per output frame.
    step: f64,
    /// Where the next output frame falls after the last input frame, in input frames.
    at: f64,
    /// The last input frame, once one came.
    last: Option<[f32; 2]>,
}

impl Resample {
    /// From `from` Hz to `to` Hz.
    fn new(from: u32, to: u32) -> Self {
        Self {
            step: f64::from(from.max(1)) / f64::from(to.max(1)),
            at: 0.0,
            last: None,
        }
    }

    /// Takes one input frame and adds the output frames that fall between the last one and it.
    #[allow(
        clippy::cast_possible_truncation,
        reason = "A place between two frames is in 0 to 1."
    )]
    fn push(&mut self, frame: Gain, out: &mut Vec<f32>) {
        let Some(last) = self.last.replace(frame) else {
            return;
        };
        while self.at < 1.0 {
            let t = self.at as f32;
            out.extend(last.iter().zip(frame).map(|(a, b)| a + (b - a) * t));
            self.at += self.step;
        }
        self.at -= 1.0;
    }
}

#[cfg(test)]
mod tests;
