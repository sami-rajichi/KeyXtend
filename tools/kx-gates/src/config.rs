//! Loads and checks `kx-gates.toml` and `kx-gates-p3.toml`; `cfgfile` finds the first, and the second sits beside it.
#![cfg(windows)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::capture::Capture;
use crate::cfgfile::{self, FILE, P3_FILE};
use crate::featcfg::{self, EditKeys, G7, G19, G20, G21, G23, G24, G25};
use crate::g8cfg::G8;
use crate::g22cfg::G22;
use crate::gatecfg::{AssistCfg, G1, G2, G3, G4, G5};
use crate::p3cfg::{self, G9, G10, G11, G13, G26};
use crate::probecfg::{G6, G12, Probes, Sim};
use crate::text::Charsets;

/// All gate settings.
#[derive(Debug, Clone, Deserialize)]
pub struct GatesConfig {
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
    /// G12 settings.
    pub g12: G12,
    /// Editing shortcuts of the stage-1b probes.
    pub edit_keys: EditKeys,
    /// G7 settings.
    pub g7: G7,
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
    /// `capture-layouts` settings.
    pub capture: Capture,
    /// G8 settings.
    pub g8: G8,
    /// G9 settings.
    pub g9: G9,
    /// G10 settings.
    pub g10: G10,
    /// G11 settings.
    pub g11: G11,
    /// G13 settings.
    pub g13: G13,
    /// G26 settings.
    pub g26: G26,
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
    /// Longest wait for the product to end after `close` posts it Quit.
    pub quit_wait_ms: u64,
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

impl GatesConfig {
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

/// The settings file at `path`, parsed.
fn read_table(path: &Path) -> Result<toml::Table, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    text.parse().map_err(|e| format!("{}: {e}", path.display()))
}

/// `main` with the sections of `extra`, which may not repeat one.
fn merge(mut main: toml::Table, extra: toml::Table) -> Result<toml::Table, String> {
    for (name, section) in extra {
        if main.contains_key(&name) {
            return Err(format!("[{name}] is in both {FILE} and {P3_FILE}"));
        }
        main.insert(name, section);
    }
    Ok(main)
}

/// Loads and parses the settings: `kx-gates.toml`, then `kx-gates-p3.toml` from the same folder.
pub fn load() -> Result<GatesConfig, String> {
    let file = cfgfile::path();
    let dir = file.parent().map(Path::to_path_buf).unwrap_or_default();
    let table = merge(read_table(&file)?, read_table(&dir.join(P3_FILE))?)?;
    let mut config: GatesConfig = table
        .try_into()
        .map_err(|e| format!("{} or {P3_FILE}: {e}", file.display()))?;
    config.dir = dir;
    config.text.validate()?;
    featcfg::check(&config)?;
    config.g3.check()?;
    config.g12.check()?;
    config.g8.check()?;
    p3cfg::check(&config.g11, &config.g13, &config.g26)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shipped_file_loads_and_has_every_app() {
        let cfg = load().expect("kx-gates.toml loads");
        for app in crate::apps::AppKind::ALL {
            assert!(cfg.app(app.name()).is_ok(), "missing app {}", app.name());
        }
        assert_eq!(cfg.g3.probe_keys.len(), 3);
        assert!(cfg.g4.clicks > 0 && cfg.timing.quit_wait_ms > 0);
        let hold = spike_core::config::load().expect("spike.toml loads").hold;
        assert!(cfg.assist.rearm_ms > 0 && cfg.assist.rearm_ms < hold.ms);
        for app in ["explorer", "word_file"] {
            assert!(cfg.app(app).is_ok(), "missing app {app}");
        }
        assert!(cfg.probes.move_steps > 0 && cfg.g6.notches > 0);
        assert!(!cfg.g25.panel_classes.is_empty());
    }

    #[test]
    fn the_p3_file_adds_sections_and_never_replaces_one() {
        let table = |text: &str| text.parse::<toml::Table>().expect("parses");
        let both =
            merge(table("[g2]\nclicks = 1\n"), table("[g8]\nclicks = 2\n")).expect("no overlap");
        assert_eq!(both.keys().collect::<Vec<_>>(), ["g2", "g8"]);
        let e =
            merge(table("[g2]\nclicks = 1\n"), table("[g2]\nclicks = 3\n")).expect_err("g2 twice");
        assert!(e.contains("[g2]"), "{e}");
    }

    #[test]
    fn g3_probes_only_keys_of_the_product_board() {
        let mut g3 = load().expect("kx-gates.toml loads").g3;
        assert!(g3.check().is_ok());
        g3.probe_keys.push("c-0xFFFF".into());
        let e = g3.check().expect_err("no such key");
        assert!(e.contains("c-0xFFFF"), "{e}");
        g3.probe_keys.clear();
        assert!(g3.check().is_err(), "probing nothing checks nothing");
    }

    #[test]
    fn the_target_window_title_is_no_title_of_the_spike() {
        let target = kx_target_window::config::load().expect("target config loads");
        let spike = spike_core::config::load().expect("spike.toml loads");
        let taken = spike_core::panelcfg::titles(&spike);
        assert!(
            !taken.contains(&&target.title),
            "kx-gates closes windows by title, so the target's must be its own"
        );
    }

    #[test]
    fn program_resolves_only_relative_paths_with_a_folder() {
        let cfg = GatesConfig {
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
