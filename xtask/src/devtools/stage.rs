//! `cargo xtask stage`: builds the app in release and stages it with only the Qt files it loads and the word packs,
//! then checks the stage.

mod check;
mod config;
mod error;
mod pe;
mod qt;
mod words;

pub(crate) use self::check::refuse_gate_exe;
pub(crate) use self::config::{Network, StageConfig};
pub(crate) use self::error::StageError;

use std::path::{Path, PathBuf};

use kx_module_api::ports::PACK_DIR;

use self::qt::QtDirs;
use self::words::Words;
use super::config::DevSetup;
use super::{DevError, paths};
use crate::workspace::{self, LoadError};

/// The section name under `workspace.metadata`.
const SECTION: &str = "stage";
/// Cargo's arguments that build one package in release with the locked dependencies.
const BUILD: [&str; 4] = ["build", "--release", "--locked", "--package"];
/// Cargo's output folder for release builds.
const RELEASE_DIR: &str = "release";

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

/// The stage-relative path of every file `plan` stages; the check refuses any other file outside the packs folder.
pub(crate) fn planned(config: &StageConfig) -> Vec<PathBuf> {
    let unread = QtDirs::default();
    let files = plan(config, &unread, Path::new(""));
    files.into_iter().map(|c| c.to).collect()
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

/// Copies `files` into `stage`, then runs `inspect` on it; on any failure the stage is removed, so it never looks
/// ready to install.
pub(crate) fn fill(
    stage: &Path,
    files: &[Copy],
    inspect: impl FnOnce(&Path) -> Result<check::Summary, DevError>,
) -> Result<check::Summary, DevError> {
    copy(stage, files)
        .and_then(|()| inspect(stage))
        .map_err(|err| discard(stage, err))
}

/// Removes `stage` after `err`; says so, or why it stayed.
fn discard(stage: &Path, err: DevError) -> DevError {
    match std::fs::remove_dir_all(stage) {
        Err(e) if stage.exists() => {
            DevError::Io(format!("{err}; {} stayed: {e}", paths::text(stage)))
        }
        _ => DevError::StageRemoved(Box::new(err)),
    }
}

/// Runs the stage check on the app folder `folder`, reading real imports and Windows' system folder, then the words
/// check on its packs folder.
pub(crate) fn verify(
    setup: &DevSetup,
    config: &StageConfig,
    folder: &Path,
    env: impl Fn(&str) -> Option<String>,
) -> Result<check::Summary, DevError> {
    let system = paths::join_rel(setup.windows(env)?, &config.system_dir);
    let summary = check::check(folder, config, pe::read, |lib| system.join(lib).is_file())?;
    words::check(folder, &Words::load(&setup.root)?.credits)?;
    Ok(summary)
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
    let (setup, config) = (super::config::load(&json)?, load(&json)?);
    let env = |key: &str| std::env::var(key).ok();
    let qt = qt::find(&config, env)?;
    let words = words::plan(&Words::load(&setup.root)?)?;
    build(&config)?;
    let exe = workspace::target_dir(&json)?
        .join(RELEASE_DIR)
        .join(config.program());
    let stage = setup.path(&config.out_dir).join(&config.app);
    let mut files = plan(&config, &qt, &exe);
    files.extend(words);
    let summary = fill(&stage, &files, |s| verify(&setup, &config, s, env))?;
    report(&setup, &stage, &config, &summary);
    let packs = paths::folder_size(&stage.join(PACK_DIR))?;
    println!("Word packs: {} MB of it.", check::mb(packs));
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
