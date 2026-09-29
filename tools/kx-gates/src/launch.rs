//! Starts apps on fresh test files for G1, G4 and the probes, and waits until each can take text.
#![cfg(windows)]

use std::collections::HashSet;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

use windows::Win32::Foundation::HWND;

use crate::apps::{AppKind, Ctx, INPUT, Opened};
use crate::config::AppConfig;
use crate::launchterm;
use crate::win::{self, sleep_ms};
use crate::winfind::{self, Match};
use crate::{admin, out};

/// G1's page body: one text box that fills the page.
const TEXT_BOX: &str =
    "<textarea autofocus spellcheck=\"false\" style=\"width:95vw;height:90vh\"></textarea>";
/// Suffix of Notepad's test file.
const NOTEPAD_FILE: &str = ".txt";
/// Suffix of Chrome's test page.
const PAGE_FILE: &str = ".html";

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

/// What spots the app's window; windows open now are skipped when it opens a new one.
pub fn window_match(app: &AppConfig, title_has: Option<String>) -> Match {
    Match {
        title_has,
        class: app.class.clone(),
        skip: if app.new_window {
            win::snapshot()
        } else {
            HashSet::default()
        },
    }
}

/// The app's arguments with `{input}` filled in.
pub fn app_args(app: &AppConfig, input: &str) -> Vec<String> {
    app.args.iter().map(|a| a.replace(INPUT, input)).collect()
}

/// Starts `[apps.<name>]` on `input` and waits for its window; kills it if none comes.
pub fn launch(
    ctx: &Ctx,
    name: &str,
    input: &str,
    title_has: Option<String>,
) -> Result<(HWND, Child), String> {
    launch_with(ctx, name, input, title_has, &[])
}

/// Like `launch`, with `extra` arguments after the configured ones.
fn launch_with(
    ctx: &Ctx,
    name: &str,
    input: &str,
    title_has: Option<String>,
    extra: &[String],
) -> Result<(HWND, Child), String> {
    let (cfg, app) = (ctx.cfg, ctx.cfg.app(name)?);
    let m = window_match(app, title_has);
    let exe = cfg.program(&app.exe);
    let args = app_args(app, input)
        .into_iter()
        .chain(extra.iter().cloned());
    // No stdio of ours, which main also makes non-inheritable, so an app left open holds no pipe.
    let mut child = Command::new(&exe)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("{}: {e}", exe.display()))?;
    let Some(hwnd) = winfind::wait_for(&m, app.wait_ms, cfg.timing.poll_ms) else {
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
    let (t, title) = (&ctx.cfg.timing, ctx.target.title.clone());
    for w in win::top_windows()
        .into_iter()
        .filter(|&w| win::title(w) == title)
    {
        let _ = win::close(w);
        if !winfind::wait_gone(w, t.close_wait_ms, t.poll_ms) {
            return Err(format!(
                "an old target window stays open: {}",
                win::describe(w)
            ));
        }
    }
    let log = &ctx.target.log;
    match std::fs::remove_file(log) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
            return Err(format!("{}: {e}", log.display()));
        }
        _ => {}
    }
    let app = ctx.cfg.app(AppKind::Target.name())?;
    let exe = ctx.cfg.program(&app.exe);
    if !exe.is_file() {
        return Err(format!(
            "{} not found; run cargo build -p kx-target-window in the root workspace",
            exe.display()
        ));
    }
    Ok((exe, title))
}

/// Closes old target windows, deletes the log, starts target-window and waits for it.
pub fn start_target(ctx: &Ctx) -> Result<Opened, String> {
    start_target_with(ctx, &[])
}

/// Like `start_target`, with `extra` arguments, such as the ones that make a password box.
pub fn start_target_with(ctx: &Ctx, extra: &[String]) -> Result<Opened, String> {
    let (_, title) = prepare_target(ctx)?;
    let name = AppKind::Target.name();
    let started = launch_with(ctx, name, "", Some(title), extra)?;
    sleep_ms(ctx.cfg.app(name)?.ready_ms);
    Ok(opened(AppKind::Target, started, None, None))
}

/// Like `start_target`, but as administrator; we hold no process handle, so the caller closes its window.
pub fn start_target_admin(ctx: &Ctx) -> Result<Opened, String> {
    let (exe, title) = prepare_target(ctx)?;
    let app = ctx.cfg.app(AppKind::Target.name())?;
    let m = window_match(app, Some(title));
    admin::run_as(&exe, &[])?;
    let hwnd = winfind::wait_for(&m, app.wait_ms, ctx.cfg.timing.poll_ms)
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

/// Prepares a fresh document for `kind`, starts the app and waits until it can take text.
pub fn open(ctx: &Ctx, kind: AppKind) -> Result<Opened, String> {
    let app = ctx.cfg.app(kind.name())?;
    let done = match kind {
        AppKind::Target => return start_target(ctx),
        AppKind::Notepad => open_notepad(ctx, "")?,
        AppKind::Word => opened(kind, launch(ctx, kind.name(), "", None)?, None, None),
        AppKind::Chrome => open_page(ctx, &page_title(ctx, &ctx.cfg.g1.page_title), TEXT_BOX)?.0,
        AppKind::Terminal => launchterm::open_terminal(ctx)?,
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
        let cfg = crate::config::load().expect("kx-gates.toml loads");
        let app = AppConfig {
            args: vec!["-File".into(), INPUT.into(), format!("x={INPUT}")],
            ..cfg.app(AppKind::Terminal.name()).expect("terminal").clone()
        };
        assert_eq!(app_args(&app, "s.ps1"), ["-File", "s.ps1", "x=s.ps1"]);
    }
}
