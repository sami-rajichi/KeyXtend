//! The pacing on fake points: a burst is told once after it settles, the cap defers a tell to the window's end with the
//! newest change winning, and idle owes nothing; property tests run any changes the way the thread does.

use super::*;
use kx_module_api::US_PER_MS;
use proptest::prelude::*;

/// The test pacing: a change settles in `SETTLE_MS`, and `CAP` tells fit in a window.
const SETTLE_MS: u32 = 100;
const CAP: u16 = 3;
/// A cap no test run reaches.
const NO_CAP: u16 = u16::MAX;
/// A `read_ms`, which pacing never reads.
const READ_MS: u32 = 250;
/// The window, in ms.
const WINDOW_MS: u64 = 1_000;
/// The most changes in a property run.
const STEPS: usize = 40;

/// Pacing with `settle_ms` and `looks_per_s`.
fn paced(settle_ms: u32, looks_per_s: u16) -> Pace {
    Pace::new(SelConfig {
        settle_ms,
        read_ms: READ_MS,
        looks_per_s,
    })
}

/// The point `ms` milliseconds after the start.
fn at(ms: u64) -> Mono {
    Mono::from_us(ms * u64::from(US_PER_MS))
}

/// Runs the pace as the thread does up to `now`: the tell due by then, told at its due point, which it returns.
fn tick(pace: &mut Pace, now: Mono) -> Option<Mono> {
    let due = pace.due().filter(|&due| due <= now)?;
    assert!(pace.tell(due), "a due tell is told");
    Some(due)
}

/// What a driven pace saw, in order.
#[derive(Copy, Clone, Debug)]
enum Seen {
    Change(Mono),
    Tell(Mono),
}

/// Feeds changes `gaps` ms apart, telling as the thread would between them, then runs until idle: what it saw.
fn drive(pace: &mut Pace, gaps: &[u64]) -> Vec<Seen> {
    let (mut now, mut seen) = (0, Vec::new());
    for gap in gaps {
        now += gap;
        seen.extend(tick(pace, at(now)).map(Seen::Tell));
        pace.change(at(now));
        seen.push(Seen::Change(at(now)));
    }
    seen.extend(tick(pace, Mono::from_us(u64::MAX)).map(Seen::Tell));
    seen
}

/// The change points and the tell points of `seen`.
fn split(seen: &[Seen]) -> (Vec<Mono>, Vec<Mono>) {
    let (mut changes, mut tells) = (Vec::new(), Vec::new());
    for s in seen {
        match *s {
            Seen::Change(c) => changes.push(c),
            Seen::Tell(t) => tells.push(t),
        }
    }
    (changes, tells)
}

/// Checks each tell falls exactly where the pacing puts it: the later of the newest change settled and one window
/// after the tell `cap` places back.
fn exact(seen: &[Seen], settle: Duration, cap: usize) -> Result<(), TestCaseError> {
    let (mut newest, mut told) = (None, Vec::new());
    for s in seen {
        match *s {
            Seen::Change(c) => newest = Some(c),
            Seen::Tell(t) => {
                let settled = newest.map(|c: Mono| c.saturating_add(settle));
                let back = told.len().checked_sub(cap).and_then(|k| told.get(k));
                let free = back.map(|u: &Mono| u.saturating_add(WINDOW));
                prop_assert_eq!(Some(t), settled.max(free), "the tell at {:?}", t);
                told.push(t);
            }
        }
    }
    Ok(())
}

#[test]
fn idle_owes_nothing_and_never_tells() {
    let mut pace = paced(SETTLE_MS, CAP);
    assert_eq!(pace.due(), None);
    assert!(!pace.tell(at(WINDOW_MS)));
    assert_eq!(pace.due(), None, "still idle");
}

#[test]
fn a_burst_is_told_once_settle_after_its_last_change() {
    let mut pace = paced(SETTLE_MS, CAP);
    for ms in [0, 40, 80] {
        pace.change(at(ms));
    }
    let due = at(80 + u64::from(SETTLE_MS));
    assert_eq!(pace.due(), Some(due));
    assert!(!pace.tell(at(179)), "not settled yet");
    assert!(pace.tell(due));
    assert_eq!(pace.due(), None, "told once");
    assert!(!pace.tell(at(10 * WINDOW_MS)));
}

