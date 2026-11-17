//! The stage's settings from `[workspace.metadata.stage]`, and the checks that keep them plain and complete.

use std::path::Path;

use serde::Deserialize;

use super::check;
use super::engine::Engine;
use super::voice::Voice;
use crate::config::ConfigError;
use crate::devtools::config::first_failing;
use crate::devtools::paths;

/// Settings for `cargo xtask stage`, from `[workspace.metadata.stage]`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StageConfig {
    /// The app package built in release; its program has the same name.
    pub app: String,
    /// The media worker package built in release and staged beside the app; its program has the same name.
    pub worker: String,
    /// Folder, under the workspace root, holding the stage.
    pub out_dir: String,
    /// The download budget, in MB: the staged files deflated.
    pub max_mb: u64,
    /// Env var naming Qt's qmake.
    pub qmake_env: String,
    /// The only Qt version the stage takes.
    pub qt_version: String,
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
    /// Qt's network library and the only staged libraries allowed to import it.
    pub network: Network,
    /// Name prefixes of Windows API sets, which never load from a folder.
    pub api_sets: Vec<String>,
    /// Windows' system folder, under the Windows folder.
    pub system_dir: String,
    /// The gate-only build's flag; the check refuses a program that holds its text.
    pub gate_flag: String,
    /// The worker's features in the stage build, which never hold the gate's.
    pub worker_features: Vec<String>,
    /// The worker's gate-only feature; the check refuses a program holding the sound flag only it takes.
    pub worker_gate: Gate,
    /// The speech engine's parts, staged beside the worker (ADR-0044).
    pub engine: Engine,
    /// The voice files, staged in the platform's folder beside the worker (ADR-0044).
    pub voice: Voice,
}

/// A gate-only feature; its flag is kx-ipc's `SOUND_FLAG`, so it is written once.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Gate {
    /// The feature, which no staged build turns on.
    pub feature: String,
}

/// Qt's network library and its allowed importers.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Network {
    /// The library's file name.
    pub lib: String,
    /// The staged libraries, beside the program, that may import it.
    pub by: Vec<String>,
}

impl StageConfig {
    /// Rejects a zero cap, empty lists, names or paths that are not plain, network names missing from `libs`, a worker
    /// built with its gate, and a bad engine or voice section.
    pub(crate) fn validate(&self) -> Result<(), ConfigError> {
        if self.max_mb == 0 {
            return Err(ConfigError::ZeroValue("max_mb"));
        }
        let texts = [
            ("qmake_env", &self.qmake_env),
            ("qt_version", &self.qt_version),
            ("gate_flag", &self.gate_flag),
        ];
        first_failing(&texts, |v| !v.trim().is_empty(), ConfigError::EmptyValue)?;
        let names = [
            ("app", &self.app),
            ("worker", &self.worker),
            ("qml_dir", &self.qml_dir),
            ("network", &self.network.lib),
        ];
        first_failing(&names, paths::is_plain_name, ConfigError::NotPlain)?;
        let rel = [("out_dir", &self.out_dir), ("system_dir", &self.system_dir)];
        first_failing(&rel, paths::is_relative_plain, ConfigError::NotRelative)?;
        self.check_lists()?;
        self.check_network()?;
        self.check_worker()?;
        self.engine.validate()?;
        self.voice.validate()
    }

    /// The worker's features must be named and never hold the gate's, which must be named.
    fn check_worker(&self) -> Result<(), ConfigError> {
        const GATE: &str = "worker_gate";
        let gate = [(GATE, &self.worker_gate.feature)];
        first_failing(&gate, |v| !v.trim().is_empty(), ConfigError::EmptyValue)?;
        let named = |v: &str| !v.trim().is_empty();
        each(
            "worker_features",
            &self.worker_features,
            named,
            ConfigError::BlankEntry,
        )?;
        if self.worker_features.contains(&self.worker_gate.feature) {
            return Err(ConfigError::Same("worker_features", GATE));
        }
        Ok(())
    }

    /// The network library and its importers must be in `libs`, so a rename there cannot leave the guard idle.
    fn check_network(&self) -> Result<(), ConfigError> {
        let listed = |name: &String| self.libs.iter().any(|l| check::key(l) == check::key(name));
        let mut names = std::iter::once(&self.network.lib).chain(&self.network.by);
        if names.all(listed) {
            Ok(())
        } else {
            Err(ConfigError::NotListed("network", "libs"))
        }
    }

    /// Every list must hold values: plain names, relative paths or, for the API sets, any text. Libraries and
    /// plugins must be `.dll` files, since the check reads only those.
    fn check_lists(&self) -> Result<(), ConfigError> {
        let names = [
            ("libs", &self.libs),
            ("banned", &self.banned),
            ("network", &self.network.by),
        ];
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
        let not_lib = |name: &'static str| ConfigError::NotType(name, check::LIB_EXT);
        for (name, list) in [("libs", &self.libs), ("plugins", &self.plugins)] {
            each(name, list, |v| check::is_lib(Path::new(v)), not_lib)?;
        }
        let filled = |v: &str| !v.trim().is_empty();
        each("api_sets", &self.api_sets, filled, ConfigError::EmptyValue)
    }

    /// Every gate-only flag no staged or signed program may hold: the app's own, and the sound flag its voice gate
    /// build passes the gate worker, which takes it too.
    pub(crate) fn gate_flags(&self) -> [&str; 2] {
        [self.gate_flag.as_str(), kx_ipc::SOUND_FLAG]
    }

    /// The app's program file name.
    pub(crate) fn program(&self) -> String {
        exe(&self.app)
    }

    /// The media worker's program file name.
    pub(crate) fn worker_program(&self) -> String {
        exe(&self.worker)
    }
}

/// The program file name of package `name`.
fn exe(name: &str) -> String {
    format!("{name}{}", std::env::consts::EXE_SUFFIX)
}

/// Fails on an empty `list`, or with `error` on its first value that fails `ok`.
pub(super) fn each(
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
