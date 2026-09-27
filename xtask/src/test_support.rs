//! Shared helpers for unit tests: unique temp folders and test configs.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::config::{DcoConfig, TidyConfig};

static UNIQUE: AtomicU32 = AtomicU32::new(0);

/// A fresh, not-yet-existing folder under the OS temp dir, unique across parallel tests.
pub(crate) fn unique_temp_dir(label: &str) -> PathBuf {
    let n = UNIQUE.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("kx-xtask-{label}-{}-{n}", std::process::id()))
}

/// A trimmed copy of the project's tidy config: the same limits, prefixes and app, but shorter lists.
pub(crate) fn tidy_config() -> TidyConfig {
    TidyConfig {
        max_file_lines: 400,
        warn_file_lines: 300,
        extensions: vec!["rs".to_string()],
        scan_dirs: vec!["xtask".to_string()],
        skip_dirs: vec!["target".to_string()],
        app_dir: "apps".to_string(),
        module_prefix: "kx-mod-".to_string(),
        platform_prefix: "kx-platform-".to_string(),
        module_allowed: vec!["kx-module-api".to_string()],
        app: "keyxtend".to_string(),
        banned_network: vec!["reqwest".to_string()],
        banned_media: vec!["image".to_string()],
        unsafe_attr: "#![forbid(unsafe_code)]".to_string(),
        root_kinds: vec![
            "lib".to_string(),
            "bin".to_string(),
            "proc-macro".to_string(),
        ],
    }
}

/// The project's DCO config: the `Signed-off-by` trailer, with `dependabot[bot]` exempt.
pub(crate) fn dco_config() -> DcoConfig {
    DcoConfig {
        trailer: "Signed-off-by".to_string(),
        exempt_authors: vec!["dependabot[bot]".to_string()],
    }
}
