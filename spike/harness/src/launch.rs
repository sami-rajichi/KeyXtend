//! Starts apps on fresh test files for G1, G4 and the probes, and waits until each can take text.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use windows::Win32::Foundation::HWND;

use crate::apps::{AppKind, Ctx, INPUT, Opened};
use crate::config::AppConfig;
use crate::win::{self, Match, sleep_ms};
use crate::{admin, out};

/// Placeholder in `PS1` for the ready file.
const READY: &str = "{ready}";
/// Placeholder in `PS1` for the result file.
const RESULT: &str = "{result}";
/// G1's page body: one text box that fills the page.
const TEXT_BOX: &str =
    "<textarea autofocus spellcheck=\"false\" style=\"width:95vw;height:90vh\"></textarea>";
/// Terminal script: writes whether it runs as administrator, reads one line, saves it as UTF-8 without BOM.
const PS1: &str = "\u{FEFF}$a = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::\
GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)\r\n\
[IO.File]::WriteAllText('{ready}', \"$a\")\r\n$t = Read-Host\r\n\
[IO.File]::WriteAllText('{result}', $t, [Text.UTF8Encoding]::new($false))\r\n";
/// What the script writes to the ready file when it runs as administrator.
const ADMIN_ANSWER: &str = "True";
/// Suffix of Notepad's test file.
const NOTEPAD_FILE: &str = ".txt";
/// Suffix of Chrome's test page.
const PAGE_FILE: &str = ".html";
/// Suffix of Terminal's script.
const SCRIPT_FILE: &str = ".ps1";
/// Suffix of the file Terminal's script writes once it waits for a line.
const READY_FILE: &str = "-ready.txt";
/// Suffix of the file Terminal's script saves the typed line to.
const RESULT_FILE: &str = "-result.txt";

/// `text` made safe as HTML element text.
pub fn html_text(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// The page title `base` made unique to this run, so no two of our tabs share one.
pub fn page_title(ctx: &Ctx, base: &str) -> String {
    format!("{base} {}", ctx.base)
}

/// A Chrome test page called `title` around `body`, which is HTML already.
fn page(title: &str, body: &str) -> String {
    let title = html_text(title);
    format!(
        "<!doctype html>\n<html><head><meta charset=\"utf-8\"><title>{title}</title></head>\n\
<body>{body}</body></html>\n"
    )
}

/// True when the ready file says the script runs as administrator.
fn is_admin(ready: &str) -> bool {
    ready.trim() == ADMIN_ANSWER
}

/// `path` made safe inside a single-quoted PowerShell string.
fn ps_quote(path: &Path) -> String {
    path.to_string_lossy().replace('\'', "''")
}

/// What spots the app's window; windows open now are skipped when it opens a new one.
fn window_match(app: &AppConfig, title_has: Option<String>) -> Match {
    Match {
        title_has,
        class: app.class.clone(),
        skip: if app.new_window {
            win::snapshot()
        } else {
            Default::default()
        },
    }
}

/// The app's arguments with `{input}` filled in.
fn app_args(app: &AppConfig, input: &str) -> Vec<String> {
    app.args.iter().map(|a| a.replace(INPUT, input)).collect()
}

/// Starts `[apps.<name>]` on `input` and waits for its window; kills it if none comes.
pub fn launch(
    ctx: &Ctx,
    name: &str,
    input: &str,
    title_has: Option<String>,
) -> Result<(HWND, Child), String> {
    let (cfg, app) = (ctx.cfg, ctx.cfg.app(name)?);
    let m = window_match(app, title_has);
    let exe = cfg.program(&app.exe);
    let args = app_args(app, input);
    // No stdio of ours, which main also makes non-inheritable, so an app left open holds no pipe.
    let mut child = Command::new(&exe)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("{}: {e}", exe.display()))?;
    let Some(hwnd) = win::wait_for(&m, app.wait_ms, cfg.timing.poll_ms) else {
        // No window in time: end what we started, so nothing is left behind.
        let _ = child.kill();
        return Err(format!("no {name} window after {} ms", app.wait_ms));
    };
    Ok((hwnd, child))
}

/// An app we started, with its window.
pub fn opened(
    kind: AppKind,
    (hwnd, child): (HWND, Child),
    file: Option<PathBuf>,
    title_has: Option<String>,
) -> Opened {
    Opened {
        kind,
        hwnd,
        child: Some(child),
        file,
        title_has,
    }
}

