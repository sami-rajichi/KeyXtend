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
/// File name of the settings P7 adds; it is read only from the folder of `FILE`.
pub const P7_FILE: &str = "kx-gates-p7.toml";
/// File name of the settings P8 adds; it is read only from the folder of `FILE`.
pub const P8_FILE: &str = "kx-gates-p8.toml";
/// kx-gates' own settings files, in the order they are read.
pub const OWN: [&str; 6] = [FILE, P3_FILE, P4_FILE, P5_FILE, P7_FILE, P8_FILE];
/// Every settings file kx-gates reads: its own, the spike's and target-window's.
pub const ALL: [&str; 8] = [
    FILE,
    P3_FILE,
    P4_FILE,
    P5_FILE,
    P7_FILE,
    P8_FILE,
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
mod tests;
