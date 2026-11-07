//! A seeded random source: the same seed always gives the same bytes, so ids and made passwords repeat in tests. It
//! runs `SplitMix64` (Steele, Lea and Flood, 2014), which spreads every byte value evenly; it is not for real secrets.

use crate::lock;
use kx_module_api::ports::{Random, RandomError};
use std::sync::{Arc, Mutex};

/// `SplitMix64`'s step, the golden ratio's fraction.
const GAMMA: u64 = 0x9e37_79b9_7f4a_7c15;
/// `SplitMix64`'s two mixing multipliers.
const MIX: [u64; 2] = [0xbf58_476d_1ce4_e5b9, 0x94d0_49bb_1331_11eb];
/// `SplitMix64`'s three mixing shifts.
const SHIFTS: [u32; 3] = [30, 27, 31];

/// The stream's state, and what the test set for it.
#[derive(Debug, Default)]
struct State {
    /// `SplitMix64`'s counter.
    seed: u64,
    /// The bytes of the last word not yet handed out.
    left: Vec<u8>,
    /// The next fill fails.
    fail: bool,
    /// How many fills were asked, failed ones too.
    fills: usize,
}

impl State {
    /// The next word of the stream.
    fn word(&mut self) -> u64 {
        self.seed = self.seed.wrapping_add(GAMMA);
        let mut z = self.seed;
        z = (z ^ (z >> SHIFTS[0])).wrapping_mul(MIX[0]);
        z = (z ^ (z >> SHIFTS[1])).wrapping_mul(MIX[1]);
        z ^ (z >> SHIFTS[2])
    }

    /// The next byte of the stream, from the last word's bytes first.
    fn byte(&mut self) -> u8 {
        if self.left.is_empty() {
            let word = self.word().to_le_bytes();
            self.left = word.iter().rev().copied().collect();
        }
        self.left.pop().unwrap_or_default()
    }
}

/// A random source shared by its clones: one seeded stream, read in any cuts, with one failure on cue.
#[derive(Clone, Debug)]
pub struct FakeRandom {
    state: Arc<Mutex<State>>,
}

impl FakeRandom {
    /// A source whose stream starts from `seed`.
    #[must_use]
    pub fn new(seed: u64) -> Self {
        let state = State {
            seed,
            ..State::default()
        };
        Self {
            state: Arc::new(Mutex::new(state)),
        }
    }

    /// Refuses the next fill, leaving its buffer and the stream as they were.
    pub fn fail_next(&self) {
        lock(&self.state).fail = true;
    }

    /// How many fills were asked, failed ones too.
    #[must_use]
    pub fn fills(&self) -> usize {
        lock(&self.state).fills
    }
}

impl Random for FakeRandom {
    fn fill(&self, buf: &mut [u8]) -> Result<(), RandomError> {
        let mut state = lock(&self.state);
        state.fills += 1;
        if std::mem::take(&mut state.fail) {
            return Err(RandomError);
        }
        for b in buf.iter_mut() {
            *b = state.byte();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
