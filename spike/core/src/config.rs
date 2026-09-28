//! Loads `spike.toml`: next to the running exe first, else the spike folder.

use std::path::{Path, PathBuf};

use serde::Deserialize;

/// File name of the spike settings.
const FILE: &str = "spike.toml";
/// Error file a face writes in TEMP; `{face}` becomes the face name.
const ERROR_FILE: &str = "kx-spike-{face}-error.txt";
/// Placeholder for the face name in `ERROR_FILE`.
const FACE: &str = "{face}";

/// All spike settings.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpikeConfig {
    /// Keyboard block and sizes.
    pub keyboard: KeyboardConfig,
    /// The recording target window.
    pub target: TargetConfig,
    /// The tools row, the selection pill, quick-fill and snip.
    pub tools: ToolsConfig,
    /// Voice: the worker, its engines and the caption bar.
    pub voice: crate::voicecfg::VoiceConfig,
    /// The stage-2 keyboard's geometry.
    pub layout: crate::kbgeom::LayoutConfig,
    /// Keyboard size steps, presets and limits.
    pub size: crate::sizer::SizeConfig,
    /// Theme, mode, frosted plate and tooltip delay.
    pub look: crate::lookcfg::LookConfig,
    /// How each named key looks and what side keys do.
    pub keys: crate::facecfg::KeysConfig,
    /// The top bar and the test strip.
    pub bar: crate::facecfg::BarConfig,
    /// Where fonts and icons are.
    pub assets: crate::facecfg::AssetsConfig,
    /// Short language names and the space bar's joiner.
    pub lang: crate::langinfo::LangConfig,
    /// The Arabic test panel (gate G7).
    pub panel: crate::panelcfg::PanelConfig,
    /// Folder the file was read from; relative paths start here.
    #[serde(skip)]
    pub dir: PathBuf,
}

/// Keyboard block and sizes.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyboardConfig {
    /// Window title per face (`slint`, `qt`, `tauri`).
    pub titles: std::collections::BTreeMap<String, String>,
    /// One key unit in logical pixels.
    pub key_px: f32,
    /// Gap between keys in logical pixels.
    pub gap_px: f32,
    /// Label font size in logical pixels.
    pub font_px: f32,
    /// Wait after the window appears before guarding it, and between retries, in ms.
    pub guard_delay_ms: u64,
    /// Guard tries before giving up while no window of ours is visible yet.
    pub guard_tries: u32,
    /// How often labels follow the foreground layout, in ms.
    pub relabel_ms: u64,
    /// Rows of scan codes; `0xE0xx` marks an extended key.
    pub rows: Vec<Vec<u32>>,
    /// `[scan code, width]` pairs; other keys are `layout::DEFAULT_WIDTH` wide.
    pub widths: Vec<(u32, f32)>,
}

/// The recording target window.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetConfig {
    /// Window title.
    pub title: String,
    /// Character log, relative to the settings folder.
    pub log: String,
    /// Text font name.
    pub font: String,
    /// Text font height in pixels.
    pub font_px: i32,
    /// Window width in pixels.
    pub width_px: i32,
    /// Window height in pixels.
    pub height_px: i32,
    /// Command-line switch that makes the box a single-line password box.
    pub password_arg: String,
}

/// Bytes in one MiB.
pub const MIB: usize = 1 << 20;

/// The tools row, the selection pill, quick-fill and snip (stage 1b).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolsConfig {
    /// Button labels.
    pub labels: ToolLabels,
    /// Title of the pill window, so the harness can find it.
    pub pill_title: String,
    /// Title of the snip overlay.
    pub overlay_title: String,
    /// How often the selection watcher looks at the focused text, in ms.
    pub selection_poll_ms: u64,
    /// Pill width, height and gap from the text, in logical pixels.
    pub pill_px: [f32; 3],
    /// Virtual keys the pill's Copy button presses, in order.
    pub copy_keys: Vec<u16>,
    /// Text of the Windows Hello prompt.
    pub hello_message: String,
    /// Fake user name that Fill types after Hello says yes.
    pub test_user: String,
    /// Fake password that Fill types after Hello says yes.
    pub test_password: String,
    /// Wait after giving the app in front its focus back, before typing, in ms.
    pub fill_settle_ms: u64,
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

/// Labels of the tool buttons and the pill.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolLabels {
    /// Types the test user name.
    pub fill_user: String,
    /// Types the test password.
    pub fill_password: String,
    /// Starts a snip.
    pub snip: String,
    /// Starts and stops recording.
    pub mic: String,
    /// The pill's copy button.
    pub copy: String,
}