/// Closes old target windows and deletes the log; returns target-window's program and title.
fn prepare_target(ctx: &Ctx) -> Result<(PathBuf, String), String> {
    let (t, title) = (&ctx.cfg.timing, ctx.spike.target.title.clone());
    for w in win::top_windows()
        .into_iter()
        .filter(|&w| win::title(w) == title)
    {
        let _ = win::close(w);
        if !win::wait_gone(w, t.close_wait_ms, t.poll_ms) {
            return Err(format!(
                "an old target window stays open: {}",
                win::describe(w)
            ));
        }
    }
    let log = ctx.spike.resolve(&ctx.spike.target.log);
    match std::fs::remove_file(&log) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
            return Err(format!("{}: {e}", log.display()));
        }
        _ => {}
    }
    let app = ctx.cfg.app(AppKind::Target.name())?;
    let exe = ctx.cfg.program(&app.exe);
    if !exe.is_file() {
        return Err(format!(
            "{} not found; run cargo build -p target-window",
            exe.display()
        ));
    }
    Ok((exe, title))
}

/// Closes old target windows, deletes the log, starts target-window and waits for it.
pub fn start_target(ctx: &Ctx) -> Result<Opened, String> {
    let (_, title) = prepare_target(ctx)?;
    let started = launch(ctx, AppKind::Target.name(), "", Some(title))?;
    sleep_ms(ctx.cfg.app(AppKind::Target.name())?.ready_ms);
    Ok(opened(AppKind::Target, started, None, None))
}

/// Like `start_target`, but as administrator; we hold no process handle, so the caller closes its window.
pub fn start_target_admin(ctx: &Ctx) -> Result<Opened, String> {
    let (exe, title) = prepare_target(ctx)?;
    let app = ctx.cfg.app(AppKind::Target.name())?;
    let m = window_match(app, Some(title));
    admin::run_as(&exe, &[])?;
    let hwnd = win::wait_for(&m, app.wait_ms, ctx.cfg.timing.poll_ms)
        .ok_or_else(|| format!("no admin target window after {} ms", app.wait_ms))?;
    sleep_ms(app.ready_ms);
    Ok(Opened {
        kind: AppKind::Target,
        hwnd,
        child: None,
        file: None,
        title_has: None,
    })
}

/// Notepad on a fresh text file holding `text`, spotted by the file name in its title.
pub fn open_notepad(ctx: &Ctx, text: &str) -> Result<Opened, String> {
    let file = ctx.file(NOTEPAD_FILE);
    out::write(&file, text)?;
    let name = file.file_name().map(|n| n.to_string_lossy().into_owned());
    let kind = AppKind::Notepad;
    let started = launch(ctx, kind.name(), &file.to_string_lossy(), name.clone())
        .inspect_err(|_| drop(std::fs::remove_file(&file)))?;
    Ok(opened(kind, started, Some(file), name))
}

/// Chrome on a local page called `title` holding `body`; returns it and the page file.
pub fn open_page(ctx: &Ctx, title: &str, body: &str) -> Result<(Opened, PathBuf), String> {
    let file = ctx.file(PAGE_FILE);
    out::write(&file, &page(title, body))?;
    let kind = AppKind::Chrome;
    let title = Some(title.to_string());
    let started = launch(ctx, kind.name(), &out::file_url(&file), title.clone())
        .inspect_err(|_| drop(std::fs::remove_file(&file)))?;
    Ok((opened(kind, started, None, title), file))
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
fn open_terminal(ctx: &Ctx) -> Result<Opened, String> {
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
    let hwnd = win::wait_for(&m, app.wait_ms, poll)
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

/// Prepares a fresh document for `kind`, starts the app and waits until it can take text.
pub fn open(ctx: &Ctx, kind: AppKind) -> Result<Opened, String> {
    let app = ctx.cfg.app(kind.name())?;
    let done = match kind {
        AppKind::Target => return start_target(ctx),
        AppKind::Notepad => open_notepad(ctx, "")?,
        AppKind::Word => opened(kind, launch(ctx, kind.name(), "", None)?, None, None),
        AppKind::Chrome => open_page(ctx, &page_title(ctx, &ctx.cfg.g1.page_title), TEXT_BOX)?.0,
        AppKind::Terminal => open_terminal(ctx)?,
        AppKind::Explorer => return Err("Explorer is a probe app, not a G1 app".to_string()),
    };
    sleep_ms(app.ready_ms);
    Ok(done)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_escapes_the_title_but_keeps_the_body_html() {
        let p = page("a<b", "<p>x</p>");
        assert!(p.contains("<title>a&lt;b</title>") && p.contains("<body><p>x</p></body>"));
    }

    #[test]
    fn html_text_escapes_markup_once() {
        assert_eq!(html_text("a&b<c>d"), "a&amp;b&lt;c&gt;d");
        assert_eq!(html_text("&lt;"), "&amp;lt;");
        assert_eq!(html_text("KeyXtend G1"), "KeyXtend G1");
    }

    #[test]
    fn app_args_fill_the_input_in_every_argument() {
        let cfg = crate::config::load().expect("harness.toml loads");
        let app = AppConfig {
            args: vec!["-File".into(), INPUT.into(), format!("x={INPUT}")],
            ..cfg.app(AppKind::Terminal.name()).expect("terminal").clone()
        };
        assert_eq!(app_args(&app, "s.ps1"), ["-File", "s.ps1", "x=s.ps1"]);
    }

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
