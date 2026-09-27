//! The status line every face shows under its keys, and when to retry the window guard.

/// Separator between the parts of the status line.
pub const SEP: &str = " · ";
/// Shown until the first guard runs.
pub const GUARD_PENDING: &str = "guard pending";
/// Words for uiAccess on and off.
const YES_NO: (&str, &str) = ("yes", "no");

/// `uiAccess: yes|no · keys: N`.
pub fn base(ui_access: bool, keys: usize) -> String {
    let access = if ui_access { YES_NO.0 } else { YES_NO.1 };
    format!("uiAccess: {access}{SEP}keys: {keys}")
}

/// `base · note`.
pub fn with(base: &str, note: &str) -> String {
    format!("{base}{SEP}{note}")
}

/// What to do after one guard try.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Guard {
    /// Our windows are guarded: hand the foreground back and show the note.
    Done(String),
    /// No window of ours is visible yet: try again after the guard delay.
    Retry,
    /// Out of tries or failed: still hand the foreground back, and show why.
    GiveUp(String),
}

/// The next step after try number `attempt` (from 1) of `tries`, given the guard's result.
pub fn guard(result: Result<usize, String>, attempt: u32, tries: u32) -> Guard {
    match result {
        Ok(0) if attempt < tries => Guard::Retry,
        Ok(0) => Guard::GiveUp(format!("guard failed: no window after {tries} tries")),
        Ok(n) => Guard::Done(format!("guarded: {n}")),
        Err(e) => Guard::GiveUp(format!("guard failed: {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_line_parts() {
        assert_eq!(base(true, 60), "uiAccess: yes · keys: 60");
        assert_eq!(base(false, 0), "uiAccess: no · keys: 0");
        assert_eq!(with("a", "b"), "a · b");
    }

    #[test]
    fn guard_retries_only_while_no_window_is_visible() {
        assert_eq!(guard(Ok(0), 1, 3), Guard::Retry);
        assert_eq!(
            guard(Ok(0), 3, 3),
            Guard::GiveUp("guard failed: no window after 3 tries".into())
        );
        assert_eq!(guard(Ok(2), 1, 3), Guard::Done("guarded: 2".into()));
        assert_eq!(
            guard(Err("x".into()), 1, 3),
            Guard::GiveUp("guard failed: x".into())
        );
    }
}
