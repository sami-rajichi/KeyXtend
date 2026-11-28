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
    /// The frozen face's title, which the gates find it by.
    pub keyboard: KeyboardConfig,
    /// The cap on the gates' screen copies.
    pub shot: ShotConfig,
    /// The Arabic test panel (gate G7).
    pub panel: crate::panelcfg::PanelConfig,
}

/// The frozen face's title; the keyboard itself is kx-mod-keyboard's since P3.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyboardConfig {
    /// The face's window title.
    pub title: String,
}

/// Bytes in one MiB.
pub const MIB: usize = 1 << 20;

/// The cap on the gates' screen copies: G1's desktop screenshot and G6's pictures.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShotConfig {
    /// Largest screen copy we make, in MiB.
    pub cap_mb: usize,
}

impl ShotConfig {
    /// The screen copy cap in bytes.
    pub fn cap(&self) -> usize {
        self.cap_mb.saturating_mul(MIB)
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
    config.panel.check().map_err(at)?;
    crate::panelcfg::check_titles(&config).map_err(at)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shot_cap_is_in_megabytes() {
        let mut s = load().expect("spike.toml loads").shot;
        assert!(s.cap() > 0, "spike.toml sets a cap");
        s.cap_mb = 2;
        assert_eq!(s.cap(), 2 * 1024 * 1024);
        s.cap_mb = usize::MAX;
        assert_eq!(s.cap(), usize::MAX, "a huge value saturates");
    }
}
