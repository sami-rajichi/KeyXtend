//! Loads and checks `kx-gates.toml` and the files each phase adds (`kx-gates-p3.toml` to `kx-gates-p7.toml`);
//! `cfgfile` finds their folder and `cfgload` reads them.
#![cfg(windows)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::capture::Capture;
use crate::cfgfile::{self, FILE, OWN};
use crate::cfgload;
use crate::featcfg::{self, EditKeys, G7, G19, G20, G21, G23, G24, G25};
use crate::fit::{self, Fit};
use crate::g8cfg::G8;
use crate::g8lit::Osk;
use crate::g22cfg::G22;
use crate::gatecfg::{G1, G2, G3, G4};
use crate::mouseguard::MouseGuard;
use crate::p3cfg::{self, G9, G10, G11, G13, G26};
use crate::p4cfg::{G5, G8Hook, G12, GateInput, Hand};
use crate::p5cfg::{self, G6, G8Scroll, Host, Outline, Pad};
use crate::p7cfg::{self, G8Predict, G21Product, G27, OwnerSettings};
use crate::probecfg::{Probes, Sim};
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
    /// The owner's hand try.
    pub hand: Hand,
    /// The simulated user's timing.
    pub sim: Sim,
    /// G5 settings.
    pub g5: G5,
    /// G17, G18 and G6 documents and moves.
    pub probes: Probes,
    /// G6 settings, on the test build.
    pub g6: G6,
    /// The scroll pad's parts, which G6, G9 and G10 tap and press.
    pub pad: Pad,
    /// G9's outline steps.
    pub outline: Outline,
    /// G8's scroll steps.
    pub g8_scroll: G8Scroll,
    /// What G8 logs of the machine beside its numbers.
    pub host: Host,
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
    /// G8's hook part, on the test build.
    pub g8_hook: G8Hook,
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
    /// The On-Screen Keyboard that `g8 osk` times.
    pub osk: Osk,
    /// The mouse guard, shared by every gate.
    pub mouse_guard: MouseGuard,
    /// What a run spends beyond its counted steps, for the run-length checks.
    pub fit: Fit,
    /// The product's test build, which P4's gates drive.
    pub gate_input: GateInput,
    /// G27 settings.
    pub g27: G27,
    /// G21's product half.
    pub g21_product: G21Product,
    /// G8's prediction part.
    pub g8_predict: G8Predict,
    /// Where the owner's settings bytes wait while a gate runs its own copies.
    pub owner_settings: OwnerSettings,
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
    /// Longest wait for the product to end after `close` or a gate copy's end posts it Quit.
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

/// Loads and parses the settings: `kx-gates.toml`, then each phase's file from its folder.
pub fn load() -> Result<GatesConfig, String> {
    let file = cfgfile::path()?;
    let dir = file.parent().map(Path::to_path_buf).unwrap_or_default();
    let mut config: GatesConfig = cfgload::read(&dir)?
        .try_into()
        .map_err(|e| format!("{} ({}): {e}", dir.display(), OWN.join(", ")))?;
    config.dir = dir;
    config.text.validate()?;
    featcfg::check(&config)?;
    config.g2.check()?;
    config.g3.check()?;
    config.g12.check()?;
    config.g8.check()?;
    config.mouse_guard.check()?;
    config.gate_input.check()?;
    config.g5.check()?;
    config.g8_hook.check()?;
    fit::check(&config)?;
    p3cfg::check(&config)?;
    p5cfg::check(&config)?;
    p7cfg::check(&config)?;
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
        assert!(cfg.hand.try_secs > 0 && cfg.g5.rounds > 0);
        for app in ["explorer", "word_file"] {
            assert!(cfg.app(app).is_ok(), "missing app {app}");
        }
        assert!(cfg.probes.move_steps > 0 && cfg.g6.phase_ms > 0);
        assert!(!cfg.g25.panel_classes.is_empty());
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
