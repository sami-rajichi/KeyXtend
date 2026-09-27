//! Subcommand parsing for the xtask CLI.

use std::fmt;

/// Exit code for a successful run.
pub(crate) const EXIT_SUCCESS: u8 = 0;
/// Exit code when a subcommand finds a problem.
pub(crate) const EXIT_CHECK_FAILED: u8 = 1;
/// Exit code for a usage error: unknown, missing or malformed subcommand.
pub(crate) const EXIT_USAGE_ERROR: u8 = 2;

const SUB_TIDY: &str = "tidy";
const SUB_DCO: &str = "dco";
const SUB_LICENCES: &str = "licences";
const SUB_DIST: &str = "dist";

/// Builds the usage line from the subcommand-name constants, so each name is
/// spelled exactly once.
fn usage_line() -> String {
    format!("usage: cargo xtask <{SUB_TIDY}|{SUB_DCO} <base> <head>|{SUB_LICENCES}|{SUB_DIST}>")
}

/// A parsed xtask subcommand.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Command {
    /// Run the workspace tidy checks.
    Tidy,
    /// Check DCO sign-offs between two git revisions.
    Dco {
        /// The base revision, exclusive.
        base: String,
        /// The head revision, inclusive.
        head: String,
    },
    /// Check third-party licences.
    Licences,
    /// Build release distribution artifacts.
    Dist,
}

/// A command-line usage error. Its `Display` is the usage line.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct UsageError;

impl fmt::Display for UsageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", usage_line())
    }
}

/// Parses xtask's arguments (excluding the program name) into a [`Command`].
pub(crate) fn parse(args: &[String]) -> Result<Command, UsageError> {
    match args {
        [name] if name == SUB_TIDY => Ok(Command::Tidy),
        [name] if name == SUB_LICENCES => Ok(Command::Licences),
        [name] if name == SUB_DIST => Ok(Command::Dist),
        [name, base, head] if name == SUB_DCO => Ok(Command::Dco {
            base: base.clone(),
            head: head.clone(),
        }),
        _ => Err(UsageError),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|v| (*v).to_string()).collect()
    }

    #[test]
    fn unknown_subcommand_is_a_usage_error() {
        assert_eq!(parse(&args(&["foo"])), Err(UsageError));
    }

    #[test]
    fn no_subcommand_is_a_usage_error() {
        assert_eq!(parse(&args(&[])), Err(UsageError));
    }

    #[test]
    fn tidy_parses() {
        assert_eq!(parse(&args(&["tidy"])), Ok(Command::Tidy));
    }

    #[test]
    fn licences_parses() {
        assert_eq!(parse(&args(&["licences"])), Ok(Command::Licences));
    }

    #[test]
    fn dist_parses() {
        assert_eq!(parse(&args(&["dist"])), Ok(Command::Dist));
    }

    #[test]
    fn dco_parses_with_base_and_head() {
        assert_eq!(
            parse(&args(&["dco", "main", "HEAD"])),
            Ok(Command::Dco {
                base: "main".into(),
                head: "HEAD".into(),
            })
        );
    }

    #[test]
    fn dco_with_zero_arguments_is_a_usage_error() {
        assert_eq!(parse(&args(&["dco"])), Err(UsageError));
    }

    #[test]
    fn dco_with_one_argument_is_a_usage_error() {
        assert_eq!(parse(&args(&["dco", "main"])), Err(UsageError));
    }

    #[test]
    fn dco_with_three_arguments_is_a_usage_error() {
        assert_eq!(
            parse(&args(&["dco", "main", "HEAD", "extra"])),
            Err(UsageError)
        );
    }

    #[test]
    fn usage_error_message_mentions_every_subcommand() {
        let message = UsageError.to_string();
        for name in [SUB_TIDY, SUB_DCO, SUB_LICENCES, SUB_DIST] {
            assert!(message.contains(name), "usage message missing {name}");
        }
    }
}
