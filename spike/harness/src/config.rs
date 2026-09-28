//! Loads `harness.toml`: next to the running exe first, else the spike folder.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use spike_core::config::settings_path;

use crate::featcfg::{self, EditKeys, G19, G20, G21, G22, G23, G24, G25};
use crate::probecfg::{G6, Probes, Sim};
use crate::text::Charsets;

/// File name of the harness settings.
const FILE: &str = "harness.toml";

/// All harness settings.
#[derive(Debug, Clone, Deserialize)]
pub struct HarnessConfig {
    /// Where results go.
    pub paths: Paths,
    /// Shared waits and retries.
    pub timing: Timing,
    /// Shortcut key combos.
    pub keys: Keys,
    /// G1 character groups.
    pub text: Charsets,
    /// G1 settings.
    pub g1: G1,
    /// G1 apps by name.
    pub apps: BTreeMap<String, AppConfig>,
    /// G2 settings.
    pub g2: G2,
    /// G3 settings.
    pub g3: G3,
    /// G4 settings.
    pub g4: G4,
    /// Hold engine settings.
    pub assist: AssistCfg,
    /// The simulated user's timing.
    pub sim: Sim,
    /// G5 settings.
    pub g5: G5,
    /// G17, G18 and G6 documents and moves.
    pub probes: Probes,
    /// G6 settings.
    pub g6: G6,
    /// Editing shortcuts of the stage-1b probes.
    pub edit_keys: EditKeys,
    /// G19 settings.
    pub g19: G19,
    /// G20 settings.
    pub g20: G20,
    /// G21 settings.
    pub g21: G21,
    /// G22 settings.
    pub g22: G22,
    /// G23 settings.
    pub g23: G23,
    /// G24 settings.
    pub g24: G24,
    /// G25 settings.
    pub g25: G25,
    /// Folder the file was read from; relative paths start here.
    #[serde(skip)]
    pub dir: PathBuf,
}

/// Output folder.
#[derive(Debug, Clone, Deserialize)]
pub struct Paths {
    /// Results folder, relative to the settings folder.
    pub out: String,
}

/// Shared waits (ms) and retries.
#[derive(Debug, Clone, Deserialize)]
pub struct Timing {
    /// How often we look again for a window, file or clipboard change.
    pub poll_ms: u64,
    /// Pause after the Alt tap, before `SetForegroundWindow`.
    pub alt_settle_ms: u64,
    /// Tries to bring a window to the front.
    pub front_tries: u32,
    /// Wait after each try to bring a window to the front.
    pub front_wait_ms: u64,
    /// Pause after a shortcut such as Ctrl+A or Enter.
    pub key_settle_ms: u64,
    /// Longest wait for a clipboard change or a saved file.
    pub read_wait_ms: u64,
    /// Longest wait for a window we closed to go away.
    pub close_wait_ms: u64,
    /// Longest wait for an app's text box to take the keyboard.
    pub focus_wait_ms: u64,
}

/// Virtual-key combos: pressed in order, released in reverse.
#[derive(Debug, Clone, Deserialize)]
pub struct Keys {
    /// Alt tap that lets us bring a window to the front.
    pub alt: Vec<u16>,
    /// Select all.
    pub select_all: Vec<u16>,
    /// Copy.
    pub copy: Vec<u16>,
    /// Save.
    pub save: Vec<u16>,
    /// Close the tab or document.
    pub close_tab: Vec<u16>,
    /// Enter.
    pub enter: Vec<u16>,
}

/// G1 typing settings.
#[derive(Debug, Clone, Deserialize)]
pub struct G1 {
    /// Characters typed per run.
    pub count: usize,
    /// Characters per `SendInput` call.
    pub chunk_chars: usize,
    /// Pause after each chunk, in ms.
    pub chunk_pause_ms: u64,
    /// Title of the Chrome test page, used to find its window.
    pub page_title: String,
    /// Characters shown on each side of the first difference.
    pub context_chars: usize,
    /// Longest clipboard text we read, in characters; more is an error.
    pub max_read_chars: usize,
}

/// One G1 app: how to start it, spot its window, and how long to wait (ms).
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    /// Program; a relative path with a folder starts at the settings folder.
    pub exe: String,
    /// Arguments; `{input}` becomes the prepared file or URL.
    pub args: Vec<String>,
    /// Window class, if it helps to spot the window; `--attach` needs it.
    #[serde(default)]
    pub class: Option<String>,
    /// Class of the text box that must hold the keyboard before and while typing.
    #[serde(default)]
    pub focus_class: Option<String>,
    /// Only a window that did not exist before counts.
    pub new_window: bool,
    /// Longest wait for the window.
    pub wait_ms: u64,
    /// Pause before typing.
    pub ready_ms: u64,
    /// Pause before reading back.
    pub done_ms: u64,
}

