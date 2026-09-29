//! Terminal's launch: a script that saves one typed line, in a normal or an administrator Terminal.
#![cfg(windows)]

use std::path::{Path, PathBuf};

use crate::apps::{AppKind, Ctx, Opened};
use crate::launch::{app_args, launch, opened, window_match};
use crate::win::{self, sleep_ms};
use crate::winfind;
use crate::{admin, out};

/// Placeholder in `PS1` for the ready file.
const READY: &str = "{ready}";
/// Placeholder in `PS1` for the result file.
const RESULT: &str = "{result}";
/// Terminal script: writes whether it runs as administrator, reads one line, saves it as UTF-8 without BOM.
const PS1: &str = "\u{FEFF}$a = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::\
GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)\r\n\
[IO.File]::WriteAllText('{ready}', \"$a\")\r\n$t = Read-Host\r\n\
[IO.File]::WriteAllText('{result}', $t, [Text.UTF8Encoding]::new($false))\r\n";
/// What the script writes to the ready file when it runs as administrator.
const ADMIN_ANSWER: &str = "True";
/// Suffix of Terminal's script.
const SCRIPT_FILE: &str = ".ps1";
/// Suffix of the file Terminal's script writes once it waits for a line.
const READY_FILE: &str = "-ready.txt";
/// Suffix of the file Terminal's script saves the typed line to.
const RESULT_FILE: &str = "-result.txt";

/// True when the ready file says the script runs as administrator.
fn is_admin(ready: &str) -> bool {
    ready.trim() == ADMIN_ANSWER
}

/// `path` made safe inside a single-quoted PowerShell string.
fn ps_quote(path: &Path) -> String {
    path.to_string_lossy().replace('\'', "''")
}

/// Writes Terminal's script; returns it, the file it marks ready in, and the file it saves to.
fn terminal_files(ctx: &Ctx) -> Result<(PathBuf, PathBuf, PathBuf), String> {
    let (script, ready, result) = (
        ctx.file(SCRIPT_FILE),
        ctx.file(READY_FILE),
        ctx.file(RESULT_FILE),
    );
    let text = PS1
        .replace(READY, &ps_quote(&ready))
        .replace(RESULT, &ps_quote(&result));
    out::write(&script, &text)?;
    Ok((script, ready, result))
}

/// Terminal running a script that saves one typed line; waits until the script is ready.
pub fn open_terminal(ctx: &Ctx) -> Result<Opened, String> {
    let (script, ready, result) = terminal_files(ctx)?;
    let kind = AppKind::Terminal;
    let (wait, poll) = (ctx.cfg.app(kind.name())?.wait_ms, ctx.cfg.timing.poll_ms);
    let started = launch(ctx, kind.name(), &script.to_string_lossy(), None)?;
    out::wait_read(&ready, wait, poll).map_err(|e| format!("terminal not ready: {e}"))?;
    Ok(opened(kind, started, Some(result), None))
}

/// Terminal as administrator on the one-line script; also says whether the script runs as admin.
pub fn open_admin(ctx: &Ctx) -> Result<(Opened, bool), String> {
    let kind = AppKind::Terminal;
    let (app, poll) = (ctx.cfg.app(kind.name())?, ctx.cfg.timing.poll_ms);
    let (script, ready, result) = terminal_files(ctx)?;
    let m = window_match(app, None);
    let args = app_args(app, &script.to_string_lossy());
    admin::run_as(&ctx.cfg.program(&app.exe), &args)?;
    let hwnd = winfind::wait_for(&m, app.wait_ms, poll)
        .ok_or_else(|| format!("no admin {} window after {} ms", kind.name(), app.wait_ms))?;
    let answer = out::wait_read(&ready, app.wait_ms, poll).map_err(|e| {
        let _ = win::close(hwnd);
        format!("admin {} not ready: {e}", kind.name())
    })?;
    sleep_ms(app.ready_ms);
    let opened = Opened {
        kind,
        hwnd,
        child: None,
        file: Some(result),
        title_has: None,
    };
    Ok((opened, is_admin(&answer)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_admin_reads_the_script_answer() {
        assert!(is_admin("True"));
        assert!(is_admin("True\r\n"));
        assert!(!is_admin("False"));
        assert!(!is_admin(""));
    }

    #[test]
    fn ps_quote_doubles_single_quotes() {
        assert_eq!(ps_quote(Path::new(r"D:\it's\r.txt")), r"D:\it''s\r.txt");
        assert_eq!(ps_quote(Path::new(r"D:\plain.txt")), r"D:\plain.txt");
    }
}
