//! Where kx-gates' settings are: beside the running exe first, else in the crate folder; with uiAccess, only beside
//! the exe, as the crate folder is user-writable.
#![cfg(windows)]

use std::path::{Path, PathBuf};

use kx_platform_windows::uiaccess;

/// File name of the gate settings.
pub const FILE: &str = "kx-gates.toml";
/// File name of the settings of the gates P3 adds; it is read only from the folder of `FILE`.
pub const P3_FILE: &str = "kx-gates-p3.toml";
/// File name of the settings P4 adds; it is read only from the folder of `FILE`.
pub const P4_FILE: &str = "kx-gates-p4.toml";
/// File name of the settings P5 adds; it is read only from the folder of `FILE`.
pub const P5_FILE: &str = "kx-gates-p5.toml";
/// kx-gates' own settings files, in the order they are read.
pub const OWN: [&str; 4] = [FILE, P3_FILE, P4_FILE, P5_FILE];
/// Every settings file kx-gates reads: its own, the spike's and target-window's.
pub const ALL: [&str; 6] = [
    FILE,
    P3_FILE,
    P4_FILE,
    P5_FILE,
    spike_core::config::FILE,
    kx_target_window::config::FILE,
];

/// The folder kx-gates' own settings are read from, for an exe in `exe_dir`; `present` says whether a file sits beside
/// it. With `ui_access`, every file in `ALL` must sit there; without, `FILE` beside it wins over `crate_dir`; unknown,
/// none is read.
pub fn choose(
    exe_dir: Option<&Path>,
    crate_dir: &Path,
    ui_access: Result<bool, String>,
    present: impl Fn(&str) -> bool,
) -> Result<PathBuf, String> {
    let ui_access = ui_access.map_err(|e| {
        format!("kx-gates reads no settings without knowing whether it has uiAccess: {e}")
    })?;
    if !ui_access {
        let beside = exe_dir.filter(|_| present(FILE));
        return Ok(beside.unwrap_or(crate_dir).to_path_buf());
    }
    let dir = exe_dir.ok_or("kx-gates has uiAccess but no exe folder to read its settings from")?;
    let missing: Vec<&str> = ALL.into_iter().filter(|name| !present(name)).collect();
    if missing.is_empty() {
        return Ok(dir.to_path_buf());
    }
    Err(format!(
        "kx-gates has uiAccess, so it reads its settings only beside its exe in {}: {} missing. Put {} beside the \
         built kx-gates.exe before `cargo xtask dev-install`.",
        dir.display(),
        missing.join(", "),
        ALL.join(", ")
    ))
}

/// Where `FILE` is for this run, by `choose`.
pub fn path() -> Result<PathBuf, String> {
    let exe = std::env::current_exe().ok();
    let exe_dir = exe.as_deref().and_then(Path::parent);
    let present = |name: &str| exe_dir.is_some_and(|dir| dir.join(name).is_file());
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let ui_access = uiaccess::check().map_err(|e| e.to_string());
    Ok(choose(exe_dir, crate_dir, ui_access, present)?.join(FILE))
}

#[cfg(test)]
mod tests {
    use std::path::{Component, Path, PathBuf};

    use super::*;
    use crate::config;

    /// The shipped settings file.
    const SHIPPED: &str = include_str!("../kx-gates.toml");
    /// The shipped settings of the gates P3 adds.
    const SHIPPED_P3: &str = include_str!("../kx-gates-p3.toml");
    /// The shipped settings P4 adds.
    const SHIPPED_P4: &str = include_str!("../kx-gates-p4.toml");
    /// The shipped settings P5 adds.
    const SHIPPED_P5: &str = include_str!("../kx-gates-p5.toml");
    /// The workspace manifest, whose `[workspace.metadata.tidy]` holds the file size rule.
    const WORKSPACE: &str = include_str!("../../../Cargo.toml");

    #[test]
    fn each_shipped_settings_file_stays_within_the_size_rule() {
        let ws: toml::Table = WORKSPACE.parse().expect("parses");
        let max = ws["workspace"]["metadata"]["tidy"]["max_file_lines"]
            .as_integer()
            .expect("a number");
        for (name, text) in [
            (FILE, SHIPPED),
            (P3_FILE, SHIPPED_P3),
            (P4_FILE, SHIPPED_P4),
            (P5_FILE, SHIPPED_P5),
        ] {
            let lines = i64::try_from(text.lines().count()).expect("fits");
            assert!(
                lines <= max,
                "{name} has {lines} lines: move a section to its own file"
            );
        }
    }

