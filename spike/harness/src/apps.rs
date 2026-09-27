//! G1 apps: the app list, the run context, using a window already open, and cleaning up.

use std::path::{Path, PathBuf};
use std::process::Child;

use spike_core::config::SpikeConfig;
use spike_core::window::foreground;
use windows::Win32::Foundation::HWND;

use crate::config::{AppConfig, HarnessConfig, Timing};
use crate::keys;
use crate::win::{self, Match};

/// Placeholder in app arguments for the prepared file or URL.
pub const INPUT: &str = "{input}";

/// The G1 apps.
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
        }
    }

    /// The app called `name`.
    pub fn parse(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|a| a.name() == name)
    }
}

/// Paths and settings shared by one run.
pub struct Ctx<'a> {
    /// Harness settings.
    pub cfg: &'a HarnessConfig,
    /// Spike settings: the key block and the target window.
    pub spike: &'a SpikeConfig,
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
    /// Notepad's file, or Terminal's result file.
    pub file: Option<PathBuf>,
    /// Title text that spots the window (Notepad's file name).
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
pub fn clean_up(ctx: &Ctx, app: Opened) -> Vec<String> {
    // A window we did not open is never touched.
    let Some(mut child) = app.child else {
        return Vec::new();
    };
    let (t, k) = (&ctx.cfg.timing, &ctx.cfg.keys);
    let in_front = foreground() == app.hwnd;
    let mut left = Vec::new();
    match app.kind {
        AppKind::Target => left.extend(close_target(app.hwnd, &mut child, t)),
        AppKind::Notepad => {
            // Ctrl+W only while our file's tab is the one shown, never the owner's own tab.
            let ours = app
                .title_has
                .as_deref()
                .is_some_and(|t| win::title(app.hwnd).contains(t));
            if in_front && ours {
                let _ = keys::combo(&k.close_tab);
            }
            let m = Match {
                title_has: app.title_has,
                ..Default::default()
            };
            if let Some(w) = win::wait_no_match(&m, t.close_wait_ms, t.poll_ms) {
                left.push(format!("notepad left open: {}", win::describe(w)));
            }
        }
        AppKind::Chrome if in_front => {
            let _ = keys::combo(&k.close_tab);
            if !win::wait_gone(app.hwnd, t.close_wait_ms, t.poll_ms) {
                left.push(format!("left open: {}", win::describe(app.hwnd)));
            }
        }
        AppKind::Terminal if win::wait_gone(app.hwnd, t.close_wait_ms, t.poll_ms) => {}
        _ if win::exists(app.hwnd) => left.push(format!("left open: {}", win::describe(app.hwnd))),
        _ => {}
    }
    left
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
            args: args.iter().map(|a| a.to_string()).collect(),
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
    fn only_word_can_be_attached_with_harness_toml() {
        let cfg = crate::config::load().expect("harness.toml loads");
        let ok: Vec<&str> = AppKind::ALL
            .iter()
            .map(|k| k.name())
            .filter(|&n| attach_class(n, cfg.app(n).expect("app")).is_ok())
            .collect();
        assert_eq!(ok, [AppKind::Word.name()]);
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
