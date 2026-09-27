//! Shared helpers for unit tests: unique temp folders and test configs.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use crate::config::{DcoConfig, TidyConfig};
use crate::devtools::config::DevToolsConfig;

static UNIQUE: AtomicU32 = AtomicU32::new(0);

/// A fresh, not-yet-existing folder under the OS temp dir, unique across parallel tests.
pub(crate) fn unique_temp_dir(label: &str) -> PathBuf {
    let n = UNIQUE.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("kx-xtask-{label}-{}-{n}", std::process::id()))
}

/// A temp folder that is created now and deleted when dropped, even if a test fails.
pub(crate) struct TempDir(PathBuf);

impl TempDir {
    /// Creates a fresh folder named after `label`.
    pub(crate) fn new(label: &str) -> Self {
        let path = unique_temp_dir(label);
        std::fs::create_dir_all(&path).expect("the temp folder can be created");
        Self(path)
    }

    /// The folder's path.
    pub(crate) fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        // A leftover temp folder is harmless, so a failed delete is ignored.
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Links the folder `link` to `target` with a symlink, which Unix allows without admin rights.
#[cfg(unix)]
pub(crate) fn make_dir_link(link: &Path, target: &Path) {
    std::os::unix::fs::symlink(target, link).unwrap();
}

/// Links the folder `link` to `target` with a junction, which Windows allows without admin rights.
#[cfg(windows)]
pub(crate) fn make_dir_link(link: &Path, target: &Path) {
    let output = std::process::Command::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(link)
        .arg(target)
        .output()
        .unwrap();
    assert!(output.status.success(), "mklink /J failed: {output:?}");
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

/// Test dev-tools settings as `cargo metadata` prints them; `cargo xtask tidy` checks the real ones.
pub(crate) fn devtools_json() -> serde_json::Value {
    serde_json::json!({
        "cert_subject": "CN=KeyXtend Dev Test",
        "cert_days": 90,
        "cert_min_days": 14,
        "user_store": "My",
        "out_dir": "target/dev-tools",
        "cert_file": "keyxtend-dev.cer",
        "trust_launcher": "trust-dev-cert.cmd",
        "untrust_launcher": "untrust-dev-cert.cmd",
        "manifest_file": "manifest.xml",
        "admin_error_file": "admin-error.txt",
        "scripts_dir": "xtask/scripts",
        "cert_script": "dev-cert.ps1",
        "admin_script": "dev-admin.ps1",
        "shell_root_env": "SystemRoot",
        "powershell": "System32/WindowsPowerShell/v1.0/powershell.exe",
        "install_dir": "KeyXtend-dev",
        "install_max_mb": 500,
        "secure_env": ["ProgramFiles", "ProgramFiles(x86)"],
        "sdk_bin": "C:\\Program Files (x86)\\Windows Kits\\10\\bin",
        "sdk_arch": "x64",
        "signtool": "signtool.exe",
        "mt": "mt.exe",
        "digest": "SHA256",
        "ui_access_tag": "requestedExecutionLevel",
        "ui_access_attr": "uiAccess",
        "ui_access_on": "true",
        "cancel_code": 1223,
        "fail_code": 1
    })
}

/// The test dev-tools settings, deserialized.
pub(crate) fn devtools_config() -> DevToolsConfig {
    serde_json::from_value(devtools_json()).expect("the test settings are valid")
}

/// The project's DCO config: the `Signed-off-by` trailer, with `dependabot[bot]` exempt.
pub(crate) fn dco_config() -> DcoConfig {
    DcoConfig {
        trailer: "Signed-off-by".to_string(),
        exempt_authors: vec!["dependabot[bot]".to_string()],
    }
}
