//! Tidy and DCO config, deserialized from `[workspace.metadata]` in `Cargo.toml`, and the shared config errors.

use std::collections::BTreeMap;
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
    /// Packages whose manifest is under this folder are test tools, which never ship.
    pub tool_dir: String,
    /// The build tool, which never ships; like the test tools, it may depend on a module, but not on a platform adapter.
    pub build_tool: String,
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
    /// Crate features no package may turn on, listed by crate name.
    pub banned_features: BTreeMap<String, Vec<String>>,
    /// Crate features only builds, tests and tools may turn on, listed by crate name; the app's tree never does.
    pub build_only_features: BTreeMap<String, Vec<String>>,
    /// Attribute every non-platform crate root must carry.
    pub unsafe_attr: String,
    /// Attribute a test tool's or the bridge crate's roots may carry instead of `unsafe_attr`.
    pub deny_attr: String,
    /// Target kinds counted as a crate root for the unsafe-attribute rule.
    pub root_kinds: Vec<String>,
    /// Crates only tests and tools may depend on.
    pub test_only: Vec<String>,
    /// The one crate beside the platform adapters that holds `unsafe`: the cxx-qt bridges.
    pub bridge_crate: String,
    /// The folder, relative to that crate, that alone may hold `unsafe`.
    pub bridge_dir: String,
    /// The comment that must stand above each `unsafe` in the bridge folder.
    pub safety_tag: String,
    /// Folders whose packages' product code starts no process itself (tidy F4).
    pub spawn_dirs: Vec<String>,
    /// File extensions F4 reads.
    pub spawn_extensions: Vec<String>,
    /// Process-start APIs F4 bans, as paths: it looks for the last name in a file that also names the one before it,
    /// and for a rename of the one before it.
    pub banned_spawns: Vec<String>,
    /// Build scripts, as paths inside their package, which may start processes.
    pub build_scripts: Vec<String>,
}

