//! Times as counts of the job's samples: `SOUND_HZ` mono samples a second, or the source's own.

use super::mic::Format;
use kx_ipc::SOUND_HZ;

/// Milliseconds in a second.
const MS_PER_S: usize = 1000;
/// Seconds of source sound a run takes past its limit, so the resampler's last outputs still come.
const SLACK_S: u64 = 1;

/// The source samples a run limited to `record_max_s` can use at `format`, with `SLACK_S` to spare.
pub fn source(record_max_s: u16, format: Format) -> u64 {
    (u64::from(record_max_s) + SLACK_S)
        .saturating_mul(u64::from(format.rate))
        .saturating_mul(u64::from(format.channels))
}

/// Samples in `s` seconds.
pub fn secs(s: u16) -> usize {
    usize::from(s) * SOUND_HZ
}

/// Samples in `ms` milliseconds, rounded down.
pub fn millis(ms: u16) -> usize {
    usize::from(ms) * SOUND_HZ / MS_PER_S
}

/// How long `samples` last, in whole milliseconds.
pub fn heard_ms(samples: usize) -> u32 {
    let ms = samples.saturating_mul(MS_PER_S) / SOUND_HZ;
    u32::try_from(ms).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seconds_and_milliseconds_count_samples_both_ways() {
        assert_eq!(secs(2), 2 * SOUND_HZ);
        assert_eq!(millis(300), SOUND_HZ * 3 / 10);
        assert_eq!(heard_ms(secs(10)), 10_000);
        assert_eq!(heard_ms(millis(300) + 1), 300);
        assert_eq!(heard_ms(usize::MAX), u32::MAX);
    }

    #[test]
    fn a_run_may_use_its_limit_and_a_second_of_the_source_and_never_overflows() {
        let stereo = Format {
            rate: 48_000,
            channels: 2,
        };
        assert_eq!(source(10, stereo), 11 * 96_000);
        let widest = Format {
            rate: u32::MAX,
            channels: u16::MAX,
        };
        assert!(source(u16::MAX, widest) >= source(u16::MAX - 1, widest));
    }
}
