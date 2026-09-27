//! The `licences` subcommand: runs cargo-deny's licence check, then lists licences.

use std::fmt;

/// One argument list to run as `cargo <args>`.
type Step = &'static [&'static str];

/// The licence-check steps, run in order. Each is `cargo`'s own subcommand args.
const STEPS: &[Step] = &[&["deny", "check", "licenses"], &["deny", "list"]];

/// Why `cargo xtask licences` failed.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum LicenceError {
    /// A step's process could not be started at all.
    Spawn {
        /// The step's display name: its arguments joined by spaces.
        step: String,
        /// Why the process could not start.
        reason: String,
    },
    /// A step ran but exited with a failure status.
    Failed {
        /// The step's display name: its arguments joined by spaces.
        step: String,
    },
}

impl fmt::Display for LicenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spawn { step, reason } => write!(f, "could not run `cargo {step}`: {reason}"),
            Self::Failed { step } => write!(f, "`cargo {step}` failed"),
        }
    }
}

/// A step's display name: its arguments joined by spaces.
fn step_name(step: Step) -> String {
    step.join(" ")
}

/// Runs every step in order with `run`, stopping at the first that fails or can't start.
///
/// `run` reports whether a step succeeded, or why it could not start, so tests never spawn a process.
pub(crate) fn run_steps(
    mut run: impl FnMut(&[&str]) -> Result<bool, String>,
) -> Result<(), LicenceError> {
    for step in STEPS {
        let name = step_name(step);
        match run(step) {
            Ok(true) => {}
            Ok(false) => return Err(LicenceError::Failed { step: name }),
            Err(reason) => return Err(LicenceError::Spawn { step: name, reason }),
        }
    }
    Ok(())
}

/// Spawns `cargo <args>` with inherited stdio, so the user sees cargo-deny's own output.
fn spawn_step(args: &[&str]) -> Result<bool, String> {
    crate::cargo::command()
        .args(args)
        .status()
        .map(|status| status.success())
        .map_err(|err| err.to_string())
}

/// Runs `cargo xtask licences`: cargo-deny's licence check, then its dependency list.
pub(crate) fn run() -> Result<(), LicenceError> {
    run_steps(spawn_step)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_steps_succeed_and_run_in_order() {
        let mut calls: Vec<String> = Vec::new();
        let result = run_steps(|args| {
            calls.push(args.join(" "));
            Ok(true)
        });
        assert_eq!(result, Ok(()));
        assert_eq!(calls, vec!["deny check licenses", "deny list"]);
    }

    #[test]
    fn first_step_failing_stops_the_run_before_the_second() {
        let mut calls: Vec<String> = Vec::new();
        let result = run_steps(|args| {
            calls.push(args.join(" "));
            Ok(false)
        });
        assert_eq!(
            result,
            Err(LicenceError::Failed {
                step: "deny check licenses".to_string()
            })
        );
        assert_eq!(calls, vec!["deny check licenses"]);
    }

    #[test]
    fn a_step_that_cannot_start_names_itself_in_the_error() {
        let result = run_steps(|_| Err("no such file".to_string()));
        assert_eq!(
            result,
            Err(LicenceError::Spawn {
                step: "deny check licenses".to_string(),
                reason: "no such file".to_string(),
            })
        );
    }
}
