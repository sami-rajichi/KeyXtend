//! When the window guard tries again: while no window of ours shows yet, up to `[ui] guard_tries` times.

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
    fn a_try_past_the_last_never_tries_again() {
        assert_eq!(next(&Ok(0), 4, 3), Next::Done);
        assert_eq!(next(&Ok(0), u32::MAX, 1), Next::Done);
    }
}
