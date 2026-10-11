//! What kx-ui's build script and the Quick tests' plugin (`tools/kx-test-icons`) share: the check that Qt is the one the
//! workspace pins, since both compile the soft shadows' adopter, and the build-kind header kx-ui's C++ includes.

use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::process::Command;

/// The variable naming Qt's qmake, as cxx-qt-build reads it, and the program it runs when that is unset.
const QMAKE_ENV: &str = "QMAKE";
const QMAKE: &str = "qmake";
/// qmake's key for Qt's version.
const VERSION_KEY: &str = "QT_VERSION";
/// The workspace manifest, from either crate's folder, and the path of the one Qt version it pins.
const WORKSPACE: &str = "../../Cargo.toml";
const PIN_PATH: [&str; 4] = ["workspace", "metadata", "stage", "qt_version"];
/// The header that tells kx-ui's C++ the build kind.
const KIND_H: &str = "kx-ui-build.h";

/// Qt's qmake, as cxx-qt-build finds it.
pub fn qmake() -> OsString {
    println!("cargo::rerun-if-env-changed={QMAKE_ENV}");
    std::env::var_os(QMAKE_ENV).unwrap_or_else(|| QMAKE.into())
}

/// Qt's version; an error when it is not the one the workspace pins, as the graphics device glue and the shadows'
/// adopter keep few promises across versions.
pub fn pinned_version(qmake: &OsStr) -> Result<String, String> {
    let (version, pinned) = (query(qmake, VERSION_KEY)?, pinned_qt()?);
    if version == pinned {
        return Ok(version);
    }
    let (table, key) = PIN_PATH.split_at(PIN_PATH.len() - 1);
    Err(format!(
        "Qt {version} is not the pinned {pinned}; check the graphics device glue (ADR-0021) and who frees \
         RectangularShadow's default material (the soft shadows' adopter) first, then change [{}] {} in the root \
         Cargo.toml",
        table.join("."),
        key.join(".")
    ))
}

/// What `qmake -query key` answers; an error when qmake does not run, fails, or answers nothing.
pub fn query(qmake: &OsStr, key: &str) -> Result<String, String> {
    let failed =
        |why: String| format!("qmake -query {key} failed: {why}; set {QMAKE_ENV} to Qt's qmake");
    let out = Command::new(qmake).args(["-query", key]).output();
    let out = out.map_err(|e| failed(e.to_string()))?;
    let text = String::from_utf8(out.stdout).map_err(|e| failed(e.to_string()))?;
    let text = text.trim();
    if !out.status.success() || text.is_empty() {
        return Err(failed(format!(
            "it ended with {} and answered {text:?}",
            out.status
        )));
    }
    Ok(text.to_owned())
}

/// The Qt version the workspace manifest pins.
fn pinned_qt() -> Result<String, String> {
    println!("cargo::rerun-if-changed={WORKSPACE}");
    let text = std::fs::read_to_string(WORKSPACE).map_err(|e| format!("{WORKSPACE}: {e}"))?;
    let table: toml::Table = text.parse().map_err(|e| format!("{WORKSPACE}: {e}"))?;
    let root = toml::Value::Table(table);
    let pin = PIN_PATH.iter().try_fold(&root, |v, key| v.get(key));
    pin.and_then(toml::Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| format!("{WORKSPACE} pins no Qt version at {}", PIN_PATH.join(".")))
}

/// Writes the build-kind header into `dir`, defining macro `probes` when given, so kx-ui's C++ builds with its
/// test-only probes or without them.
pub fn kind_header(dir: &Path, probes: Option<&str>) -> Result<(), String> {
    let probes = probes
        .map(|p| format!("#define {p} 1\n"))
        .unwrap_or_default();
    let text = format!("#pragma once\n{probes}");
    std::fs::write(dir.join(KIND_H), text).map_err(|e| e.to_string())
}