/// The tool buttons, in row order; the faces and the harness share this order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolButton {
    /// Types the test user name after Hello.
    FillUser,
    /// Types the test password after Hello.
    FillPassword,
    /// Starts a snip.
    Snip,
    /// Starts and stops recording.
    Mic,
}

impl ToolButton {
    /// Every button, in row order.
    pub const ALL: [ToolButton; 4] = [Self::FillUser, Self::FillPassword, Self::Snip, Self::Mic];

    /// The button at row position `i`.
    pub fn at(i: usize) -> Option<ToolButton> {
        Self::ALL.get(i).copied()
    }

    /// Its row position.
    pub fn index(self) -> usize {
        Self::ALL
            .iter()
            .position(|&b| b == self)
            .unwrap_or_default()
    }
}

impl ToolLabels {
    /// The label of button `b`.
    pub fn of(&self, b: ToolButton) -> &str {
        match b {
            ToolButton::FillUser => &self.fill_user,
            ToolButton::FillPassword => &self.fill_password,
            ToolButton::Snip => &self.snip,
            ToolButton::Mic => &self.mic,
        }
    }
}

impl ToolsConfig {
    /// The folder snips go to, under TEMP.
    pub fn snip_dir(&self) -> PathBuf {
        std::env::temp_dir().join(&self.snip_folder)
    }

    /// The screen copy cap in bytes.
    pub fn shot_cap(&self) -> usize {
        self.shot_cap_mb.saturating_mul(MIB)
    }
}

impl SpikeConfig {
    /// The title for `face`, or the face name itself.
    pub fn title(&self, face: &str) -> String {
        self.keyboard
            .titles
            .get(face)
            .cloned()
            .unwrap_or_else(|| face.to_string())
    }

    /// `rel` resolved against the settings folder.
    pub fn resolve(&self, rel: &str) -> PathBuf {
        self.dir.join(rel)
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

/// Reads settings file `name` and parses it with `parse`; an error starts with the file's path.
pub fn read_settings<T>(
    name: &str,
    parse: impl FnOnce(&str) -> Result<T, String>,
) -> Result<T, String> {
    let file = settings_path(name);
    let at = |e: String| format!("{}: {e}", file.display());
    let text = std::fs::read_to_string(&file).map_err(|e| at(e.to_string()))?;
    parse(&text).map_err(at)
}

/// Prints `err` and saves it in TEMP, since release faces have no console to show it.
pub fn report_error(face: &str, err: &str) {
    eprintln!("{err}");
    let file = std::env::temp_dir().join(ERROR_FILE.replace(FACE, face));
    let _ = std::fs::write(file, err);
}

/// Loads and parses the settings.
pub fn load() -> Result<SpikeConfig, String> {
    let file = settings_path(FILE);
    let text = std::fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
    let at = |e: String| format!("{}: {e}", file.display());
    let mut config: SpikeConfig = toml::from_str(&text).map_err(|e| at(e.to_string()))?;
    config.layout.check().map_err(at)?;
    config.size.check().map_err(at)?;
    config.look.check().map_err(at)?;
    config.voice.caption.check().map_err(at)?;
    config.panel.check().map_err(at)?;
    crate::panelcfg::check_title(&config).map_err(at)?;
    crate::facecfg::check_keys(&config.keys, &config.layout).map_err(at)?;
    crate::facecfg::check_bar(&config.bar, config.size.presets.len()).map_err(at)?;
    config.dir = file.parent().map(Path::to_path_buf).unwrap_or_default();
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_buttons_keep_their_row_order() {
        for (i, b) in ToolButton::ALL.iter().enumerate() {
            assert_eq!(ToolButton::at(i), Some(*b));
            assert_eq!(b.index(), i);
        }
        assert_eq!(ToolButton::at(ToolButton::ALL.len()), None);
    }

    #[test]
    fn spike_toml_loads_the_tools() {
        let cfg = load().expect("spike.toml loads");
        assert_eq!(cfg.tools.labels.of(ToolButton::Snip), cfg.tools.labels.snip);
        assert_eq!(cfg.tools.labels.of(ToolButton::Mic), cfg.tools.labels.mic);
        assert_eq!(
            ToolButton::Mic.index(),
            ToolButton::ALL.len() - 1,
            "Mic is last"
        );
        assert!(cfg.tools.selection_poll_ms > 0 && !cfg.tools.copy_keys.is_empty());
        assert!(cfg.tools.snip_edge_px > 0.0 && !cfg.tools.snip_hint.is_empty());
        let size = &cfg.tools.snip_size;
        assert!(size.contains("{w}") && size.contains("{h}"), "{size}");
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