#[test]
fn a_tell_past_the_cap_waits_for_the_windows_end_and_the_newest_change_wins() {
    let mut pace = paced(SETTLE_MS, CAP);
    let mut first = None;
    for ms in [0, 200, 400] {
        pace.change(at(ms));
        let told = tick(&mut pace, at(ms + u64::from(SETTLE_MS)));
        assert!(told.is_some(), "inside the cap");
        first = first.or(told);
    }
    let end = first.map(|t| t.saturating_add(WINDOW));
    pace.change(at(600));
    assert_eq!(pace.due(), end, "the window's end, not 700 ms");
    pace.change(at(900));
    assert_eq!(pace.due(), end, "a newer change waits as well");
    pace.change(at(1_050));
    assert_eq!(
        pace.due(),
        Some(at(1_150)),
        "settling ends after the window"
    );
    assert_eq!(tick(&mut pace, Mono::from_us(u64::MAX)), Some(at(1_150)));
    assert_eq!(pace.due(), None, "one tell for all three");
}

#[test]
fn a_cap_of_zero_still_tells_once_a_window() {
    let mut pace = paced(0, 0);
    pace.change(at(0));
    assert!(pace.tell(at(0)));
    pace.change(at(1));
    assert_eq!(pace.due(), Some(at(WINDOW_MS)));
}

#[test]
fn an_earlier_point_never_moves_the_newest_change_back() {
    let mut pace = paced(SETTLE_MS, CAP);
    pace.change(at(500));
    pace.change(at(100));
    assert_eq!(pace.due(), Some(at(500 + u64::from(SETTLE_MS))));
}

/// Gaps between changes, in ms, up to `most`.
fn gaps(most: u64) -> impl Strategy<Value = Vec<u64>> {
    prop::collection::vec(0..=most, 0..STEPS)
}

proptest! {
    #[test]
    fn any_burst_is_told_exactly_once_settle_after_its_last_change(
        first in 0..=WINDOW_MS,
        rest in prop::collection::vec(0..u64::from(SETTLE_MS), 0..STEPS),
    ) {
        let mut pace = paced(SETTLE_MS, CAP);
        let gaps: Vec<u64> = std::iter::once(first).chain(rest).collect();
        let (changes, tells) = split(&drive(&mut pace, &gaps));
        let last = changes.last().map(|c| c.saturating_add(pace.settle));
        prop_assert_eq!(tells, last.into_iter().collect::<Vec<_>>());
    }

    #[test]
    fn with_no_cap_each_burst_is_told_once_after_it_settles(gaps in gaps(3 * u64::from(SETTLE_MS))) {
        let mut pace = paced(SETTLE_MS, NO_CAP);
        let (changes, tells) = split(&drive(&mut pace, &gaps));
        let settle = u64::from(SETTLE_MS);
        let ends = changes.iter().enumerate().filter(|&(i, _)| gaps.get(i + 1).is_none_or(|&g| g >= settle));
        let want: Vec<Mono> = ends.map(|(_, c)| c.saturating_add(pace.settle)).collect();
        prop_assert_eq!(tells, want);
    }

    #[test]
    fn each_tell_falls_where_settle_and_cap_put_it_and_no_window_holds_more(
        settle_ms in 0..=2 * SETTLE_MS,
        cap in 1..=CAP + 2,
        gaps in gaps(3 * WINDOW_MS / 2),
    ) {
        let mut pace = paced(settle_ms, cap);
        let seen = drive(&mut pace, &gaps);
        exact(&seen, pace.settle, usize::from(cap))?;
        let (_, tells) = split(&seen);
        for &t in &tells {
            let inside = tells.iter().filter(|&&u| u <= t && t.saturating_since(u) < WINDOW).count();
            prop_assert!(inside <= usize::from(cap), "{inside} tells in the window ending {t:?}");
        }
        prop_assert!(tells.is_sorted(), "told in order");
        prop_assert_eq!(pace.due(), None, "every change told, then idle: no wake-up");
    }
}