impl TidyConfig {
    /// Rejects a config whose limits or lists don't make sense.
    pub(crate) fn validate(&self) -> Result<(), ConfigError> {
        if self.warn_file_lines > self.max_file_lines {
            return Err(ConfigError::WarnExceedsMax);
        }
        // An empty list would make its check pass while reading nothing.
        let lists = [
            ("extensions", &self.extensions),
            ("scan_dirs", &self.scan_dirs),
            ("spawn_dirs", &self.spawn_dirs),
            ("spawn_extensions", &self.spawn_extensions),
            ("banned_spawns", &self.banned_spawns),
        ];
        if let Some((name, _)) = lists.iter().find(|(_, l)| l.is_empty()) {
            return Err(ConfigError::EmptyList(name));
        }
        let texts = [
            ("tool_dir", &self.tool_dir),
            ("build_tool", &self.build_tool),
            ("deny_attr", &self.deny_attr),
            ("bridge_crate", &self.bridge_crate),
            ("bridge_dir", &self.bridge_dir),
            ("safety_tag", &self.safety_tag),
        ];
        if let Some((name, _)) = texts.iter().find(|(_, v)| v.trim().is_empty()) {
            return Err(ConfigError::EmptyValue(name));
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

/// A `workspace.metadata` value that fails validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConfigError {
    /// `warn_file_lines` is greater than `max_file_lines`.
    WarnExceedsMax,
    /// A required list is empty.
    EmptyList(&'static str),
    /// A required text value is empty or blank.
    EmptyValue(&'static str),
    /// A count that must be non-zero is zero.
    ZeroValue(&'static str),
    /// An exit code is below 1.
    NotPositive(&'static str),
    /// A text holds a `"` or ends with `\`, which breaks command-line quoting.
    NotQuotable(&'static str),
    /// The first value must be smaller than the second.
    NotBelow(&'static str, &'static str),
    /// Two values that must differ are equal.
    Same(&'static str, &'static str),
    /// A value must be one plain file or folder name.
    NotPlain(&'static str),
    /// A value must be a relative `/`-separated path of plain names.
    NotRelative(&'static str),
    /// The first value must name only entries of the second list.
    NotListed(&'static str, &'static str),
    /// A list must hold only files with the given extension.
    NotType(&'static str, &'static str),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WarnExceedsMax => write!(f, "warn_file_lines exceeds max_file_lines"),
            Self::EmptyList(name) | Self::EmptyValue(name) => write!(f, "{name} must not be empty"),
            Self::ZeroValue(name) => write!(f, "{name} must not be zero"),
            Self::NotPositive(name) => write!(f, "{name} must be at least 1"),
            Self::NotQuotable(name) => write!(f, "{name} must not contain \" or end with \\"),
            Self::NotBelow(name, limit) => write!(f, "{name} must be smaller than {limit}"),
            Self::Same(a, b) => write!(f, "{a} and {b} must differ"),
            Self::NotPlain(name) => write!(f, "{name} must be one plain name"),
            Self::NotRelative(name) => write!(f, "{name} must be a relative path of plain names"),
            Self::NotListed(name, list) => write!(f, "{name} must name only entries of {list}"),
            Self::NotType(name, ext) => write!(f, "{name} must hold only .{ext} files"),
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
    fn each_required_list_must_not_be_empty() {
        let emptied = |name: &'static str, clear: fn(&mut TidyConfig)| {
            let mut config = tidy();
            clear(&mut config);
            assert_eq!(config.validate(), Err(ConfigError::EmptyList(name)));
        };
        emptied("extensions", |c| c.extensions.clear());
        emptied("scan_dirs", |c| c.scan_dirs.clear());
        emptied("spawn_dirs", |c| c.spawn_dirs.clear());
        emptied("spawn_extensions", |c| c.spawn_extensions.clear());
        emptied("banned_spawns", |c| c.banned_spawns.clear());
    }

    #[test]
    fn empty_tool_dir_is_rejected() {
        let mut config = tidy();
        config.tool_dir = String::new();
        assert_eq!(config.validate(), Err(ConfigError::EmptyValue("tool_dir")));
    }

    #[test]
    fn blank_deny_attr_is_rejected() {
        let mut config = tidy();
        config.deny_attr = "  ".to_string();
        assert_eq!(config.validate(), Err(ConfigError::EmptyValue("deny_attr")));
    }

    #[test]
    fn blank_bridge_values_are_rejected() {
        let mut config = tidy();
        config.bridge_dir = String::new();
        assert_eq!(
            config.validate(),
            Err(ConfigError::EmptyValue("bridge_dir"))
        );
        let mut config = tidy();
        config.safety_tag = " ".to_string();
        assert_eq!(
            config.validate(),
            Err(ConfigError::EmptyValue("safety_tag"))
        );
        let mut config = tidy();
        config.bridge_crate = String::new();
        let want = Err(ConfigError::EmptyValue("bridge_crate"));
        assert_eq!(config.validate(), want);
    }

    #[test]
    fn blank_build_tool_is_rejected() {
        let mut config = tidy();
        config.build_tool = " ".to_string();
        let want = Err(ConfigError::EmptyValue("build_tool"));
        assert_eq!(config.validate(), want);
    }

    #[test]
    fn error_messages_name_their_values() {
        assert_eq!(
            ConfigError::ZeroValue("x").to_string(),
            "x must not be zero"
        );
        assert_eq!(
            ConfigError::NotPositive("x").to_string(),
            "x must be at least 1"
        );
        assert_eq!(
            ConfigError::NotQuotable("x").to_string(),
            "x must not contain \" or end with \\"
        );
        assert_eq!(
            ConfigError::NotBelow("a", "b").to_string(),
            "a must be smaller than b"
        );
        assert_eq!(
            ConfigError::Same("a", "b").to_string(),
            "a and b must differ"
        );
        assert_eq!(
            ConfigError::NotListed("a", "b").to_string(),
            "a must name only entries of b"
        );
        assert_eq!(
            ConfigError::NotType("a", "dll").to_string(),
            "a must hold only .dll files"
        );
    }

    #[test]
    fn tidy_config_rejects_unknown_field() {
        let json = r#"{
            "max_file_lines": 400, "warn_file_lines": 300,
            "extensions": [], "scan_dirs": [], "skip_dirs": [],
            "app_dir": "apps", "tool_dir": "tools", "build_tool": "xtask",
            "module_prefix": "kx-mod-", "platform_prefix": "kx-platform-",
            "module_allowed": [], "app": "keyxtend",
            "banned_network": [], "banned_media": [], "banned_features": {},
            "build_only_features": {}, "unsafe_attr": "x",
            "deny_attr": "y", "root_kinds": [], "test_only": [], "bridge_crate": "b",
            "bridge_dir": "d", "safety_tag": "t", "spawn_dirs": [], "spawn_extensions": [],
            "banned_spawns": [], "build_scripts": [], "bogus": 1
        }"#;
        assert!(serde_json::from_str::<TidyConfig>(json).is_err());
        let known = json.replace(r#", "bogus": 1"#, "");
        assert!(
            serde_json::from_str::<TidyConfig>(&known).is_ok(),
            "only the unknown field fails"
        );
    }

    #[test]
    fn dco_config_rejects_unknown_field() {
        let json = r#"{"trailer": "Signed-off-by", "exempt_authors": [], "bogus": 1}"#;
        assert!(serde_json::from_str::<DcoConfig>(json).is_err());
    }
}
