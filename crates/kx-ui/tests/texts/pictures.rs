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

/// Whether the pictures go elsewhere than their baseline, as the file system finds both, so case, `..` and links
/// count; saved over it, each picture would match itself.
fn apart(folder: Option<&Path>, base: Option<&Path>) -> bool {
    let real = |p: &Path| std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    match (folder, base) {
        (Some(f), Some(b)) => real(f) != real(b),
        _ => true,
    }
}

/// `folder` as a QML string, with forward slashes, or an empty string.
fn qml_string(folder: Option<&Path>) -> String {
    let shown = folder.map(|f| f.to_string_lossy().replace('\\', "/"));
    serde_json::to_string(&shown.unwrap_or_default()).expect("a string")
}

/// Writes `PICTURES_QML` into `mirror` from the folders `PICTURES_VAR` and `BASE_VAR` name, as `hand_to` does.
pub fn hand_over(mirror: &Path) {
    let [folder, base] = [named(PICTURES_VAR), named(BASE_VAR)];
    hand_to(mirror, folder.as_deref(), base.as_deref());
}

/// Writes `PICTURES_QML` into `mirror`: pictures folder `folder`, made if missing; baseline `base`; and the scratch
/// folder, made. A missing folder hands over an empty string; the same folder twice is refused, and no older hand-over
/// is left for the picture test to read.
fn hand_to(mirror: &Path, folder: Option<&Path>, base: Option<&Path>) {
    let file = mirror.join(PICTURES_QML);
    if file.exists() {
        std::fs::remove_file(&file).expect("the old hand-over goes");
    }
    let scratch = repo().join(SCRATCH);
    for f in folder.into_iter().chain([scratch.as_path()]) {
        std::fs::create_dir_all(f).expect("a picture folder is made");
    }
    assert!(
        apart(folder, base),
        "{PICTURES_VAR} and {BASE_VAR} name the same folder"
    );
    let [folder, base, scratch] = [
        qml_string(folder),
        qml_string(base),
        qml_string(Some(&scratch)),
    ];
    let qml = format!(
        "// Written by kx-ui's texts test from {PICTURES_VAR} and {BASE_VAR}.\nimport QtQml\nQtObject {{\n    \
         readonly property string folder: {folder}\n    readonly property string base: {base}\n    \
         readonly property string scratch: {scratch}\n}}\n"
    );
    std::fs::write(&file, qml).expect("the picture folders are handed over");
}

#[test]
fn pictures_saved_over_their_own_baseline_are_refused() {
    let dir = repo().join(SCRATCH).join("apart");
    let [a, b, mirror] = [dir.join("a"), dir.join("b"), dir.join("mirror")];
    for f in [&a, &b, &mirror] {
        std::fs::create_dir_all(f).expect("a test folder is made");
    }
    assert!(apart(Some(&a), Some(&b)) && apart(Some(&a), None) && apart(None, None));
    // The same folder written another way: through `..`, and in another case where the file system ignores case.
    let twin = b.join("..").join("a");
    assert!(!apart(Some(&a), Some(&twin)));
    #[cfg(windows)]
    assert!(!apart(Some(&a), Some(&dir.join("A"))));
    std::fs::write(mirror.join(PICTURES_QML), "old").expect("an old hand-over");
    let refused = std::panic::catch_unwind(|| hand_to(&mirror, Some(&a), Some(&twin)));
    let left = mirror.join(PICTURES_QML).exists();
    std::fs::remove_dir_all(&dir).expect("the test folders go");
    assert!(
        refused.is_err() && !left,
        "refused, with no old hand-over left"
    );
}
