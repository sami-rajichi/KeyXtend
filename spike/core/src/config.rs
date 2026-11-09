//! Loads `spike.toml`: next to the running exe first, else the spike folder.

use std::path::{Path, PathBuf};

use serde::Deserialize;

/// File name of the spike settings.
pub const FILE: &str = "spike.toml";
/// The test face's name: the face argument of kx-gates.
pub const FACE: &str = "qt";

/// The spike settings its later-phase parts and the gates read; the P3 sections moved to the product.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpikeConfig {
    /// The face's title and stage-1 geometry, which the gates and the tools row aim with.
    pub keyboard: KeyboardConfig,
    /// The tools row and snip.
    pub tools: ToolsConfig,
    /// Voice: the worker, its engines and the caption bar.
    pub voice: crate::voicecfg::VoiceConfig,
    /// The Arabic test panel (gate G7).
    pub panel: crate::panelcfg::PanelConfig,
}

/// The face's title and stage-1 geometry; the keyboard itself is kx-mod-keyboard's since P3.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyboardConfig {
    /// The face's window title.
    pub title: String,
    /// One key unit in logical pixels.
    pub key_px: f32,
    /// Gap between keys in logical pixels.
    pub gap_px: f32,
    /// Label font size in logical pixels.
    pub font_px: f32,
    /// Rows of scan codes; `0xE0xx` marks an extended key.
    pub rows: Vec<Vec<u32>>,
    /// `[scan code, width]` pairs; other keys are `place::DEFAULT_WIDTH` wide.
    pub widths: Vec<(u32, f32)>,
}

/// Bytes in one MiB.
pub const MIB: usize = 1 << 20;

/// The tools row and snip (stage 1b).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolsConfig {
    /// Button labels.
    pub labels: ToolLabels,
    /// How many buttons share the frozen face's tools row.
    pub slots: usize,
    /// Each button's place in that row.
    pub slot: ToolSlots,
    /// Title of the snip overlay.
    pub overlay_title: String,
    /// Folder under TEMP for snips and the frozen screen.
    pub snip_folder: String,
    /// Largest screen copy we make or read, in MiB.
    pub shot_cap_mb: usize,
    /// Width of the snip region's edge, in logical pixels.
    pub snip_edge_px: f32,
    /// What the snip overlay tells a screen reader to do; it is also shown at the top.
    pub snip_hint: String,
    /// The region's size tag, with `{w}` and `{h}` in physical pixels.
    pub snip_size: String,
}

/// Labels of the tool buttons.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolLabels {
    /// Starts a snip.
    pub snip: String,
    /// Starts and stops recording.
    pub mic: String,
}

/// Each tool button's place in the frozen face's row, 0 on the left.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolSlots {
    /// The snip button's place.
    pub snip: usize,
    /// The mic button's place.
    pub mic: usize,
}

/// The tool buttons, in row order; the faces and kx-gates share this order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolButton {
    /// Starts a snip.
    Snip,
    /// Starts and stops recording.
    Mic,
}

impl ToolButton {
    /// Every button, in row order.
    pub const ALL: [ToolButton; 2] = [Self::Snip, Self::Mic];

    /// The button at row position `i`.
    pub fn at(i: usize) -> Option<ToolButton> {
        Self::ALL.get(i).copied()
    }
}

impl ToolLabels {
    /// The label of button `b`.
    pub fn of(&self, b: ToolButton) -> &str {
        match b {
            ToolButton::Snip => &self.snip,
            ToolButton::Mic => &self.mic,
        }
    }
}

impl ToolSlots {
    /// The place of button `b`.
    pub fn of(&self, b: ToolButton) -> usize {
        match b {
            ToolButton::Snip => self.snip,
            ToolButton::Mic => self.mic,
        }
    }
}

impl ToolsConfig {
    /// Refuses a button outside the row, or two buttons in one place.
    pub fn check(&self) -> Result<(), String> {
        let mut used: Vec<usize> = ToolButton::ALL.iter().map(|&b| self.slot.of(b)).collect();
        let inside = used.iter().all(|&s| s < self.slots);
        used.sort_unstable();
        used.dedup();
        if inside && used.len() == ToolButton::ALL.len() {
            return Ok(());
        }
        Err(format!(
            "tools.slot: each button needs its own place below tools.slots ({})",
            self.slots
        ))
    }

