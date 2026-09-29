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
///
/// Pure, so each tool with its own settings file shares the rule.
#[must_use]
pub fn pick(name: &str, exe_dir: Option<&Path>, crate_dir: &Path, beside_exists: bool) -> PathBuf {
    match exe_dir {
        Some(dir) if beside_exists => dir.join(name),
        _ => crate_dir.join(name),
    }
}

/// Where the settings file is: beside the running exe, else in the crate folder.
#[must_use]
pub fn path() -> PathBuf {
    let exe = std::env::current_exe().ok();
    let exe_dir = exe.as_deref().and_then(Path::parent);
    let beside = exe_dir.is_some_and(|dir| dir.join(FILE).is_file());
    pick(FILE, exe_dir, Path::new(env!("CARGO_MANIFEST_DIR")), beside)
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
mod tests {
    use super::*;

    /// The shipped settings file.
    const SHIPPED: &str = include_str!("../kx-target-window.toml");

    /// A stand-in settings file that lives in `cfg/dir`.
    fn file() -> &'static Path {
        Path::new("cfg/dir/kx-target-window.toml")
    }

    /// The shipped file with the line of `key` set to `value`, a TOML value.
    fn with(key: &str, value: &str) -> String {
        let start = format!("{key} =");
        let mut hit = false;
        let lines: Vec<String> = SHIPPED
            .lines()
            .map(|line| {
                hit |= line.starts_with(&start);
                if line.starts_with(&start) {
                    format!("{start} {value}")
                } else {
                    line.to_string()
                }
            })
            .collect();
        assert!(hit, "the shipped file has no {key}");
        lines.join("\n")
    }

    #[test]
    fn the_shipped_file_parses() {
        let cfg = parse(SHIPPED, file()).expect("the shipped file parses");
        assert!(!cfg.title.is_empty() && !cfg.password_arg.is_empty());
        assert!(cfg.font_px > 0 && cfg.width_px > 0 && cfg.height_px > 0);
    }

    #[test]
    fn an_unknown_key_fails() {
        let text = format!("{SHIPPED}\nsurprise = 1\n");
        let err = parse(&text, file()).expect_err("unknown key");
        assert!(matches!(err, ConfigError::Parse { .. }), "{err}");
        assert!(err.to_string().contains("surprise"), "{err}");
    }

    #[test]
    fn a_missing_key_fails() {
        let text: String = SHIPPED
            .lines()
            .filter(|l| !l.starts_with("font_px"))
            .collect::<Vec<_>>()
            .join("\n");
        let err = parse(&text, file()).expect_err("missing key");
        assert!(matches!(err, ConfigError::Parse { .. }), "{err}");
    }

    #[test]
    fn an_empty_text_value_fails() {
        for key in ["title", "log", "font", "password_arg"] {
            let err = parse(&with(key, "\"\""), file()).expect_err(key);
            assert!(
                matches!(err, ConfigError::Empty { key: k, .. } if k == key),
                "{err}"
            );
        }
    }

    #[test]
    fn a_zero_or_negative_size_fails() {
        for key in ["font_px", "width_px", "height_px"] {
            for bad in ["0", "-5"] {
                let err = parse(&with(key, bad), file()).expect_err(key);
                let named = matches!(err, ConfigError::NotPositive { key: k, .. } if k == key);
                assert!(named, "{key} = {bad}: {err}");
            }
        }
    }

    #[test]
    fn a_relative_log_starts_at_the_files_folder() {
        let cfg = parse(&with("log", "\"out/x.log\""), file()).expect("parses");
        assert_eq!(cfg.log, Path::new("cfg/dir/out/x.log"));
    }

    #[test]
    fn an_absolute_log_stays_as_it_is() {
        let abs = std::env::temp_dir().join("abs.log");
        let cfg = parse(&with("log", &format!("'{}'", abs.display())), file()).expect("parses");
        assert_eq!(cfg.log, abs);
    }

    #[test]
    fn every_error_names_the_file() {
        let bad = [
            with("title", "\"\""),
            with("width_px", "0"),
            format!("{SHIPPED}\nsurprise = 1\n"),
        ];
        for text in bad {
            let err = parse(&text, file()).expect_err("bad settings");
            assert!(err.to_string().contains("kx-target-window.toml"), "{err}");
        }
    }

    #[test]
    fn the_file_beside_the_exe_wins_when_it_exists() {
        let (exe, krate) = (Path::new("exe/dir"), Path::new("crate/dir"));
        assert_eq!(pick(FILE, Some(exe), krate, true), exe.join(FILE));
        assert_eq!(pick(FILE, Some(exe), krate, false), krate.join(FILE));
        assert_eq!(pick(FILE, None, krate, true), krate.join(FILE));
        assert_eq!(pick(FILE, None, krate, false), krate.join(FILE));
        assert_eq!(
            pick("other.toml", Some(exe), krate, true),
            exe.join("other.toml")
        );
    }

    #[test]
    fn a_test_run_loads_the_file_from_the_crate_folder() {
        let crate_file = Path::new(env!("CARGO_MANIFEST_DIR")).join(FILE);
        assert_eq!(path(), crate_file);
        let cfg = load().expect("the crate folder's file loads");
        assert!(
            cfg.log.starts_with(env!("CARGO_MANIFEST_DIR")),
            "{:?}",
            cfg.log
        );
    }

    #[test]
    fn a_missing_file_gives_the_read_error_naming_its_path() {
        let file = Path::new("no/such/dir").join(FILE);
        let err = load_from(&file).expect_err("no such file");
        assert!(matches!(err, ConfigError::Read { .. }), "{err}");
        assert!(
            err.to_string().starts_with(&file.display().to_string()),
            "{err}"
        );
    }
}
