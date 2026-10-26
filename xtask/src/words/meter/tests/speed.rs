//! The `--speed` verdict: a query kind fails only when its top percentile takes longer than the ceiling.

use super::super::speed::{over, passed};
use super::rules;
use crate::words::manifest::PERCENT;

/// The samples timed: one per percent, so the top percentile is that many places from the fastest.
fn samples() -> usize {
    usize::try_from(PERCENT).unwrap()
}

/// `fast` times at the ceiling, then the rest of the samples one microsecond past it.
fn times(fast: usize, ceiling: u64) -> Vec<u64> {
    let mut times = vec![ceiling; fast];
    times.resize(samples(), ceiling + 1);
    times
}

#[test]
fn a_kind_at_or_under_the_ceiling_passes_and_one_past_it_fails() {
    let rules = rules();
    let ceiling = rules.ceiling_us;
    for under in [ceiling, ceiling - 1] {
        let line = over("en", "one letter", &vec![under; samples()], &rules);
        assert_eq!(line, None, "{under} us");
    }
    let line = over("en", "one letter", &vec![ceiling + 1; samples()], &rules).unwrap();
    let parts = [
        "en",
        "one letter",
        &(ceiling + 1).to_string(),
        &ceiling.to_string(),
    ];
    assert!(parts.iter().all(|p| line.contains(p)), "{line}");
}

#[test]
fn the_verdict_reads_the_top_percentile_and_no_other() {
    let rules = rules();
    let (top, ceiling) = (rules.top(), rules.ceiling_us);
    let at = usize::try_from(top).unwrap();
    let slowest_over = times(at, ceiling);
    assert_eq!(
        over("en", "k", &slowest_over, &rules),
        None,
        "only past the top"
    );
    let top_over = times(at - 1, ceiling);
    let line = over("en", "k", &top_over, &rules).unwrap();
    assert!(line.contains(&format!("p{top} ")), "{line}");
}

#[test]
fn the_passed_line_names_the_top_percentile_and_the_ceiling() {
    let rules = rules();
    let line = passed(&rules);
    let top = format!("p{}", rules.top());
    assert!(
        line.contains(&top) && line.contains(&rules.ceiling_us.to_string()),
        "{line}"
    );
}
