//! Developer tool for workspace checks and release tasks.
#![forbid(unsafe_code)]
// Allow: xtask is a CLI, so reporting results on stdout is its job.
#![allow(clippy::print_stdout)]

mod cargo;
mod cli;
mod config;
mod dco;
mod devtools;
mod dist;
mod licences;
#[cfg(test)]
mod test_support;
mod text;
mod tidy;
mod workspace;

use std::ffi::OsString;
use std::fmt::Display;
use std::process::ExitCode;

fn main() -> ExitCode {
    let parsed = to_utf8(std::env::args_os().skip(1)).and_then(|args| cli::parse(&args));
    match parsed {
        Ok(command) => run(&command),
        Err(err) => {
            println!("{err}");
            ExitCode::from(cli::EXIT_USAGE_ERROR)
        }
    }
}

/// Converts OS argument strings to UTF-8, treating a non-Unicode argument as a
/// usage error instead of panicking (`std::env::args` panics on those).
fn to_utf8(args: impl Iterator<Item = OsString>) -> Result<Vec<String>, cli::UsageError> {
    args.map(|a| a.into_string().map_err(|_| cli::UsageError))
        .collect()
}

/// Dispatches a parsed command to its handler and maps the outcome to an exit code.
fn run(command: &cli::Command) -> ExitCode {
    match command {
        cli::Command::Tidy => run_tidy(),
        cli::Command::Dco { base, head } => run_dco(base, head),
        cli::Command::Licences => finish(licences::run()),
        cli::Command::Dist => finish(dist::run()),
        cli::Command::DevCert { remove } => finish(devtools::cert::run(*remove)),
        cli::Command::DevInstall { folder } => finish(devtools::install::run(folder)),
        cli::Command::DevUninstall { name } => finish(devtools::install::remove(name)),
        cli::Command::CheckUiAccess { exe } => {
            devtools::check::run(exe).map_or_else(fail, exit_for)
        }
    }
}

/// Prints `err` and returns the check-failed exit code.
fn fail(err: impl Display) -> ExitCode {
    println!("{err}");
    ExitCode::from(cli::EXIT_CHECK_FAILED)
}

/// The success exit code if `passed`, the check-failed one otherwise.
fn exit_for(passed: bool) -> ExitCode {
    ExitCode::from(if passed {
        cli::EXIT_SUCCESS
    } else {
        cli::EXIT_CHECK_FAILED
    })
}

/// Maps a subcommand result to an exit code, printing the error if there is one.
fn finish(result: Result<(), impl Display>) -> ExitCode {
    result.map_or_else(fail, |()| exit_for(true))
}

/// Runs `cargo xtask tidy`: loads the real workspace, runs every rule and prints the report.
///
/// It also validates the real dev-tools settings, so CI catches a bad value there.
fn run_tidy() -> ExitCode {
    let loaded = workspace::metadata_json().and_then(|json| {
        devtools::config::load(&json)?;
        workspace::load(&json)
    });
    match loaded {
        Ok(ws) => report_tidy(&tidy::run(&ws)),
        Err(err) => fail(err),
    }
}

/// Prints every violation and the summary line, then exits 0 only if [`tidy::passed`].
fn report_tidy(violations: &[tidy::Violation]) -> ExitCode {
    for violation in violations {
        println!("{}", tidy::render(violation));
    }
    println!("{}", tidy::summary(violations));
    exit_for(tidy::passed(violations))
}

/// Runs `cargo xtask dco <base> <head>`: reads the commit range via git and checks each commit.
fn run_dco(base: &str, head: &str) -> ExitCode {
    let ws = match workspace::from_cargo() {
        Ok(ws) => ws,
        Err(err) => return fail(err),
    };
    match dco::git::log(base, head) {
        Ok(commits) => report_dco(&dco::check(&commits, &ws.dco)),
        Err(err) => fail(err),
    }
}

/// Prints every failing commit and the summary line, then exits 0 only if [`dco::passed`].
fn report_dco(violations: &[dco::Violation]) -> ExitCode {
    for violation in violations {
        println!("{}", dco::render(violation));
    }
    println!("{}", dco::summary(violations));
    exit_for(dco::passed(violations))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_utf8_passes_through_valid_args() {
        let args = vec![OsString::from("tidy"), OsString::from("extra")];
        assert_eq!(
            to_utf8(args.into_iter()),
            Ok(vec!["tidy".to_string(), "extra".to_string()])
        );
    }

    #[cfg(windows)]
    #[test]
    fn to_utf8_maps_non_unicode_arg_to_usage_error() {
        use std::os::windows::ffi::OsStringExt;
        // 0xD800 is an unpaired surrogate: valid neither as UTF-16 nor UTF-8.
        let invalid = OsString::from_wide(&[0xD800]);
        assert_eq!(to_utf8(vec![invalid].into_iter()), Err(cli::UsageError));
    }

    #[test]
    fn exit_for_maps_pass_and_fail_to_their_codes() {
        assert_eq!(exit_for(true), ExitCode::from(cli::EXIT_SUCCESS));
        assert_eq!(exit_for(false), ExitCode::from(cli::EXIT_CHECK_FAILED));
    }

    #[test]
    fn finish_maps_ok_to_success_and_err_to_check_failed() {
        assert_eq!(
            finish(Ok::<(), &str>(())),
            ExitCode::from(cli::EXIT_SUCCESS)
        );
        assert_eq!(finish(Err("boom")), ExitCode::from(cli::EXIT_CHECK_FAILED));
    }
}
