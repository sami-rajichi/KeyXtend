//! `cargo xtask stage`: builds the app in release and stages it with only the Qt files it loads, then checks the stage.

mod check;
mod pe;
mod qt;

use std::path::{Path, PathBuf};

use serde::Deserialize;

use self::qt::QtDirs;
use super::config::{self, DevSetup, first_failing};
use super::{DevError, paths};
use crate::config::ConfigError;
use crate::workspace::{self, LoadError};

/// The section name under `workspace.metadata`.
const SECTION: &str = "stage";
/// Cargo's arguments that build one package in release with the locked dependencies.
const BUILD: [&str; 4] = ["build", "--release", "--locked", "--package"];
/// Cargo's output folder for release builds.
const RELEASE_DIR: &str = "release";

/// Settings for `cargo xtask stage`, from `[workspace.metadata.stage]`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StageConfig {
    /// The app package built in release; its program has the same name.
    pub app: String,
    /// Folder, under the workspace root, holding the stage.
    pub out_dir: String,
    /// Largest stage, in MB.
    pub max_mb: u64,
    /// Env var naming Qt's qmake.
    pub qmake_env: String,
    /// Qt libraries, from Qt's bin folder, staged beside the program.
    pub libs: Vec<String>,
    /// Qt plugins, from Qt's plugins folder, staged in the same subfolders.
    pub plugins: Vec<String>,
    /// Qt QML module files, from Qt's qml folder, staged in `qml_dir`.
    pub qml: Vec<String>,
    /// Folder beside the program that holds Qt's QML modules.
    pub qml_dir: String,
    /// Folder names the stage must never hold.
    pub banned: Vec<String>,
    /// Name prefixes of Windows API sets, which never load from a folder.
    pub api_sets: Vec<String>,
    /// Windows' system folder, under the Windows folder.
    pub system_dir: String,
}

impl StageConfig {
    /// Rejects a zero cap, empty lists and names or paths that are not plain.
    pub(crate) fn validate(&self) -> Result<(), ConfigError> {
        if self.max_mb == 0 {
            return Err(ConfigError::ZeroValue("max_mb"));
        }
        let texts = [("qmake_env", &self.qmake_env)];
        first_failing(&texts, |v| !v.trim().is_empty(), ConfigError::EmptyValue)?;
        let names = [("app", &self.app), ("qml_dir", &self.qml_dir)];
        first_failing(&names, paths::is_plain_name, ConfigError::NotPlain)?;
        let rel = [("out_dir", &self.out_dir), ("system_dir", &self.system_dir)];
        first_failing(&rel, paths::is_relative_plain, ConfigError::NotRelative)?;
        self.check_lists()
    }

    /// Every list must hold values: plain names, relative paths or, for the API sets, any text.
    fn check_lists(&self) -> Result<(), ConfigError> {
        let names = [("libs", &self.libs), ("banned", &self.banned)];
        for (name, list) in names {
            each(name, list, paths::is_plain_name, ConfigError::NotPlain)?;
        }
        let rel = [("plugins", &self.plugins), ("qml", &self.qml)];
        for (name, list) in rel {
            each(
                name,
                list,
                paths::is_relative_plain,
                ConfigError::NotRelative,
            )?;
        }
        let filled = |v: &str| !v.trim().is_empty();
        each("api_sets", &self.api_sets, filled, ConfigError::EmptyValue)
    }

    /// The app's program file name.
    pub(crate) fn program(&self) -> String {
        format!("{}{}", self.app, std::env::consts::EXE_SUFFIX)
    }
}

/// Fails on an empty `list`, or with `error` on its first value that fails `ok`.
fn each(
    name: &'static str,
    list: &[String],
    ok: impl Fn(&str) -> bool,
    error: impl Fn(&'static str) -> ConfigError,
) -> Result<(), ConfigError> {
    if list.is_empty() {
        return Err(ConfigError::EmptyList(name));
    }
    if list.iter().any(|v| !ok(v)) {
        Err(error(name))
    } else {
        Ok(())
    }
}

/// One staged file: where it comes from, and its path inside the stage.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Copy {
    /// The source file.
    pub from: PathBuf,
    /// Its path, relative to the stage.
    pub to: PathBuf,
}

