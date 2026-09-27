//! Finds Windows SDK tools such as `signtool.exe` in the newest SDK that has them.

use std::path::{Path, PathBuf};

use super::DevError;
use super::config::DevToolsConfig;

/// Parses an SDK folder name such as `10.0.22621.0` into comparable numbers.
fn version(name: &str) -> Option<Vec<u32>> {
    name.split('.').map(|part| part.parse().ok()).collect()
}

/// The newest version folder in `names` for which `has_tool` is true.
pub(crate) fn pick_newest(names: &[String], has_tool: impl Fn(&str) -> bool) -> Option<&str> {
    names
        .iter()
        .filter_map(|name| version(name).map(|v| (v, name.as_str())))
        .filter(|(_, name)| has_tool(name))
        .max()
        .map(|(_, name)| name)
}

/// The full path of `tool` in the newest SDK version folder that contains it.
pub(crate) fn find_tool(config: &DevToolsConfig, tool: &str) -> Result<PathBuf, DevError> {
    let root = Path::new(&config.sdk_bin);
    let names: Vec<String> = std::fs::read_dir(root)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    let path_of = |name: &str| root.join(name).join(&config.sdk_arch).join(tool);
    pick_newest(&names, |name| path_of(name).is_file())
        .map(path_of)
        .ok_or_else(|| DevError::NoTool {
            tool: tool.to_string(),
            root: config.sdk_bin.clone(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(values: &[&str]) -> Vec<String> {
        values.iter().map(|v| (*v).to_string()).collect()
    }

    #[test]
    fn newest_version_with_the_tool_wins() {
        let list = names(&["10.0.17134.0", "10.0.22621.0"]);
        assert_eq!(pick_newest(&list, |_| true), Some("10.0.22621.0"));
    }

    #[test]
    fn a_newer_version_without_the_tool_is_skipped() {
        let list = names(&["10.0.22621.0", "10.0.26100.0"]);
        let found = pick_newest(&list, |v| v != "10.0.26100.0");
        assert_eq!(found, Some("10.0.22621.0"));
    }

    #[test]
    fn versions_compare_as_numbers_not_text() {
        let list = names(&["10.0.9.0", "10.0.10.0"]);
        assert_eq!(pick_newest(&list, |_| true), Some("10.0.10.0"));
    }

    #[test]
    fn folders_that_are_not_versions_are_ignored() {
        let list = names(&["x64", "arm64", "10.0.16299.0"]);
        assert_eq!(pick_newest(&list, |_| true), Some("10.0.16299.0"));
    }

    #[test]
    fn no_folder_with_the_tool_gives_none() {
        let list = names(&["10.0.22621.0"]);
        assert_eq!(pick_newest(&list, |_| false), None);
    }

    #[test]
    fn find_tool_returns_version_then_arch_then_tool() {
        let dir = crate::test_support::TempDir::new("sdk");
        let bin = dir.path().join("10.0.1.0").join("x64");
        std::fs::create_dir_all(&bin).unwrap();
        std::fs::write(bin.join("signtool.exe"), b"").unwrap();
        std::fs::create_dir_all(dir.path().join("10.0.2.0").join("x64")).unwrap();
        let mut config = crate::test_support::devtools_config();
        config.sdk_bin = dir.path().to_string_lossy().into_owned();
        let found = find_tool(&config, "signtool.exe");
        assert_eq!(found, Ok(bin.join("signtool.exe")));
    }

    #[test]
    fn a_missing_sdk_root_names_the_tool() {
        let mut config = crate::test_support::devtools_config();
        config.sdk_bin = crate::test_support::unique_temp_dir("no-sdk")
            .to_string_lossy()
            .into_owned();
        let err = find_tool(&config, "signtool.exe").unwrap_err();
        assert!(err.to_string().contains("signtool.exe"));
    }
}
