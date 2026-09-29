//! Where `kx-gates.toml` is: beside the running exe first, else in the crate folder.
#![cfg(windows)]

use std::path::{Path, PathBuf};

use kx_target_window::config::locate;

/// File name of the gate settings.
pub const FILE: &str = "kx-gates.toml";

/// Where the settings file is: beside the running exe, else in the crate folder.
pub fn path() -> PathBuf {
    locate(FILE, Path::new(env!("CARGO_MANIFEST_DIR")))
}

#[cfg(test)]
mod tests {
    use std::path::{Component, Path, PathBuf};

    use kx_target_window::config::locate_for;

    use super::*;
    use crate::config;

    /// The shipped settings file.
    const SHIPPED: &str = include_str!("../kx-gates.toml");

    /// Cargo's build folder, in the repo root.
    const BUILD_DIR: &str = "target";
    /// The `[apps.*]` entry that is the test target window.
    const TARGET_APP: &str = "target";

    /// Where the settings file is for a run of `exe`.
    fn for_exe(exe: Option<&Path>) -> PathBuf {
        locate_for(exe, FILE, Path::new(env!("CARGO_MANIFEST_DIR")))
    }

    /// The settings file in the crate folder.
    fn crate_file() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join(FILE)
    }

    /// `path` with `.` and `..` folded away by name, so a folder that does not exist yet still resolves.
    fn fold(path: &Path) -> PathBuf {
        let mut out = PathBuf::new();
        for part in path.components() {
            match part {
                Component::CurDir => {}
                Component::ParentDir => {
                    out.pop();
                }
                other => out.push(other.as_os_str()),
            }
        }
        out
    }

    /// The repo root: two folders above the crate folder.
    fn repo_root() -> PathBuf {
        fold(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
    }

    /// Adds to `out` every string in `v` that starts at `.` or `..`.
    fn dotted(v: &toml::Value, out: &mut Vec<String>) {
        match v {
            toml::Value::String(s) => {
                let first = Path::new(s).components().next();
                if matches!(first, Some(Component::CurDir | Component::ParentDir)) {
                    out.push(s.clone());
                }
            }
            toml::Value::Array(items) => items.iter().for_each(|i| dotted(i, out)),
            toml::Value::Table(table) => table.values().for_each(|i| dotted(i, out)),
            _ => {}
        }
    }

    #[test]
    fn the_crate_folder_is_used_when_no_file_sits_beside_the_exe() {
        assert_eq!(
            path(),
            crate_file(),
            "the test exe sits in target/<profile>/deps"
        );
        let exe = Path::new("no/such/dir/kx-gates.exe");
        assert_eq!(for_exe(Some(exe)), crate_file());
        assert_eq!(for_exe(None), crate_file());
    }

    #[test]
    fn a_file_beside_the_exe_wins() {
        let dir = std::env::temp_dir().join(format!("kx-gates-cfg-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp folder");
        std::fs::write(dir.join(FILE), "").expect("temp file");
        let got = for_exe(Some(&dir.join("kx-gates.exe")));
        std::fs::remove_dir_all(&dir).expect("clean up");
        assert_eq!(got, dir.join(FILE));
    }

    #[test]
    fn folding_catches_a_path_that_leaves_the_repo() {
        let root = repo_root();
        assert!(fold(&root.join("target/gate-logs")).starts_with(&root));
        assert!(fold(&root.join("target/../docs")).starts_with(&root));
        assert!(!fold(&root.join("../elsewhere")).starts_with(&root));
        assert!(!fold(&root.join("target/../../elsewhere")).starts_with(&root));
    }

    #[test]
    fn every_relative_path_stays_inside_the_repo() {
        let cfg = config::load().expect("kx-gates.toml loads");
        let mut paths = vec![cfg.out_dir(), cfg.dir.join(&cfg.g22.worker_dir)];
        paths.extend(cfg.apps.values().map(|app| cfg.program(&app.exe)));
        let mut strings = Vec::new();
        dotted(&toml::from_str(SHIPPED).expect("parses"), &mut strings);
        paths.extend(strings.iter().map(|s| cfg.dir.join(s)));
        let root = repo_root();
        for p in paths.iter().filter(|p| p.starts_with(&cfg.dir)) {
            assert!(
                fold(p).starts_with(&root),
                "{} leaves {}",
                p.display(),
                root.display()
            );
        }
        // Build output only: an un-rebased `../target` stays in the repo but not in its `target/`.
        let target = root.join(BUILD_DIR);
        let built = [
            ("paths.out", cfg.out_dir()),
            ("g22.worker_dir", cfg.dir.join(&cfg.g22.worker_dir)),
            ("apps.target.exe", cfg.program(&cfg.apps[TARGET_APP].exe)),
        ];
        for (key, p) in built {
            assert!(
                fold(&p).starts_with(&target),
                "{key} = {} is not under {}",
                p.display(),
                target.display()
            );
        }
    }
}
