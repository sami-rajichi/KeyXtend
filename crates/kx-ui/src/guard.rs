//! When the window guard tries again: while no window of ours shows yet, up to `[ui] guard_tries` times per run.

use kx_platform::ui::UiError;

/// What to do after one guard try.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Next {
    /// No window of ours shows yet: try again after `[ui] guard_delay_ms`.
    Again,
    /// Guarded, failed or out of tries: hand the focus back and stop.
    Done,
}

/// The step after try number `attempt` (from 1) of `tries`, given how many windows it guarded or why it failed.
#[must_use]
pub fn next(result: &Result<usize, UiError>, attempt: u32, tries: u32) -> Next {
    match result {
        Ok(0) if attempt < tries => Next::Again,
        _ => Next::Done,
    }
}

/// A guard run's tries: try number 1, 2, … and a try that says stop ends the run, so the next run starts from 1
/// again. The caller makes the try between `next` and `settle`, holding nothing of the run.
#[derive(Debug, Default)]
#[cfg_attr(
    not(feature = "qt"),
    allow(dead_code, reason = "only the Qt Keyboard object runs the guard")
)]
pub struct Run {
    tries: u32,
}

#[cfg_attr(
    not(feature = "qt"),
    allow(dead_code, reason = "only the Qt Keyboard object runs the guard")
)]
impl Run {
    /// The number of the next try.
    #[must_use]
    pub fn next(&self) -> u32 {
        self.tries.saturating_add(1)
    }

    /// Records try `attempt`: `again` keeps the run going, else the next run starts from 1.
    pub fn settle(&mut self, attempt: u32, again: bool) {
        self.tries = if again { attempt } else { 0 };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::{GUARD_TRIES, UiSettings};
    use proptest::collection::vec;
    use proptest::prelude::*;

    #[test]
    fn the_guard_tries_again_only_while_no_window_shows() {
        assert_eq!(next(&Ok(0), 1, 3), Next::Again);
        assert_eq!(next(&Ok(0), 3, 3), Next::Done, "out of tries");
        assert_eq!(next(&Ok(2), 1, 3), Next::Done, "guarded");
        assert_eq!(next(&Err(UiError::Refused(5)), 1, 3), Next::Done);
    }

    #[test]
    fn a_run_counts_its_tries_and_the_next_run_starts_from_one() {
        let mut run = Run::default();
        let mut seen = Vec::new();
        for again in [true, true, false, false] {
            let attempt = run.next();
            seen.push(attempt);
            run.settle(attempt, again);
        }
        assert_eq!(seen, [1, 2, 3, 1], "a run ends when a try says stop");
    }

    #[test]
    fn naming_the_next_try_changes_nothing_until_it_settles() {
        let mut run = Run::default();
        assert_eq!((run.next(), run.next()), (1, 1));
        run.settle(1, true);
        assert_eq!(run.next(), 2);
    }

    #[test]
    fn a_try_past_the_last_never_tries_again() {
        assert_eq!(next(&Ok(0), 4, 3), Next::Done);
        assert_eq!(next(&Ok(0), u32::MAX, 1), Next::Done);
    }

    /// Any allowed `[ui] guard_tries`, often no more than the default, so runs reach their last try.
    fn tries() -> impl Strategy<Value = u32> {
        let most = UiSettings::from_defaults().unwrap().guard_tries;
        prop_oneof![*GUARD_TRIES.start()..=most, GUARD_TRIES]
    }

    /// What one try finds: mostly no window yet, else windows guarded or a refusal.
    fn found() -> impl Strategy<Value = Result<usize, UiError>> {
        prop_oneof![
            4 => Just(Ok(0)),
            1 => (1..=usize::MAX).prop_map(Ok),
            1 => any::<i32>().prop_map(|code| Err(UiError::Refused(code))),
            1 => Just(Err(UiError::NoWindow)),
        ]
    }

    proptest! {
        #[test]
        fn a_run_counts_from_one_never_passes_its_last_try_and_ends_once_a_window_shows(
            tries in tries(),
            found in vec(found(), 0..60),
        ) {
            let (mut run, mut since) = (Run::default(), 0);
            for result in found {
                let attempt = run.next();
                prop_assert_eq!(attempt, since + 1, "a run counts its tries from 1");
                prop_assert!(attempt <= tries, "try {} of {}", attempt, tries);
                let again = next(&result, attempt, tries) == Next::Again;
                if matches!(result, Ok(0)) {
                    prop_assert_eq!(again, attempt < tries, "no window yet tries again until the last try");
                } else {
                    prop_assert!(!again, "a guarded or failed try ends the run");
                }
                run.settle(attempt, again);
                since = if again { attempt } else { 0 };
            }
        }
    }
}