    /// The folder snips go to, under TEMP.
    pub fn snip_dir(&self) -> PathBuf {
        std::env::temp_dir().join(&self.snip_folder)
    }

    /// The screen copy cap in bytes.
    pub fn shot_cap(&self) -> usize {
        self.shot_cap_mb.saturating_mul(MIB)
    }
}

/// Where the settings file `name` is: beside the exe, else in the spike folder.
pub fn settings_path(name: &str) -> PathBuf {
    let beside = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join(name)));
    match beside {
        Some(p) if p.is_file() => p,
        _ => Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(name),
    }
}

/// Loads and parses the settings.
pub fn load() -> Result<SpikeConfig, String> {
    let file = settings_path(FILE);
    let text = std::fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
    let at = |e: String| format!("{}: {e}", file.display());
    let config: SpikeConfig = toml::from_str(&text).map_err(|e| at(e.to_string()))?;
    config.tools.check().map_err(at)?;
    config.voice.caption.check().map_err(at)?;
    config.panel.check().map_err(at)?;
    crate::panelcfg::check_titles(&config).map_err(at)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_buttons_keep_their_row_order() {
        for (i, b) in ToolButton::ALL.iter().enumerate() {
            assert_eq!(ToolButton::at(i), Some(*b));
        }
        assert_eq!(ToolButton::at(ToolButton::ALL.len()), None);
    }

    #[test]
    fn snip_and_mic_keep_the_frozen_faces_last_two_slots() {
        let t = load().expect("spike.toml loads").tools;
        let slots = (t.slot.of(ToolButton::Snip), t.slot.of(ToolButton::Mic));
        assert_eq!((slots, t.slots), ((2, 3), 4), "quick-fill's two came first");
    }

    #[test]
    fn a_slot_past_the_row_or_shared_by_two_buttons_is_refused() {
        let good = load().expect("spike.toml loads").tools;
        assert_eq!(good.check(), Ok(()));
        let spoils: [fn(&mut ToolsConfig); 3] = [
            |t| t.slot.mic = t.slots,
            |t| t.slot.snip = t.slot.mic,
            |t| t.slots = 0,
        ];
        for (i, spoil) in spoils.iter().enumerate() {
            let mut t = good.clone();
            spoil(&mut t);
            let e = t.check().expect_err(&format!("case {i} passed"));
            assert!(e.contains("tools.slot"), "case {i}: {e}");
        }
    }

    #[test]
    fn spike_toml_loads_the_tools() {
        let cfg = load().expect("spike.toml loads");
        assert_eq!(cfg.tools.labels.of(ToolButton::Snip), cfg.tools.labels.snip);
        assert_eq!(cfg.tools.labels.of(ToolButton::Mic), cfg.tools.labels.mic);
        assert_eq!(
            ToolButton::ALL.last(),
            Some(&ToolButton::Mic),
            "Mic is last"
        );
        assert!(cfg.tools.snip_edge_px > 0.0 && !cfg.tools.snip_hint.is_empty());
        let size = &cfg.tools.snip_size;
        assert!(size.contains("{w}") && size.contains("{h}"), "{size}");
    }

    #[test]
    fn the_caption_uses_the_mock_ups_icons() {
        let cfg = load().expect("spike.toml loads");
        let c = &cfg.voice.caption;
        assert_eq!(
            (c.busy_icon.as_str(), c.words_icon.as_str()),
            ("audio-lines", "captions")
        );
    }

    #[test]
    fn the_shot_cap_is_in_megabytes() {
        let mut t = load().expect("spike.toml loads").tools;
        t.shot_cap_mb = 2;
        assert_eq!(t.shot_cap(), 2 * 1024 * 1024);
        t.shot_cap_mb = usize::MAX;
        assert_eq!(t.shot_cap(), usize::MAX, "a huge value saturates");
    }
}
