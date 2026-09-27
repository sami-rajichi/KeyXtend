//! Dev-tools settings, deserialized from `[workspace.metadata.devtools]` in `Cargo.toml`.

use std::path::PathBuf;

use serde::Deserialize;

use super::{DevError, launcher, paths};
use crate::config::ConfigError;
use crate::workspace::{self, LoadError};

/// The section name under `workspace.metadata`.
const SECTION: &str = "devtools";

/// Settings for `dev-cert`, `dev-install` and `check-uiaccess`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DevToolsConfig {
    /// Subject of the self-signed test certificate.
    pub cert_subject: String,
    /// Days the test certificate stays valid.
    pub cert_days: u32,
    /// A certificate with fewer days left is replaced.
    pub cert_min_days: u32,
    /// Current-user certificate store that holds the test key.
    pub user_store: String,
    /// Output folder under the workspace root.
    pub out_dir: String,
    /// Exported public certificate file name.
    pub cert_file: String,
    /// Launcher the owner opens to trust the certificate.
    pub trust_launcher: String,
    /// Launcher the owner opens to stop trusting it.
    pub untrust_launcher: String,
    /// Manifest file extracted by `check-uiaccess`.
    pub manifest_file: String,
    /// File where the admin window leaves its error message.
    pub admin_error_file: String,
    /// Scripts folder under the workspace root.
    pub scripts_dir: String,
    /// User-level certificate script.
    pub cert_script: String,
    /// Admin-level script.
    pub admin_script: String,
    /// Env var naming the Windows folder.
    pub shell_root_env: String,
    /// PowerShell program, relative to the Windows folder.
    pub powershell: String,
    /// Folder name for test installs under Program Files.
    pub install_dir: String,
    /// Largest build folder `dev-install` copies, in MB.
    pub install_max_mb: u64,
    /// Env vars naming the folders Windows accepts for uiAccess; the first receives installs.
    pub secure_env: Vec<String>,
    /// Windows SDK bin folder.
    pub sdk_bin: String,
    /// SDK tool architecture folder.
    pub sdk_arch: String,
    /// SDK signing tool.
    pub signtool: String,
    /// SDK manifest tool.
    pub mt: String,
    /// Signature digest algorithm.
    pub digest: String,
    /// Manifest element that holds the uiAccess attribute.
    pub ui_access_tag: String,
    /// Manifest attribute that grants uiAccess.
    pub ui_access_attr: String,
    /// Attribute value that turns uiAccess on.
    pub ui_access_on: String,
    /// Exit code for a cancelled Windows prompt.
    pub cancel_code: i32,
    /// Exit code for any other script failure.
    pub fail_code: i32,
}

impl DevToolsConfig {
    /// Rejects bad counts, clashing exit codes, unsafe names and empty values.
    pub(crate) fn validate(&self) -> Result<(), ConfigError> {
        self.check_counts()?;
        self.check_codes()?;
        self.check_texts()?;
        first_failing(&self.names(), paths::is_plain_name, ConfigError::NotPlain)?;
        first_failing(
            &self.rel_paths(),
            paths::is_relative_plain,
            ConfigError::NotRelative,
        )
    }

    /// Counts must be non-zero, and the reuse limit must be below the lifetime.
    fn check_counts(&self) -> Result<(), ConfigError> {
        if self.cert_days == 0 {
            return Err(ConfigError::ZeroValue("cert_days"));
        }
        if self.install_max_mb == 0 {
            return Err(ConfigError::ZeroValue("install_max_mb"));
        }
        if self.cert_min_days >= self.cert_days {
            return Err(ConfigError::NotBelow("cert_min_days", "cert_days"));
        }
        Ok(())
    }

    /// Exit codes must be at least 1 and must differ, or a cancel could read as success.
    fn check_codes(&self) -> Result<(), ConfigError> {
        if self.cancel_code < 1 {
            return Err(ConfigError::NotPositive("cancel_code"));
        }
        if self.fail_code < 1 {
            return Err(ConfigError::NotPositive("fail_code"));
        }
        if self.cancel_code == self.fail_code {
            return Err(ConfigError::Same("cancel_code", "fail_code"));
        }
        Ok(())
    }