/// G2 click settings.
#[derive(Debug, Clone, Deserialize)]
pub struct G2 {
    /// Clicks per run.
    pub clicks: usize,
    /// Pause after each click before checking focus and caret, in ms.
    pub click_gap_ms: u64,
    /// A character belongs to a click if it arrives within this time, in ms.
    pub match_window_ms: i64,
    /// Wait for the last characters to reach the log, in ms.
    pub drain_ms: u64,
    /// Failures listed in the report.
    pub failures_shown: usize,
    /// Failures that stop the run.
    pub max_failures: usize,
}

/// G3 top-band settings.
#[derive(Debug, Clone, Deserialize)]
pub struct G3 {
    /// Wait after opening a surface before probing, in ms.
    pub open_wait_ms: u64,
    /// Scan codes of the probed keys.
    pub probe_codes: Vec<u32>,
    /// System surfaces opened over the face.
    pub surfaces: Vec<Surface>,
}

/// G4 admin-window settings; click pauses and failure limits come from G2.
#[derive(Debug, Clone, Deserialize)]
pub struct G4 {
    /// Clicks per run.
    pub clicks: usize,
}

/// Hold engine settings (spec §5.1).
#[derive(Debug, Clone, Deserialize)]
pub struct AssistCfg {
    /// A still left hold fires after this, in ms.
    pub hold_ms: u64,
    /// Moves up to this many px count as still.
    pub still_px: i32,
    /// Longest wait for the hook thread to start or answer, in ms.
    pub reply_ms: u64,
    /// How long the owner's hand try runs, in seconds.
    pub try_secs: u64,
    /// How often the Grab hand try turns Grab back on after a drop, in ms.
    pub rearm_ms: u64,
}

/// G5 hold-engine settings; times in ms, distances in px.
#[derive(Debug, Clone, Deserialize)]
pub struct G5 {
    /// A drag while pressed.
    pub drag_px: i32,
    /// How far inside and outside the still radius the wiggle and the short move go.
    pub still_margin_px: i32,
    /// How many times the cases run.
    pub rounds: usize,
    /// Wait for the target log after each case.
    pub settle_ms: u64,
    /// How far a logged point may be from where we pressed.
    pub point_slack_px: i32,
    /// Budget for the hook callback's 99th percentile.
    pub hook_p99_ms: f64,
}

/// A system surface to open over the face.
#[derive(Debug, Clone, Deserialize)]
pub struct Surface {
    /// Name in the report.
    pub name: String,
    /// Virtual keys that open it.
    pub open: Vec<u16>,
    /// Virtual keys that close it.
    #[serde(default)]
    pub close_keys: Vec<u16>,
    /// Class of a new window to close with `WM_CLOSE`.
    #[serde(default)]
    pub close_class: Option<String>,
}

impl HarnessConfig {
    /// The results folder.
    pub fn out_dir(&self) -> PathBuf {
        self.dir.join(&self.paths.out)
    }

    /// The settings of `app`.
    pub fn app(&self, app: &str) -> Result<&AppConfig, String> {
        self.apps
            .get(app)
            .ok_or_else(|| format!("no [apps.{app}] in {FILE}"))
    }

    /// `exe` as a program to run: relative paths with a folder start at the settings folder.
    pub fn program(&self, exe: &str) -> PathBuf {
        let path = Path::new(exe);
        if path.is_relative() && path.components().count() > 1 {
            self.dir.join(path)
        } else {
            path.to_path_buf()
        }
    }
}

/// Loads and parses the settings.
pub fn load() -> Result<HarnessConfig, String> {
    let file = settings_path(FILE);
    let text = std::fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
    let mut config: HarnessConfig =
        toml::from_str(&text).map_err(|e| format!("{}: {e}", file.display()))?;
    config.dir = file.parent().map(Path::to_path_buf).unwrap_or_default();
    config.text.validate()?;
    featcfg::check(&config)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn harness_toml_loads_and_has_every_app() {
        let cfg = load().expect("harness.toml loads");
        for app in crate::apps::AppKind::ALL {
            assert!(cfg.app(app.name()).is_ok(), "missing app {}", app.name());
        }
        assert_eq!(cfg.g3.probe_codes.len(), 3);
        assert!(cfg.g4.clicks > 0);
        assert!(cfg.assist.rearm_ms > 0 && cfg.assist.rearm_ms < cfg.assist.hold_ms);
        for app in ["explorer", "word_file"] {
            assert!(cfg.app(app).is_ok(), "missing app {app}");
        }
        assert!(cfg.probes.move_steps > 0 && cfg.g6.notches > 0);
        assert!(!cfg.g25.panel_classes.is_empty());
    }

    #[test]
    fn program_resolves_only_relative_paths_with_a_folder() {
        let cfg = HarnessConfig {
            dir: PathBuf::from("base"),
            ..load().expect("loads")
        };
        assert_eq!(cfg.program("notepad.exe"), PathBuf::from("notepad.exe"));
        assert_eq!(
            cfg.program("../t/x.exe"),
            Path::new("base").join("../t/x.exe")
        );
    }
}
