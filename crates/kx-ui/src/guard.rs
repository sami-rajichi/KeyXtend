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

/// A guard run's tries: each step makes try number 1, 2, … and a try that says stop ends the run, so the next
/// run starts from 1 again.
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
    /// Makes the next try with `try_once(attempt)`; true to try again after `[ui] guard_delay_ms`.
    pub fn step(&mut self, try_once: impl FnOnce(u32) -> bool) -> bool {
        let attempt = self.tries.saturating_add(1);
        let again = try_once(attempt);
        self.tries = if again { attempt } else { 0 };
        again
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            run.step(|attempt| {
                seen.push(attempt);
                again
            });
        }
        assert_eq!(seen, [1, 2, 3, 1], "a run ends when a try says stop");
        assert!(run.step(|_| true));
    }

    #[test]
    fn a_try_past_the_last_never_tries_again() {
        assert_eq!(next(&Ok(0), 4, 3), Next::Done);
        assert_eq!(next(&Ok(0), u32::MAX, 1), Next::Done);
    }
}
