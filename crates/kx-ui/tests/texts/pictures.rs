//! Hands the Quick picture test its folders: where to save the board's pictures, the baseline to compare them with,
//! and a scratch folder for its own check of that comparison.

use std::path::{Path, PathBuf};

/// The variable naming a folder for the board's pictures; a relative one counts from the repo root.
const PICTURES_VAR: &str = "KX_PICTURES";
/// The variable naming a folder of baseline pictures to compare with; a relative one counts from the repo root.
const BASE_VAR: &str = "KX_PICTURES_BASE";
/// The picture test's scratch folder, from the repo root.
const SCRATCH: &str = "target/tmp/qmltest-scratch";
/// The file in the laid-out assets that hands the folders to the Quick picture test.
pub const PICTURES_QML: &str = "Pictures.qml";
/// The repo root, as levels above this crate.
const REPO_UP: usize = 2;

/// The repo root.
fn repo() -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(REPO_UP);
    root.expect("the repo root").to_path_buf()
}

/// The folder variable `name` names, from the repo root, or none when it is unset or empty.
fn named(name: &str) -> Option<PathBuf> {
    let value = std::env::var_os(name).filter(|v| !v.is_empty());
    value.map(|v| repo().join(v))
}

/// Whether the pictures go elsewhere than their baseline; saved over it, each picture would match itself.
fn apart(folder: Option<&Path>, base: Option<&Path>) -> bool {
    folder.is_none() || folder != base
}

/// `folder` as a QML string, with forward slashes, or an empty string.
fn qml_string(folder: Option<&Path>) -> String {
    let shown = folder.map(|f| f.to_string_lossy().replace('\\', "/"));
    serde_json::to_string(&shown.unwrap_or_default()).expect("a string")
}

/// Writes `PICTURES_QML` into `mirror`: the pictures folder, made if missing; the baseline folder; and the scratch
/// folder, made. An unset variable hands over an empty string.
pub fn hand_over(mirror: &Path) {
    let [folder, base] = [named(PICTURES_VAR), named(BASE_VAR)];
    let same = !apart(folder.as_deref(), base.as_deref());
    assert!(!same, "{PICTURES_VAR} and {BASE_VAR} name the same folder");
    let scratch = repo().join(SCRATCH);
    for f in folder.iter().chain([&scratch]) {
        std::fs::create_dir_all(f).expect("a picture folder is made");
    }
    let [folder, base, scratch] = [
        qml_string(folder.as_deref()),
        qml_string(base.as_deref()),
        qml_string(Some(&scratch)),
    ];
    let qml = format!(
        "// Written by kx-ui's texts test from {PICTURES_VAR} and {BASE_VAR}.\nimport QtQml\nQtObject {{\n    \
         readonly property string folder: {folder}\n    readonly property string base: {base}\n    \
         readonly property string scratch: {scratch}\n}}\n"
    );
    std::fs::write(mirror.join(PICTURES_QML), qml).expect("the picture folders are handed over");
}

#[test]
fn pictures_saved_over_their_own_baseline_are_refused() {
    let [a, b] = [Path::new("target/a"), Path::new("target/b")];
    assert!(
        apart(Some(a), Some(b))
            && apart(Some(a), None)
            && apart(None, Some(a))
            && apart(None, None)
    );
    assert!(
        !apart(Some(a), Some(Path::new("target/a/"))),
        "the same folder, written another way"
    );
}