    /// Free-text values and the secure-folder list must not be blank; the subject must quote cleanly.
    fn check_texts(&self) -> Result<(), ConfigError> {
        let texts = [
            ("cert_subject", &self.cert_subject),
            ("shell_root_env", &self.shell_root_env),
            ("sdk_bin", &self.sdk_bin),
            ("digest", &self.digest),
            ("ui_access_tag", &self.ui_access_tag),
            ("ui_access_attr", &self.ui_access_attr),
            ("ui_access_on", &self.ui_access_on),
        ];
        first_failing(&texts, |v| !v.trim().is_empty(), ConfigError::EmptyValue)?;
        if !launcher::is_quotable(&self.cert_subject) {
            return Err(ConfigError::NotQuotable("cert_subject"));
        }
        if self.secure_env.is_empty() {
            return Err(ConfigError::EmptyList("secure_env"));
        }
        if self.secure_env.iter().any(|v| v.trim().is_empty()) {
            return Err(ConfigError::EmptyValue("secure_env"));
        }
        Ok(())
    }

    /// Values that must be one plain file or folder name.
    fn names(&self) -> [(&'static str, &String); 12] {
        [
            ("user_store", &self.user_store),
            ("cert_file", &self.cert_file),
            ("trust_launcher", &self.trust_launcher),
            ("untrust_launcher", &self.untrust_launcher),
            ("manifest_file", &self.manifest_file),
            ("admin_error_file", &self.admin_error_file),
            ("cert_script", &self.cert_script),
            ("admin_script", &self.admin_script),
            ("install_dir", &self.install_dir),
            ("sdk_arch", &self.sdk_arch),
            ("signtool", &self.signtool),
            ("mt", &self.mt),
        ]
    }

    /// Values that must be relative `/`-separated paths of plain names.
    fn rel_paths(&self) -> [(&'static str, &String); 3] {
        [
            ("out_dir", &self.out_dir),
            ("scripts_dir", &self.scripts_dir),
            ("powershell", &self.powershell),
        ]
    }

    /// The secure folders set in the environment; empty or relative values are dropped.
    pub(crate) fn secure_roots(&self, env: impl Fn(&str) -> Option<String>) -> Vec<PathBuf> {
        self.secure_env
            .iter()
            .filter_map(|key| env(key))
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .collect()
    }

    /// The env var naming the folder that receives installs: the first secure one.
    pub(crate) fn install_env(&self) -> &str {
        self.secure_env.first().map_or("", String::as_str)
    }

    /// The folder that receives installs, from [`Self::install_env`] only.
    pub(crate) fn install_root(
        &self,
        env: impl Fn(&str) -> Option<String>,
    ) -> Result<PathBuf, DevError> {
        let key = self.install_env();
        env(key)
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .ok_or_else(|| DevError::MissingEnv(key.to_string()))
    }
}

/// The error for the first value that fails `ok`, if any.
fn first_failing(
    values: &[(&'static str, &String)],
    ok: impl Fn(&str) -> bool,
    error: impl Fn(&'static str) -> ConfigError,
) -> Result<(), ConfigError> {
    match values.iter().find(|(_, value)| !ok(value)) {
        Some((name, _)) => Err(error(name)),
        None => Ok(()),
    }
}

/// The dev-tools settings plus the workspace root they are relative to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DevSetup {
    /// The workspace root directory.
    pub root: PathBuf,
    /// The validated settings.
    pub config: DevToolsConfig,
}

impl DevSetup {
    /// A `/`-separated path from the settings, joined under the workspace root.
    pub(crate) fn path(&self, rel: &str) -> PathBuf {
        paths::join_rel(self.root.clone(), rel)
    }

    /// A file in the output folder.
    pub(crate) fn out(&self, file: &str) -> PathBuf {
        self.path(&self.config.out_dir).join(file)
    }

    /// A script in the scripts folder.
    pub(crate) fn script(&self, file: &str) -> PathBuf {
        self.path(&self.config.scripts_dir).join(file)
    }

    /// The PowerShell program inside the Windows folder named by the environment.
    pub(crate) fn powershell(
        &self,
        env: impl Fn(&str) -> Option<String>,
    ) -> Result<PathBuf, DevError> {
        let key = &self.config.shell_root_env;
        let root = env(key)
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .ok_or_else(|| DevError::MissingEnv(key.clone()))?;
        Ok(paths::join_rel(root, &self.config.powershell))
    }
}

/// Loads and validates the dev-tools settings from `cargo metadata` JSON.
pub(crate) fn load(json: &str) -> Result<DevSetup, LoadError> {
    let (root, config): (PathBuf, DevToolsConfig) = workspace::root_and_section(json, SECTION)?;
    config.validate().map_err(workspace::config_error)?;
    Ok(DevSetup { root, config })
}

/// Runs `cargo metadata` and loads the dev-tools settings from it.
pub(crate) fn from_cargo() -> Result<DevSetup, LoadError> {
    load(&workspace::metadata_json()?)
}

#[cfg(test)]
mod tests;
