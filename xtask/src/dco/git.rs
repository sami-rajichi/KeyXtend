//! Runs `git log` for the DCO check and hands its output to [`super::parse_log`].

use std::fmt;
use std::path::Path;
use std::process::Command;

use super::{Commit, ParseError, parse_log};

/// The `git` executable name.
const GIT: &str = "git";
/// Git env vars that point git at another repo, ignoring its working folder.
const REPO_ENV: [&str; 3] = ["GIT_DIR", "GIT_INDEX_FILE", "GIT_WORK_TREE"];
/// `git log --pretty` format: hash, author name/email, parents, raw body, with fields
/// split by the ASCII unit separator (`\x1f`) and records ended by the record
/// separator (`\x1e`).
pub(crate) const LOG_FORMAT: &str = "%H%x1f%an%x1f%ae%x1f%P%x1f%B%x1e";

/// Why reading commits for the DCO check failed.
#[derive(Debug)]
pub(crate) enum LogError {
    /// The `git` process could not be started.
    Spawn(String),
    /// `git log` exited with an error; carries its stderr.
    Git(String),
    /// `git log`'s output could not be parsed.
    Parse(ParseError),
}

impl fmt::Display for LogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spawn(reason) => write!(f, "could not run git: {reason}"),
            Self::Git(stderr) => write!(f, "git log failed: {}", stderr.trim()),
            Self::Parse(err) => write!(f, "{err}"),
        }
    }
}

/// Lists commits in `base..head` in the current repo, parsed via [`super::parse_log`].
pub(crate) fn log(base: &str, head: &str) -> Result<Vec<Commit>, LogError> {
    log_in(base, head, None)
}

/// Like [`log`], but runs git in `dir` when given, instead of the current directory.
///
/// The `dir` parameter exists so tests can point git at a throwaway repo; `log` always
/// passes `None`, so `cargo xtask dco` still runs against the repo it's invoked in.
pub(crate) fn log_in(base: &str, head: &str, dir: Option<&Path>) -> Result<Vec<Commit>, LogError> {
    let range = format!("{base}..{head}");
    let output = git_command(dir)
        .args([
            "log",
            &format!("--pretty=format:{LOG_FORMAT}"),
            "--end-of-options",
            &range,
        ])
        .output()
        .map_err(|err| LogError::Spawn(err.to_string()))?;
    if !output.status.success() {
        return Err(LogError::Git(
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ));
    }
    parse_log(&String::from_utf8_lossy(&output.stdout)).map_err(LogError::Parse)
}

/// A `git` command, run in `dir` when given, with [`REPO_ENV`] removed so `dir` wins.
fn git_command(dir: Option<&Path>) -> Command {
    let mut command = Command::new(GIT);
    if let Some(dir) = dir {
        command.current_dir(dir);
        for var in REPO_ENV {
            command.env_remove(var);
        }
    }
    command
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;
    use std::fs;

    use super::*;
    use crate::dco::check;
    use crate::test_support::{dco_config, unique_temp_dir};

    #[test]
    fn spawn_error_names_the_reason() {
        assert_eq!(
            LogError::Spawn("no such file".to_string()).to_string(),
            "could not run git: no such file"
        );
    }

    #[test]
    fn git_error_includes_trimmed_stderr() {
        assert_eq!(
            LogError::Git("fatal: bad revision 'x'\n".to_string()).to_string(),
            "git log failed: fatal: bad revision 'x'"
        );
    }

    /// `LOG_FORMAT` must request these fields, in this order, or [`super::super::parse_record`]
    /// reads the wrong value into the wrong field. A regression here (e.g. a placeholder
    /// missing its `%`) would otherwise only surface as a real `git log` misreading commits.
    #[test]
    fn log_format_has_the_expected_placeholders_in_order() {
        let mut rest = LOG_FORMAT;
        for placeholder in ["%H", "%an", "%ae", "%P", "%B"] {
            let at = rest
                .find(placeholder)
                .unwrap_or_else(|| panic!("{placeholder} missing from LOG_FORMAT: {LOG_FORMAT}"));
            rest = &rest[at + placeholder.len()..];
        }
    }

    /// `GIT_DIR` and its friends override the working folder, so a run in `dir` must drop them.
    #[test]
    fn git_command_in_a_dir_removes_the_repo_env_vars() {
        let command = git_command(Some(Path::new(".")));
        let removed: Vec<&OsStr> = command
            .get_envs()
            .filter(|(_, value)| value.is_none())
            .map(|(key, _)| key)
            .collect();
        for var in ["GIT_DIR", "GIT_INDEX_FILE", "GIT_WORK_TREE"] {
            assert!(removed.contains(&OsStr::new(var)), "{var} not removed");
        }
    }

    #[test]
    fn git_command_without_a_dir_leaves_the_env_alone() {
        assert_eq!(git_command(None).get_envs().count(), 0);
    }

    /// Runs a git subcommand in `dir` with a fixed, throwaway identity, so the result
    /// never depends on the machine's own git config.
    fn run_git(dir: &Path, args: &[&str]) {
        const IDENTITY: [&str; 4] = [
            "-c",
            "user.name=Test Author",
            "-c",
            "user.email=test@example.com",
        ];
        /// A git fsmonitor daemon once hung a test commit for 15 minutes, so tests turn it off.
        const NO_FSMONITOR: [&str; 2] = ["-c", "core.fsmonitor=false"];
        let output = git_command(Some(dir))
            .args(IDENTITY)
            .args(NO_FSMONITOR)
            .args(["-c", "commit.gpgsign=false", "-c", "core.hooksPath="])
            .args(args)
            .output()
            .expect("git should run");
        assert!(
            output.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    /// End-to-end: a real git repo, read through the real [`log_in`], checked through the
    /// real [`crate::dco::check`]. Exercises `LOG_FORMAT` itself, not a hand-built record,
    /// so a broken format string (e.g. a missing `%`) fails this test.
    #[test]
    fn log_in_reads_a_real_repo_and_flags_only_the_unsigned_commit() {
        let dir = unique_temp_dir("log");
        fs::create_dir_all(&dir).expect("create temp dir");

        run_git(&dir, &["init", "-q"]);
        run_git(&dir, &["commit", "--allow-empty", "-q", "-m", "root"]);
        run_git(
            &dir,
            &["commit", "--allow-empty", "-q", "-m", "no sign-off here"],
        );
        run_git(
            &dir,
            &["commit", "--allow-empty", "-s", "-q", "-m", "signed commit"],
        );

        let result = log_in("HEAD~2", "HEAD", Some(&dir));

        fs::remove_dir_all(&dir).ok();

        let commits = result.expect("git log should succeed");
        assert_eq!(commits.len(), 2);

        let violations = check(&commits, &dco_config());

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].author_name, "Test Author");
        assert_eq!(violations[0].author_email, "test@example.com");
    }
}
