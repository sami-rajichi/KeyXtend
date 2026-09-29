//! Loads `kx-target-window.toml`: beside the running exe first, else in the crate folder.

use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;

/// File name of the settings.
pub const FILE: &str = "kx-target-window.toml";

/// The window's settings.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetConfig {
    /// Window title, which the gate runner finds the window by.
    pub title: String,
    /// Character log; loading resolves a relative path against the settings file's folder.
    pub log: PathBuf,
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

/// Why the settings could not be loaded; each message names the file.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// The file could not be read.
    #[error("{}: {source}", .file.display())]
    Read {
        /// The settings file.
        file: PathBuf,
        /// The reason.
        source: std::io::Error,
    },
    /// The file is not valid TOML, holds an unknown key, or misses one.
    #[error("{}: {source}", .file.display())]
    Parse {
        /// The settings file.
        file: PathBuf,
        /// The reason.
        source: toml::de::Error,
    },
    /// A text value is empty.
    #[error("{}: {key} must not be empty", .file.display())]
    Empty {
        /// The settings file.
        file: PathBuf,
        /// The key.
        key: &'static str,
    },
    /// A size is zero or negative.
    #[error("{}: {key} must be above zero", .file.display())]
    NotPositive {
        /// The settings file.
        file: PathBuf,
        /// The key.
        key: &'static str,
    },
}

/// Parses and checks the settings in `text`, which came from `file`; a relative log path starts at `file`'s folder.
///
/// # Errors
/// [`ConfigError::Parse`] for bad TOML or keys; [`ConfigError::Empty`] or [`ConfigError::NotPositive`] for bad values.
pub fn parse(text: &str, file: &Path) -> Result<TargetConfig, ConfigError> {
    let mut cfg: TargetConfig = toml::from_str(text).map_err(|source| ConfigError::Parse {
        file: file.to_path_buf(),
        source,
    })?;
    check(&cfg, file)?;
    let dir = file.parent().map(Path::to_path_buf).unwrap_or_default();
    cfg.log = dir.join(&cfg.log);
    Ok(cfg)
}

/// Refuses an empty title, log, font or switch and a size that is not above zero.
fn check(cfg: &TargetConfig, file: &Path) -> Result<(), ConfigError> {
    let texts = [
        ("title", cfg.title.is_empty()),
        ("log", cfg.log.as_os_str().is_empty()),
        ("font", cfg.font.is_empty()),
        ("password_arg", cfg.password_arg.is_empty()),
    ];
    if let Some((key, _)) = texts.iter().find(|(_, empty)| *empty) {
        return Err(ConfigError::Empty {
            file: file.to_path_buf(),
            key,
        });
    }
    let sizes = [
        ("font_px", cfg.font_px),
        ("width_px", cfg.width_px),
        ("height_px", cfg.height_px),
    ];
    if let Some((key, _)) = sizes.iter().find(|(_, v)| *v <= 0) {
        return Err(ConfigError::NotPositive {
            file: file.to_path_buf(),
            key,
        });
    }
    Ok(())
}

/// The settings file `name` to read: beside the exe when `beside_exists`, else in `crate_dir`.
fn pick(name: &str, exe_dir: Option<&Path>, crate_dir: &Path, beside_exists: bool) -> PathBuf {
    match exe_dir {
        Some(dir) if beside_exists => dir.join(name),
        _ => crate_dir.join(name),
    }
}

/// The settings file `name` for a run of `exe`: beside it when it holds `name`, else in `crate_dir`.
#[must_use]
pub fn locate_for(exe: Option<&Path>, name: &str, crate_dir: &Path) -> PathBuf {
    let exe_dir = exe.and_then(Path::parent);
    let beside = exe_dir.is_some_and(|dir| dir.join(name).is_file());
    pick(name, exe_dir, crate_dir, beside)
}

/// The settings file `name` for this run: beside the running exe, else in `crate_dir`.
///
/// Each tool passes its own `env!("CARGO_MANIFEST_DIR")`, so they share the rule.
#[must_use]
pub fn locate(name: &str, crate_dir: &Path) -> PathBuf {
    locate_for(std::env::current_exe().ok().as_deref(), name, crate_dir)
}

/// Where the settings file is: beside the running exe, else in the crate folder.
#[must_use]
pub fn path() -> PathBuf {
    locate(FILE, Path::new(env!("CARGO_MANIFEST_DIR")))
}

/// Reads, parses and checks the settings in `file`.
///
/// # Errors
/// [`ConfigError::Read`] when the file cannot be read, else what [`parse`] returns.
pub fn load_from(file: &Path) -> Result<TargetConfig, ConfigError> {
    let text = std::fs::read_to_string(file).map_err(|source| ConfigError::Read {
        file: file.to_path_buf(),
        source,
    })?;
    parse(&text, file)
}

/// Reads, parses and checks the settings at [`path`].
///
/// # Errors
/// What [`load_from`] returns.
pub fn load() -> Result<TargetConfig, ConfigError> {
    load_from(&path())
}

#[cfg(test)]
mod tests;