    /// Cargo's build folder, in the repo root.
    const BUILD_DIR: &str = "target";
    /// The `[apps.*]` entry that is the test target window.
    const TARGET_APP: &str = "target";

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

    /// The CI workflow, whose G26 job prints the logs in the gates' out folder; a workspace path from this file.
    const CI: &str = include_str!("../../../.github/workflows/ci.yml");
    /// The G26 job's variable that names that folder.
    const GATE_LOGS: &str = "GATE_LOGS: ";

    #[test]
    fn the_g26_job_prints_the_logs_of_the_gates_out_folder() {
        let cfg = config::load().expect("kx-gates.toml loads");
        let named = CI.lines().find_map(|l| l.trim().strip_prefix(GATE_LOGS));
        let out = fold(&cfg.out_dir());
        assert_eq!(repo_root().join(named.expect("the G26 job names it")), out);
        let target = kx_target_window::config::load().expect("target config loads");
        let log_dir = fold(target.log.parent().expect("a folder"));
        assert_eq!(log_dir, out, "target-window's log is there too");
    }

    #[test]
    fn the_crate_folder_is_used_when_no_file_sits_beside_the_exe() {
        assert_eq!(
            path(),
            Ok(crate_file()),
            "the test exe sits in target/<profile>/deps, without uiAccess"
        );
    }

    /// A folder for the exe and the crate folder, for `choose`.
    const EXE_DIR: &str = "exe";
    const CRATE_DIR: &str = "crate";

    /// Where `choose` reads the settings for an exe folder, uiAccess or not, and the files beside the exe.
    fn chosen(exe: bool, ui_access: bool, beside: &[&str]) -> Result<PathBuf, String> {
        let exe_dir = exe.then(|| Path::new(EXE_DIR));
        choose(exe_dir, Path::new(CRATE_DIR), Ok(ui_access), |n| {
            beside.contains(&n)
        })
    }

    #[test]
    fn an_unknown_ui_access_reads_no_settings() {
        let why = "the token said nothing";
        let e = choose(
            Some(Path::new(EXE_DIR)),
            Path::new(CRATE_DIR),
            Err(why.into()),
            |_| true,
        )
        .expect_err("fails closed");
        assert!(e.contains(why) && e.contains("uiAccess"), "{e}");
    }

    #[test]
    fn without_ui_access_the_crate_folder_serves_unless_the_file_sits_beside_the_exe() {
        assert_eq!(chosen(true, false, &[]), Ok(CRATE_DIR.into()));
        assert_eq!(chosen(false, false, &ALL), Ok(CRATE_DIR.into()));
        assert_eq!(chosen(true, false, &[FILE]), Ok(EXE_DIR.into()));
        assert_eq!(
            chosen(true, true, &ALL),
            Ok(EXE_DIR.into()),
            "uiAccess, all six beside"
        );
    }

    #[test]
    fn with_ui_access_any_file_missing_beside_the_exe_is_an_error_naming_it() {
        for gone in ALL {
            let rest: Vec<&str> = ALL.into_iter().filter(|n| *n != gone).collect();
            let e = chosen(true, true, &rest).expect_err(gone);
            assert!(e.contains(&format!("{gone} missing")), "{e}");
            assert!(
                ALL.iter().all(|n| e.contains(n)) && e.contains("dev-install"),
                "{e}"
            );
        }
        assert!(
            chosen(false, true, &ALL).is_err(),
            "no exe folder to read from"
        );
    }

    #[test]
    fn the_ui_access_rule_covers_every_file_kx_gates_reads() {
        let all: std::collections::BTreeSet<_> = ALL.into_iter().collect();
        assert_eq!(all.len(), ALL.len(), "each file once");
        for name in OWN {
            assert!(all.contains(name), "{name}");
        }
        assert!(
            all.contains(spike_core::config::FILE) && all.contains(kx_target_window::config::FILE)
        );
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
        for shipped in [SHIPPED, SHIPPED_P3, SHIPPED_P4, SHIPPED_P5] {
            dotted(&toml::from_str(shipped).expect("parses"), &mut strings);
        }
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
            ("gate_input.program", cfg.program(&cfg.gate_input.program)),
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
