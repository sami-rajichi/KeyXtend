//! Runs the dev-tools PowerShell scripts and maps their exit codes.

use std::path::Path;
use std::process::Command;

use super::config::DevSetup;
use super::{DevError, paths};

/// PowerShell flags for every script run: no profile, no prompts, script policy for this run only.
const FLAGS: [&str; 5] = [
    "-NoProfile",
    "-NonInteractive",
    "-ExecutionPolicy",
    "Bypass",
    "-File",
];

/// Arguments that run `script` with `params`, each as `-Name value`.
pub(crate) fn script_args(script: &Path, params: &[(&str, String)]) -> Vec<String> {
    let mut args: Vec<String> = FLAGS.iter().map(ToString::to_string).collect();
    args.push(paths::text(script));
    for (name, value) in params {
        args.push(format!("-{name}"));
        args.push(value.clone());
    }
    args
}

/// Maps a finished step to its stdout, or to why it failed; only a step that can show a prompt has a `cancel` code.
pub(crate) fn outcome(
    step: &str,
    code: Option<i32>,
    stdout: &str,
    stderr: &str,
    cancel: Option<i32>,
) -> Result<String, DevError> {
    match code {
        Some(0) => Ok(stdout.to_string()),
        Some(_) if code == cancel => Err(DevError::Cancelled),
        _ => Err(DevError::Failed {
            step: step.to_string(),
            detail: first_line(stderr),
        }),
    }
}

/// The first non-blank line of `text`, trimmed.
fn first_line(text: &str) -> String {
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or_default()
        .to_string()
}

/// Runs a tool that never shows a prompt, and maps the result with [`outcome`].
pub(crate) fn run(step: &str, program: &str, args: &[String]) -> Result<String, DevError> {
    exec(step, program, args, None)
}

/// Runs `program` with `args` and maps the result with [`outcome`].
fn exec(
    step: &str,
    program: &str,
    args: &[String],
    cancel: Option<i32>,
) -> Result<String, DevError> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|err| DevError::Spawn {
            program: program.to_string(),
            reason: err.to_string(),
        })?;
    outcome(
        step,
        output.status.code(),
        &String::from_utf8_lossy(&output.stdout),
        &String::from_utf8_lossy(&output.stderr),
        cancel,
    )
}

/// Runs a script from the configured scripts folder with the Windows PowerShell.
pub(crate) fn run_script(
    setup: &DevSetup,
    script: &str,
    params: &[(&str, String)],
    cancel: Option<i32>,
) -> Result<String, DevError> {
    let shell = setup.powershell(|key| std::env::var(key).ok())?;
    let args = script_args(&setup.script(script), params);
    exec(script, &paths::text(&shell), &args, cancel)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CANCEL: Option<i32> = Some(1223);

    #[test]
    fn script_args_put_flags_then_script_then_params() {
        let args = script_args(
            Path::new("D:\\ws\\x.ps1"),
            &[("Action", "Find".to_string()), ("Days", "90".to_string())],
        );
        assert_eq!(
            args,
            vec![
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
                "D:\\ws\\x.ps1",
                "-Action",
                "Find",
                "-Days",
                "90",
            ]
        );
    }

    #[test]
    fn exit_zero_returns_stdout() {
        assert_eq!(
            outcome("s", Some(0), "ABC\n", "", CANCEL),
            Ok("ABC\n".to_string())
        );
    }

    #[test]
    fn the_cancel_code_means_the_owner_cancelled() {
        assert_eq!(
            outcome("s", CANCEL, "", "", CANCEL),
            Err(DevError::Cancelled)
        );
    }

    #[test]
    fn without_a_cancel_code_every_code_is_a_failure() {
        assert!(matches!(
            outcome("signtool", CANCEL, "", "", None),
            Err(DevError::Failed { .. })
        ));
    }

    #[test]
    fn another_code_fails_with_the_first_stderr_line() {
        let err = outcome("dev-cert.ps1", Some(1), "", "\n  boom\nmore", CANCEL);
        assert_eq!(
            err,
            Err(DevError::Failed {
                step: "dev-cert.ps1".to_string(),
                detail: "boom".to_string(),
            })
        );
    }

    #[test]
    fn a_killed_process_fails() {
        assert!(matches!(
            outcome("s", None, "", "", CANCEL),
            Err(DevError::Failed { .. })
        ));
    }

    #[test]
    fn the_cancel_message_is_one_line() {
        assert!(!DevError::Cancelled.to_string().contains('\n'));
    }
}
