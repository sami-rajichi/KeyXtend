//! G1 apps: the app list, the run context, using a window already open, and cleaning up.
#![cfg(windows)]

use std::path::{Path, PathBuf};
use std::process::Child;

use kx_target_window::config::TargetConfig;
use spike_core::config::SpikeConfig;
use spike_core::window::foreground;
use windows::Win32::Foundation::HWND;

use crate::config::{AppConfig, GatesConfig, Timing};
use crate::keys;
use crate::win;
use crate::winfind::{self, Match};

/// Placeholder in app arguments for the prepared file or URL.
pub const INPUT: &str = "{input}";

/// The apps kx-gates drives: G1's five, and Explorer for the probes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppKind {
    /// Our target-window, which logs each character it gets.
    Target,
    /// Windows Notepad, on a fresh text file.
    Notepad,
    /// Microsoft Word, on a blank document.
    Word,
    /// Google Chrome, on a local test page.
    Chrome,
    /// Windows Terminal, at a PowerShell prompt.
    Terminal,
    /// File Explorer on a test folder: a probe app, so not in `ALL`.
    Explorer,
}

impl AppKind {
    /// Every G1 app, in the order the usage lists them.
    pub const ALL: [AppKind; 5] = [
        Self::Target,
        Self::Notepad,
        Self::Word,
        Self::Chrome,
        Self::Terminal,
    ];

    /// Name used on the command line and in `[apps.<name>]`.
    pub fn name(self) -> &'static str {
        match self {
            Self::Target => "target",
            Self::Notepad => "notepad",
            Self::Word => "word",
            Self::Chrome => "chrome",
            Self::Terminal => "terminal",
            Self::Explorer => "explorer",
        }
    }

    /// The app called `name`.
    pub fn parse(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|a| a.name() == name)
    }
}

/// Paths and settings shared by one run.
pub struct Ctx<'a> {
    /// Gate settings.
    pub cfg: &'a GatesConfig,
    /// Spike settings: the key block and the faces.
    pub spike: &'a SpikeConfig,
    /// The target window's settings.
    pub target: &'a TargetConfig,
    /// Plain absolute out folder.
    pub out: PathBuf,
    /// File-name start for this run, e.g. `g1-notepad-1790000000`.
    pub base: String,
}

impl Ctx<'_> {
    /// A file in the out folder named `<base><suffix>`.
    pub fn file(&self, suffix: &str) -> PathBuf {
        self.out.join(format!("{}{suffix}", self.base))
    }
}

/// A started app and its window.
pub struct Opened {
    /// Which app it is.
    pub kind: AppKind,
    /// Its main window.
    pub hwnd: HWND,
    /// The process we started; none when we use a window already open.
    pub child: Option<Child>,
    /// A prepared file: Notepad's text, Terminal's result, Word's RTF or Explorer's folder.
    pub file: Option<PathBuf>,
    /// Title text that spots our window or tab: a file or folder name.
    pub title_has: Option<String>,
}

/// The class that spots `app`'s window, when `--attach` may use it: no prepared input, a class.
fn attach_class<'a>(name: &str, app: &'a AppConfig) -> Result<&'a str, String> {
    if app.args.iter().any(|a| a.contains(INPUT)) {
        return Err(format!(
            "--attach cannot use {name}: it opens a prepared {INPUT}"
        ));
    }
    app.class
        .as_deref()
        .ok_or_else(|| format!("--attach needs a class in [apps.{name}]"))
}

/// True if the program at `image` has the file name of `exe`, ignoring case.
fn same_program(image: &str, exe: &str) -> bool {
    let name = |p: &str| {
        Path::new(p)
            .file_name()
            .map(|n| n.to_string_lossy().to_lowercase())
    };
    name(image).is_some_and(|n| name(exe) == Some(n))
}

/// Uses the app's topmost open window of the right class and program; starts and closes nothing.
pub fn attach(ctx: &Ctx, kind: AppKind) -> Result<Opened, String> {
    let (name, app) = (kind.name(), ctx.cfg.app(kind.name())?);
    let class = attach_class(name, app)?;
    let ours = |w: HWND| {
        win::class(w) == class && win::program(w).is_ok_and(|p| same_program(&p, &app.exe))
    };
    let hwnd = win::top_windows()
        .into_iter()
        .find(|&w| ours(w))
        .ok_or_else(|| format!("no open {name} window of {}", app.exe))?;
    Ok(Opened {
        kind,
        hwnd,
        child: None,
        file: None,
        title_has: None,
    })
}

/// Closes what we started and is safe to close; returns what is left open.
pub fn clean_up(ctx: &Ctx, mut app: Opened) -> Vec<String> {
    // A window we did not open is never touched.
    let Some(mut child) = app.child.take() else {
        return Vec::new();
    };
    let t = &ctx.cfg.timing;
    let in_front = foreground() == app.hwnd;
    let mut left = Vec::new();
    match app.kind {
        AppKind::Target => left.extend(close_target(app.hwnd, &mut child, t)),
        AppKind::Notepad | AppKind::Explorer => left.extend(close_tab(&app, in_front, ctx)),
        AppKind::Word if app.file.is_some() => {
            // Word gets a file only when it opens a prepared one; we only read it, so no save prompt.
            let _ = win::close(app.hwnd);
            if !winfind::wait_gone(app.hwnd, t.close_wait_ms, t.poll_ms) {
                left.push(format!("left open: {}", win::describe(app.hwnd)));
            }
        }
        AppKind::Chrome if in_front => left.extend(close_pages(&app, ctx)),
        AppKind::Terminal if winfind::wait_gone(app.hwnd, t.close_wait_ms, t.poll_ms) => {}
        _ if winfind::exists(app.hwnd) => {
            left.push(format!("left open: {}", win::describe(app.hwnd)));
        }
        _ => {}
    }
    left
}

