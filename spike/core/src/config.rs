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
pub struct SpikeConfig {
    /// Keyboard block and sizes.
    pub keyboard: KeyboardConfig,
    /// The recording target window.
    pub target: TargetConfig,
    /// Folder the file was read from; relative paths start here.
    #[serde(skip)]
    pub dir: PathBuf,
}

/// Keyboard block and sizes.
#[derive(Debug, Clone, Deserialize)]
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
    let mut config: SpikeConfig =
        toml::from_str(&text).map_err(|e| format!("{}: {e}", file.display()))?;
    config.dir = file.parent().map(Path::to_path_buf).unwrap_or_default();
    Ok(config)
}
