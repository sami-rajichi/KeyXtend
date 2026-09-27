//! The `dist` subcommand: a stub until the installer build lands in P14.

use std::fmt;

/// The milestone that will implement `cargo xtask dist`.
const DIST_MILESTONE: &str = "P14";

/// Why `cargo xtask dist` refuses to run: distribution builds aren't built yet.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct NotBuiltYet;

impl fmt::Display for NotBuiltYet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "installer build arrives in {DIST_MILESTONE}")
    }
}

/// Always fails: `cargo xtask dist` has no build to run until [`DIST_MILESTONE`].
pub(crate) fn run() -> Result<(), NotBuiltYet> {
    Err(NotBuiltYet)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dist_returns_the_p14_error() {
        assert_eq!(run(), Err(NotBuiltYet));
    }

    #[test]
    fn error_message_names_the_milestone() {
        assert_eq!(NotBuiltYet.to_string(), "installer build arrives in P14");
    }
}
