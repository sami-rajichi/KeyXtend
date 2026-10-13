//! Unit conversions shared by the synth, the importer and the writer: decibels, milliseconds and sample indexes.

/// Milliseconds per second.
pub(crate) const MS_PER_S: f64 = 1000.0;
/// Decibels per factor of ten in amplitude.
const DB_PER_DECADE: f64 = 20.0;
/// The amplitude base of the decibel scale.
const DECADE: f64 = 10.0;

/// The amplitude factor of `db` decibels.
pub(crate) fn db_gain(db: f64) -> f64 {
    DECADE.powf(db / DB_PER_DECADE)
}

/// The number of samples in `ms` milliseconds at `rate`; a negative or invalid time gives zero.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "Float-to-integer casts saturate, and sound lengths are far below usize::MAX."
)]
pub(crate) fn samples(ms: f64, rate: f64) -> usize {
    (ms * rate / MS_PER_S).round() as usize
}

/// The time, in ms, of sample `i` at `rate`.
pub(crate) fn ms_at(i: usize, rate: f64) -> f64 {
    index(i) / rate * MS_PER_S
}

/// Sample index `i` as a float.
#[allow(
    clippy::cast_precision_loss,
    reason = "Sample counts are far below 2^53."
)]
pub(crate) fn index(i: usize) -> f64 {
    i as f64
}

/// `x` rounded down to a sample index; a negative or invalid value gives zero.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "Float-to-integer casts saturate."
)]
pub(crate) fn floor_index(x: f64) -> usize {
    x.floor() as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decibels_map_to_amplitude() {
        assert!((db_gain(0.0) - 1.0).abs() < 1e-12);
        assert!((db_gain(-20.0) - 0.1).abs() < 1e-12);
    }

    #[test]
    fn milliseconds_map_to_samples_and_back() {
        assert_eq!(samples(400.0, 48_000.0), 19_200);
        assert_eq!(samples(-5.0, 48_000.0), 0);
        assert!((ms_at(480, 48_000.0) - 10.0).abs() < 1e-12);
        assert_eq!(floor_index(-0.5), 0);
        assert_eq!(floor_index(3.9), 3);
    }
}
