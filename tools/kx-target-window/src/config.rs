//! Loads `kx-target-window.toml`: beside the running exe first, else in the crate folder.

use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;

/// File name of the settings.
pub const FILE: &str = "kx-target-window.toml";
/// Windows' hung-window mark, in ms: an app that reads no message for this long counts as hung, and Windows swaps in
/// its ghost window (`IsHungAppWindow`).
pub const HUNG_MARK_MS: u64 = 5_000;
/// The most characters a user can type in an edit box before `EM_LIMITTEXT` raises it (Microsoft's `EM_LIMITTEXT`
/// docs); the fill stays within it, so typing into a filled box still works.
pub const TYPE_MAX: usize = 32_767;

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
    /// The text box's accessible name, which Narrator and NVDA read.
    pub edit_name: String,
    /// Window width in pixels.
    pub width_px: i32,
    /// Window height in pixels.
    pub height_px: i32,
    /// Command-line switch that makes the box a single-line password box.
    pub password_arg: String,
    /// Command-line switch, followed by an installed layout id, that switches the whole session's input language to
    /// that layout.
    pub layout_arg: String,
    /// Longest a gate's hang message makes the window sleep, in ms.
    pub hang_max_ms: u64,
    /// Command-line switch that fills the box with long numbered lines, unwrapped, so it scrolls both ways.
    pub fill_arg: String,
    /// Lines the fill switch puts in the box.
    pub fill_lines: u32,
    /// Characters in each of those lines.
    pub fill_cols: u32,
    /// Command-line switch that makes the window DPI-unaware, as an old app is.
    pub unaware_arg: String,
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
    /// Two switches are the same text, so one would always turn on the other, such as every `--layout` run being a
    /// password box.
    #[error("{}: password_arg, layout_arg, fill_arg and unaware_arg must all differ", .file.display())]
    SameSwitch {
        /// The settings file.
        file: PathBuf,
    },
    /// The hang cap reaches Windows' hung-window mark, so a gate's hang would ghost the window.
    #[error("{}: hang_max_ms must stay below Windows' {HUNG_MARK_MS} ms hung-window mark", .file.display())]
    HangPastHungMark {
        /// The settings file.
        file: PathBuf,
    },
    /// The fill holds more characters than a user can type in the box, so typing into it would stop.
    #[error("{}: fill_lines times fill_cols plus line ends must stay within {TYPE_MAX} characters", .file.display())]
    FillTooLong {
        /// The settings file.
        file: PathBuf,
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

/// Refuses an empty title, log, font, name or switch, two switches with one text, a hang cap at Windows' hung-window
/// mark, a size that is not above zero, and a fill too long to type into.
fn check(cfg: &TargetConfig, file: &Path) -> Result<(), ConfigError> {
    let file = file.to_path_buf();
    if let Some(key) = empty_key(cfg) {
        return Err(ConfigError::Empty { file, key });
    }
    let switches = [
        &cfg.password_arg,
        &cfg.layout_arg,
        &cfg.fill_arg,
        &cfg.unaware_arg,
    ];
    let twice = switches
        .iter()
        .enumerate()
        .any(|(i, s)| switches[..i].contains(s));
    if twice {
        return Err(ConfigError::SameSwitch { file });
    }
    if cfg.hang_max_ms >= HUNG_MARK_MS {
        return Err(ConfigError::HangPastHungMark { file });
    }
    let sizes = [
        ("font_px", i64::from(cfg.font_px)),
        ("width_px", i64::from(cfg.width_px)),
        ("height_px", i64::from(cfg.height_px)),
        ("fill_lines", i64::from(cfg.fill_lines)),
        ("fill_cols", i64::from(cfg.fill_cols)),
    ];
    if let Some(&(key, _)) = sizes.iter().find(|(_, v)| *v <= 0) {
        return Err(ConfigError::NotPositive { file, key });
    }
    let each = u64::from(cfg.fill_cols) + crate::fill::LINE_END.len() as u64;
    if u64::from(cfg.fill_lines) * each > TYPE_MAX as u64 {
        return Err(ConfigError::FillTooLong { file });
    }
    Ok(())
}

/// The first text key whose value is empty.
fn empty_key(cfg: &TargetConfig) -> Option<&'static str> {
    let texts = [
        ("title", cfg.title.is_empty()),
        ("log", cfg.log.as_os_str().is_empty()),
        ("font", cfg.font.is_empty()),
        ("edit_name", cfg.edit_name.is_empty()),
        ("password_arg", cfg.password_arg.is_empty()),
        ("layout_arg", cfg.layout_arg.is_empty()),
        ("fill_arg", cfg.fill_arg.is_empty()),
        ("unaware_arg", cfg.unaware_arg.is_empty()),
    ];
    texts.iter().find(|(_, empty)| *empty).map(|(key, _)| *key)
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
