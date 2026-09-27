//! Click-to-character matching and percentiles for G2.

use crate::tlog::Unit;

/// Microseconds per millisecond.
pub const US_PER_MS: f64 = 1000.0;
/// The rank that budgets are checked against.
pub const P99: f64 = 99.0;
/// Percentiles reported by G2 and G5: name in the results, and rank (0-100).
pub const PERCENTILES: [(&str, f64); 3] = [("p50", 50.0), ("p95", 95.0), ("p99", P99)];

/// One injected click and the UTF-16 units it should produce.
#[derive(Debug, Clone)]
pub struct Click {
    /// Scan code of the clicked key.
    pub code: u32,
    /// QPC time of the click in microseconds.
    pub us: i64,
    /// Expected UTF-16 units.
    pub expect: Vec<u16>,
}

/// Nearest-rank percentile `p` (0-100) of sorted values; `None` when empty.
pub fn percentile(sorted: &[f64], p: f64) -> Option<f64> {
    if sorted.is_empty() {
        return None;
    }
    let rank = (p / 100.0 * sorted.len() as f64).ceil() as usize;
    Some(sorted[rank.clamp(1, sorted.len()) - 1])
}

/// Latency in µs per click: the first matching units, in order, logged within `window_us` after it.
///
/// The search stops at a later click that could type the same units, so a lost key never takes its character.
pub fn latencies(clicks: &[Click], log: &[Unit], window_us: i64) -> Vec<Option<i64>> {
    let mut next = 0;
    clicks
        .iter()
        .enumerate()
        .map(|(i, click)| {
            let found = find(click, log, next, stop_us(clicks, i, window_us))?;
            next = found + click.expect.len();
            Some(log[found].us - click.us)
        })
        .collect()
}

/// When the search for click `i` ends: after its window, or at a later click that could type its units.
fn stop_us(clicks: &[Click], i: usize, window_us: i64) -> i64 {
    let (click, end) = (&clicks[i], clicks[i].us + window_us);
    let want = click.expect.as_slice();
    let could_type = |c: &&Click| c.expect.windows(want.len().max(1)).any(|w| w == want);
    clicks[i + 1..]
        .iter()
        .find(could_type)
        .map_or(end, |c| c.us.min(end))
}

/// Index of the first units that match `click`, logged from the click until before `end_us`.
fn find(click: &Click, log: &[Unit], from: usize, end_us: i64) -> Option<usize> {
    let k = click.expect.len();
    if k == 0 {
        return None;
    }
    (from..log.len().saturating_sub(k - 1))
        .take_while(|&j| log[j].us < end_us)
        .find(|&j| {
            log[j].us >= click.us
                && log[j..j + k]
                    .iter()
                    .map(|u| u.unit)
                    .eq(click.expect.iter().copied())
        })
}

/// The `PERCENTILES` of the matched latencies, in ms.
pub fn summary_ms(latencies: &[Option<i64>]) -> [(&'static str, Option<f64>); PERCENTILES.len()] {
    let mut ms: Vec<f64> = latencies
        .iter()
        .flatten()
        .map(|&us| us as f64 / US_PER_MS)
        .collect();
    ms.sort_by(f64::total_cmp);
    PERCENTILES.map(|(name, p)| (name, percentile(&ms, p)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit(us: i64, unit: u16) -> Unit {
        Unit { us, unit }
    }

    fn click(us: i64, expect: &[u16]) -> Click {
        Click {
            code: 0,
            us,
            expect: expect.to_vec(),
        }
    }

    #[test]
    fn nearest_rank_percentiles() {
        let v: Vec<f64> = (1..=100).map(f64::from).collect();
        assert_eq!(percentile(&v, 50.0), Some(50.0));
        assert_eq!(percentile(&v, 95.0), Some(95.0));
        assert_eq!(percentile(&v, 99.0), Some(99.0));
        assert_eq!(percentile(&v, 0.0), Some(1.0));
        assert_eq!(percentile(&[7.0], 99.0), Some(7.0));
        assert_eq!(percentile(&[], 50.0), None);
        assert_eq!(percentile(&[1.0, 2.0, 3.0, 4.0], 50.0), Some(2.0));
    }

    #[test]
    fn matches_clicks_in_order_and_skips_a_lost_character() {
        let clicks = [
            click(100, &[0x61]),
            click(5_000, &[0x62]),
            click(10_000, &[0x0644, 0x0627]),
            click(20_000, &[0x62]),
        ];
        let log = [
            unit(130, 0x61),
            unit(10_050, 0x0644),
            unit(10_051, 0x0627),
            unit(20_200, 0x62),
        ];
        let got = latencies(&clicks, &log, 1_000);
        assert_eq!(got, vec![Some(30), None, Some(50), Some(200)]);
        let want = [("p50", Some(0.05)), ("p95", Some(0.2)), ("p99", Some(0.2))];
        assert_eq!(summary_ms(&got), want);
    }

    #[test]
    fn a_lost_key_does_not_take_the_character_of_a_later_repeat() {
        let clicks = [click(100, &[0x61]), click(400, &[0x61])];
        assert_eq!(
            latencies(&clicks, &[unit(430, 0x61)], 1_000),
            vec![None, Some(30)]
        );
        let clicks = [click(0, &[0x61]), click(100, &[0x62]), click(200, &[0x61])];
        let log = [unit(110, 0x62), unit(210, 0x61)];
        assert_eq!(
            latencies(&clicks, &log, 1_000),
            vec![None, Some(10), Some(10)]
        );
        let clicks = [click(0, &[0x0644]), click(100, &[0x0644, 0x0627])];
        let log = [unit(110, 0x0644), unit(111, 0x0627)];
        assert_eq!(latencies(&clicks, &log, 1_000), vec![None, Some(10)]);
    }
}
