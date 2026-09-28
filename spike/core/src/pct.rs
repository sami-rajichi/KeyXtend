//! Nearest-rank percentiles, shared by the harness's latencies and the ring's frame gaps.

/// The typical value's rank.
pub const P50: f64 = 50.0;
/// The rank that budgets are checked against.
pub const P99: f64 = 99.0;
/// The whole range of ranks.
const ALL: f64 = 100.0;

/// Nearest-rank percentile `p` (0-100) of sorted values; `None` when empty.
pub fn percentile(sorted: &[f64], p: f64) -> Option<f64> {
    if sorted.is_empty() {
        return None;
    }
    let rank = (p / ALL * sorted.len() as f64).ceil() as usize;
    Some(sorted[rank.clamp(1, sorted.len()) - 1])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_rank_percentiles() {
        let v: Vec<f64> = (1..=100).map(f64::from).collect();
        assert_eq!(percentile(&v, P50), Some(50.0));
        assert_eq!(percentile(&v, 95.0), Some(95.0));
        assert_eq!(percentile(&v, P99), Some(99.0));
        assert_eq!(percentile(&v, 0.0), Some(1.0));
        assert_eq!(percentile(&[7.0], P99), Some(7.0));
        assert_eq!(percentile(&[], P50), None);
        assert_eq!(percentile(&[1.0, 2.0, 3.0, 4.0], P50), Some(2.0));
    }
}