/// Closes our tab with Ctrl+W, only while it is the one shown, never the owner's own tab; notes one left open.
fn close_tab(app: &Opened, in_front: bool, ctx: &Ctx) -> Option<String> {
    let (t, k) = (&ctx.cfg.timing, &ctx.cfg.keys);
    let ours = app
        .title_has
        .as_deref()
        .is_some_and(|t| win::title(app.hwnd).contains(t));
    if in_front && ours {
        let _ = keys::combo(&k.close_tab);
    }
    let m = Match {
        title_has: app.title_has.clone(),
        ..Default::default()
    };
    let w = winfind::wait_no_match(&m, t.close_wait_ms, t.poll_ms)?;
    Some(format!(
        "{} left open: {}",
        app.kind.name(),
        win::describe(w)
    ))
}

/// True when a window title starts with one of our test page titles; an empty one never counts.
pub fn is_test_page(title: &str, pages: &[&str]) -> bool {
    pages.iter().any(|p| !p.is_empty() && title.starts_with(p))
}

/// Closes Chrome tabs with Ctrl+W while the one in front is a test page of ours; notes the window if it stays.
///
/// Chrome may reopen an old test tab next to ours; any other tab stops the loop and is never closed.
fn close_pages(app: &Opened, ctx: &Ctx) -> Option<String> {
    let (t, k) = (&ctx.cfg.timing, &ctx.cfg.keys);
    let pages = [
        ctx.cfg.g1.page_title.as_str(),
        ctx.cfg.probes.page_title.as_str(),
    ];
    while winfind::exists(app.hwnd) && foreground() == app.hwnd {
        let before = win::title(app.hwnd);
        if !is_test_page(&before, &pages) {
            break;
        }
        let _ = keys::combo(&k.close_tab);
        let changed =
            || (!winfind::exists(app.hwnd) || win::title(app.hwnd) != before).then_some(());
        if win::poll_until(t.close_wait_ms, t.poll_ms, changed).is_none() {
            break;
        }
    }
    winfind::exists(app.hwnd).then(|| format!("left open: {}", win::describe(app.hwnd)))
}

/// Closes target-window and kills it if it does not end in time; returns a note if killed.
fn close_target(hwnd: HWND, child: &mut Child, t: &Timing) -> Option<String> {
    let _ = win::close(hwnd);
    if wait_exit(child, t.close_wait_ms, t.poll_ms) {
        return None;
    }
    let _ = child.kill();
    Some("target-window did not close; killed it".to_string())
}

/// Waits up to `timeout_ms` for `child` to end; true if it did.
fn wait_exit(child: &mut Child, timeout_ms: u64, poll_ms: u64) -> bool {
    let ended = || matches!(child.try_wait(), Ok(Some(_))).then_some(());
    win::poll_until(timeout_ms, poll_ms, ended).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app(args: &[&str], class: Option<&str>) -> AppConfig {
        AppConfig {
            exe: "x.exe".into(),
            args: args.iter().map(ToString::to_string).collect(),
            class: class.map(String::from),
            focus_class: None,
            new_window: true,
            wait_ms: 0,
            ready_ms: 0,
            done_ms: 0,
        }
    }

    #[test]
    fn attach_needs_a_class_and_no_prepared_input() {
        let word = app(&["/q", "/w"], Some("OpusApp"));
        assert_eq!(attach_class("word", &word), Ok("OpusApp"));
        let notepad = app(&[INPUT], Some("Notepad"));
        assert!(attach_class("notepad", &notepad).is_err());
        let chrome = app(&["--new-window", INPUT], Some("Chrome_WidgetWin_1"));
        assert!(attach_class("chrome", &chrome).is_err());
        assert!(attach_class("target", &app(&[], None)).is_err());
    }

    #[test]
    fn only_word_can_be_attached_with_the_shipped_file() {
        let cfg = crate::config::load().expect("kx-gates.toml loads");
        let ok: Vec<&str> = AppKind::ALL
            .iter()
            .map(|k| k.name())
            .filter(|&n| attach_class(n, cfg.app(n).expect("app")).is_ok())
            .collect();
        assert_eq!(ok, [AppKind::Word.name()]);
    }

    #[test]
    fn only_titles_that_start_with_a_test_page_count() {
        let pages = ["KeyXtend probe page", ""];
        assert!(is_test_page(
            "KeyXtend probe page g6-x - Google Chrome",
            &pages
        ));
        assert!(!is_test_page("News - KeyXtend probe page", &pages));
        assert!(!is_test_page("Mail - Google Chrome", &pages));
        assert!(!is_test_page("anything", &[""]));
    }

    #[test]
    fn same_program_compares_file_names_ignoring_case() {
        let word = r"C:\Program Files\Microsoft Office\root\Office16\WINWORD.EXE";
        assert!(same_program(
            r"C:\Program Files\Microsoft Office\root\Office16\winword.exe",
            word
        ));
        assert!(same_program(r"D:\Other\WinWord.exe", "WINWORD.EXE"));
        assert!(!same_program(r"C:\Windows\notepad.exe", word));
        assert!(!same_program("", word));
    }
}
