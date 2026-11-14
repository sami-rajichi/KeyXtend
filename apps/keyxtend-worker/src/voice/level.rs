//! The sound level the bar's wave follows: the loudest sample of each window, `level_hz` windows a second of sound,
//! from 0 for silence to `u8::MAX` at full scale. Windows are counted in samples, so the pace follows the sound itself.

use kx_ipc::SOUND_HZ;

/// Measures 16 kHz samples into levels.
pub struct Meter {
    hz: u64,
    /// Samples measured.
    done: u64,
    /// Where the current window ends, in samples.
    edge: u64,
    /// Windows told.
    told: u64,
    /// The loudest sample of the current window.
    peak: u16,
}

impl Meter {
    /// A meter telling `level_hz` levels a second, at least 1.
    pub fn new(level_hz: u8) -> Self {
        let mut meter = Self {
            hz: u64::from(level_hz.max(1)),
            done: 0,
            edge: 0,
            told: 0,
            peak: 0,
        };
        meter.edge = meter.window_end();
        meter
    }

    /// Measures `x`, adding to `levels` the level of each window it ends.
    pub fn take(&mut self, x: &[i16], levels: &mut Vec<u8>) {
        for &s in x {
            self.peak = self.peak.max(s.unsigned_abs());
            self.done += 1;
            if self.done >= self.edge {
                levels.push(level(self.peak));
                self.peak = 0;
                self.told += 1;
                self.edge = self.window_end();
            }
        }
    }

    /// Where the window after the ones told ends: the next whole share of a second, rounded up.
    fn window_end(&self) -> u64 {
        ((self.told + 1) * SOUND_HZ as u64).div_ceil(self.hz)
    }
}

/// A peak as a level: full scale is `u8::MAX`.
fn level(peak: u16) -> u8 {
    let full = u32::from(i16::MAX.unsigned_abs());
    let scaled = (u32::from(peak) * u32::from(u8::MAX) + full / 2) / full;
    u8::try_from(scaled).unwrap_or(u8::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kx_ipc::LEVEL_HZ;

    /// The levels of `x` fed in chunks of `chunk` samples.
    fn levels(hz: u8, x: &[i16], chunk: usize) -> Vec<u8> {
        let (mut m, mut out) = (Meter::new(hz), Vec::new());
        x.chunks(chunk).for_each(|c| m.take(c, &mut out));
        out
    }

    #[test]
    fn each_second_of_sound_gives_level_hz_levels_however_it_is_chunked() {
        let sound = vec![1000_i16; 3 * SOUND_HZ];
        for hz in LEVEL_HZ {
            for chunk in [1, 160, 441, SOUND_HZ] {
                assert_eq!(
                    levels(hz, &sound, chunk).len(),
                    3 * usize::from(hz),
                    "{hz} {chunk}"
                );
            }
        }
    }

    #[test]
    fn silence_is_zero_full_scale_is_the_top_and_half_is_half() {
        let n = SOUND_HZ;
        assert!(levels(10, &vec![0; n], n).iter().all(|&l| l == 0));
        assert!(
            levels(10, &vec![i16::MIN; n], n)
                .iter()
                .all(|&l| l == u8::MAX)
        );
        assert!(levels(10, &vec![1 << 14; n], n).iter().all(|&l| l == 128));
    }

    #[test]
    fn a_window_takes_its_own_peak_only() {
        let window = SOUND_HZ / 10;
        let mut sound = vec![0_i16; 2 * window];
        sound[window - 1] = i16::MAX;
        assert_eq!(levels(10, &sound, 7), [u8::MAX, 0]);
    }
}
