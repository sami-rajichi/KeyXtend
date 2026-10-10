//! The Qt Quick tests' QML plugin (kx-test-icons): it is laid out beside the bundled icons, and `qmltest/qmldir` loads
//! it from there, so every test's engine draws the icons as the app does.

use crate::qml::MIRROR;
use kx_test_support::golden::assert_golden;
use std::env::consts::{DLL_PREFIX, DLL_SUFFIX};
use std::path::Path;

/// The plugin's library name, as Cargo builds it beside this test.
const PLUGIN: &str = "kx_test_icons";
/// The file through which the Quick tests' folder loads it, under this crate.
const QMLDIR: &str = "qmltest/qmldir";

/// Copies the plugin, built beside this test, into `mirror`; only on Windows, where it is built and the Quick tests
/// run.
pub fn lay_out(mirror: &Path) {
    if cfg!(windows) {
        let name = format!("{DLL_PREFIX}{PLUGIN}{DLL_SUFFIX}");
        let exe = std::env::current_exe().expect("the test's own path");
        std::fs::copy(exe.with_file_name(&name), mirror.join(&name))
            .expect("the icon plugin is laid out");
    }
}

#[test]
fn the_quick_tests_load_the_icon_plugin_from_the_layout() {
    // The Quick tests sit one folder below this crate.
    let text =
        format!("# Written by kx-ui's texts test; do not edit.\nplugin {PLUGIN} ../{MIRROR}\n");
    assert_golden(Path::new(env!("CARGO_MANIFEST_DIR")).join(QMLDIR), &text);
}