/// Every file the stage holds: the program `exe`, then Qt's libraries, plugins and QML files.
pub(crate) fn plan(config: &StageConfig, qt: &QtDirs, exe: &Path) -> Vec<Copy> {
    let at = |base: &Path, rel: &str| paths::join_rel(base.to_path_buf(), rel);
    let qml_to = PathBuf::from(&config.qml_dir);
    let program = Copy {
        from: exe.to_path_buf(),
        to: PathBuf::from(config.program()),
    };
    let libs = config.libs.iter().map(|l| Copy {
        from: qt.bins.join(l),
        to: PathBuf::from(l),
    });
    let plugins = config.plugins.iter().map(|p| Copy {
        from: at(&qt.plugins, p),
        to: at(Path::new(""), p),
    });
    let qml = config.qml.iter().map(|q| Copy {
        from: at(&qt.qml, q),
        to: at(&qml_to, q),
    });
    std::iter::once(program)
        .chain(libs)
        .chain(plugins)
        .chain(qml)
        .collect()
}

/// Empties `stage`, then copies each file into it; a missing source is named.
pub(crate) fn copy(stage: &Path, files: &[Copy]) -> Result<(), DevError> {
    if stage.exists() {
        std::fs::remove_dir_all(stage)?;
    }
    for file in files {
        let to = stage.join(&file.to);
        if let Some(parent) = to.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(&file.from, &to)
            .map_err(|e| DevError::Io(format!("{}: {e}", paths::text(&file.from))))?;
    }
    Ok(())
}

/// Loads and validates the stage settings from `cargo metadata` JSON.
pub(crate) fn load(json: &str) -> Result<StageConfig, LoadError> {
    let (_, config): (PathBuf, StageConfig) = workspace::root_and_section(json, SECTION)?;
    config.validate().map_err(workspace::config_error)?;
    Ok(config)
}

/// Builds the app in release with cargo, showing cargo's own progress.
fn build(config: &StageConfig) -> Result<(), DevError> {
    let step = BUILD.join(" ");
    let status = crate::cargo::command()
        .args(BUILD)
        .arg(&config.app)
        .status()
        .map_err(|e| DevError::Spawn {
            program: step.clone(),
            reason: e.to_string(),
        })?;
    if status.success() {
        Ok(())
    } else {
        Err(DevError::Failed {
            step,
            detail: status.to_string(),
        })
    }
}

/// Runs `cargo xtask stage`: builds the app, stages it in `<out_dir>/<app>`, checks the stage and reports its size.
pub(crate) fn run() -> Result<(), DevError> {
    let json = workspace::metadata_json()?;
    let (setup, config) = (config::load(&json)?, load(&json)?);
    let env = |key: &str| std::env::var(key).ok();
    let qt = qt::find(&config, env)?;
    build(&config)?;
    let exe = workspace::target_dir(&json)?
        .join(RELEASE_DIR)
        .join(config.program());
    let stage = setup.path(&config.out_dir).join(&config.app);
    copy(&stage, &plan(&config, &qt, &exe))?;
    let system = paths::join_rel(setup.windows(env)?, &config.system_dir);
    let summary = check::check(&stage, &config, pe::read, |lib| system.join(lib).is_file())?;
    report(&setup, &stage, &config, &summary);
    Ok(())
}

/// Prints where the stage is, its files and its size against the cap.
fn report(setup: &DevSetup, stage: &Path, config: &StageConfig, summary: &check::Summary) {
    let rel = stage.strip_prefix(&setup.root).unwrap_or(stage);
    println!("Staged {}: {} files.", paths::text(rel), summary.files);
    println!("Size: {} MB of {} MB.", summary.mb(), config.max_mb);
}

#[cfg(test)]
mod tests;
