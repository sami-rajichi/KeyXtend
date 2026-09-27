//! Tidy and DCO config, deserialized from `[workspace.metadata]` in `Cargo.toml`.

use std::fmt;

use serde::Deserialize;

/// Tidy check settings from `[workspace.metadata.tidy]`.
#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TidyConfig {
    /// A source file over this many lines fails tidy.
    pub max_file_lines: usize,
    /// A source file over this many lines warns.
    pub warn_file_lines: usize,
    /// File extensions the size rule scans.
    pub extensions: Vec<String>,
    /// Directories tidy scans for source files.
    pub scan_dirs: Vec<String>,
    /// Directories tidy never scans.
    pub skip_dirs: Vec<String>,
    /// Packages whose manifest is under this folder are apps, not modules.
    pub app_dir: String,
    /// Crate-name prefix for feature modules.
    pub module_prefix: String,
    /// Crate-name prefix for platform adapters.
    pub platform_prefix: String,
    /// The only workspace crates a module may depend on.
    pub module_allowed: Vec<String>,
    /// The uiAccess app that the network and media bans protect.
    pub app: String,
    /// Network crates banned from the app.
    pub banned_network: Vec<String>,
    /// Media-decoding crates banned from the app.
    pub banned_media: Vec<String>,
    /// Attribute every non-platform crate root must carry.
    pub unsafe_attr: String,
    /// Target kinds counted as a crate root for the unsafe-attribute rule.
    pub root_kinds: Vec<String>,
}

impl TidyConfig {
    /// Rejects a config whose limits or lists don't make sense.
    pub(crate) fn validate(&self) -> Result<(), ConfigError> {
        if self.warn_file_lines > self.max_file_lines {
            return Err(ConfigError::WarnExceedsMax);
        }
        if self.extensions.is_empty() {
            return Err(ConfigError::EmptyList("extensions"));
        }
        if self.scan_dirs.is_empty() {
            return Err(ConfigError::EmptyList("scan_dirs"));
        }
        Ok(())
    }
}

/// DCO check settings from `[workspace.metadata.dco]`.
#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DcoConfig {
    /// Commit trailer that counts as a sign-off.
    pub trailer: String,
    /// Authors exempt from the DCO check.
    pub exempt_authors: Vec<String>,
}

/// A tidy or DCO config value that fails validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConfigError {
    /// `warn_file_lines` is greater than `max_file_lines`.
    WarnExceedsMax,
    /// A required list is empty.
    EmptyList(&'static str),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WarnExceedsMax => write!(f, "warn_file_lines exceeds max_file_lines"),
            Self::EmptyList(name) => write!(f, "{name} must not be empty"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::tidy_config as tidy;

    #[test]
    fn valid_config_passes_validation() {
        assert_eq!(tidy().validate(), Ok(()));
    }

    #[test]
    fn warn_greater_than_max_is_rejected() {
        let mut config = tidy();
        config.warn_file_lines = config.max_file_lines + 1;
        assert_eq!(config.validate(), Err(ConfigError::WarnExceedsMax));
    }

    #[test]
    fn warn_equal_to_max_is_allowed() {
        let mut config = tidy();
        config.warn_file_lines = config.max_file_lines;
        assert_eq!(config.validate(), Ok(()));
    }

    #[test]
    fn empty_extensions_is_rejected() {
        let mut config = tidy();
        config.extensions = vec![];
        assert_eq!(config.validate(), Err(ConfigError::EmptyList("extensions")));
    }

    #[test]
    fn empty_scan_dirs_is_rejected() {
        let mut config = tidy();
        config.scan_dirs = vec![];
        assert_eq!(config.validate(), Err(ConfigError::EmptyList("scan_dirs")));
    }

    #[test]
    fn tidy_config_rejects_unknown_field() {
        let json = r#"{
            "max_file_lines": 400, "warn_file_lines": 300,
            "extensions": [], "scan_dirs": [], "skip_dirs": [],
            "app_dir": "apps", "module_prefix": "kx-mod-", "platform_prefix": "kx-platform-",
            "module_allowed": [], "app": "keyxtend",
            "banned_network": [], "banned_media": [], "unsafe_attr": "x",
            "root_kinds": [], "bogus": 1
        }"#;
        assert!(serde_json::from_str::<TidyConfig>(json).is_err());
    }

    #[test]
    fn dco_config_rejects_unknown_field() {
        let json = r#"{"trailer": "Signed-off-by", "exempt_authors": [], "bogus": 1}"#;
        assert!(serde_json::from_str::<DcoConfig>(json).is_err());
    }
}
