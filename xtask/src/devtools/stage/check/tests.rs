use std::collections::BTreeMap;
use std::path::PathBuf;

use super::*;
use crate::devtools::paths;
use crate::test_support::{TempDir, make_dir_link, stage_config};

/// Libraries the fake system folder holds, in lower case.
const SYSTEM: [&str; 2] = ["kernel32.dll", "vcruntime140.dll"];
/// The app's program in the fake stages.
const PROGRAM: &str = "keyxtend.exe";

/// The test settings, with the app named so its program is [`PROGRAM`] on every system.
fn settings() -> StageConfig {
    let mut config = stage_config();
    let app = PROGRAM.strip_suffix(std::env::consts::EXE_SUFFIX).unwrap();
    config.app = app.to_string();
    config
}

/// A fake stage folder: each file is empty, and its imports are given by name.
struct Fake {
    dir: TempDir,
    imports: BTreeMap<String, Vec<String>>,
}

impl Fake {
    /// A stage holding `files`, each a `/`-separated path with the libraries it imports.
    fn new(files: &[(&str, &[&str])]) -> Self {
        let dir = TempDir::new("stage");
        let mut imports = BTreeMap::new();
        for (file, libs) in files {
            let path = paths::join_rel(dir.path().to_path_buf(), file);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, b"").unwrap();
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            imports.insert(name, libs.iter().map(ToString::to_string).collect());
        }
        Self { dir, imports }
    }

    /// The imports of the staged file at `path`.
    fn read(&self, path: &Path) -> Vec<String> {
        let name = path.file_name().unwrap().to_string_lossy();
        self.imports.get(name.as_ref()).cloned().unwrap_or_default()
    }

    /// The stage check over this folder, with `config`.
    fn check_with(&self, config: &StageConfig) -> Result<Summary, DevError> {
        let in_system = |lib: &str| SYSTEM.contains(&lib.to_lowercase().as_str());
        check(self.dir.path(), config, |p| Ok(self.read(p)), in_system)
    }

    /// The stage check over this folder, with the test settings.
    fn check(&self) -> Result<Summary, DevError> {
        self.check_with(&settings())
    }
}

/// The text of a staged path, as the check reports it.
fn shown(rel: &str) -> String {
    paths::text(&paths::join_rel(PathBuf::new(), rel))
}

#[test]
fn the_check_refuses_a_tls_plugin() {
    let stage = Fake::new(&[
        (PROGRAM, &["Qt6Core.dll"]),
        ("Qt6Core.dll", &[]),
        ("tls/qschannelbackend.dll", &["Qt6Core.dll"]),
    ]);
    let banned = shown("tls/qschannelbackend.dll");
    assert_eq!(stage.check(), Err(DevError::Banned(banned)));
}

#[test]
fn the_check_refuses_a_network_information_plugin() {
    let stage = Fake::new(&[
        (PROGRAM, &["Qt6Core.dll"]),
        ("Qt6Core.dll", &[]),
        (
            "networkinformation/qnetworklistmanager.dll",
            &["Qt6Core.dll"],
        ),
    ]);
    let banned = shown("networkinformation/qnetworklistmanager.dll");
    assert_eq!(stage.check(), Err(DevError::Banned(banned)));
}

#[test]
fn banned_folders_match_in_any_letter_case_and_depth() {
    let stage = Fake::new(&[(PROGRAM, &[]), ("qml/x/TLS/y.dll", &[])]);
    assert_eq!(
        stage.check(),
        Err(DevError::Banned(shown("qml/x/TLS/y.dll")))
    );
}

#[test]
fn the_check_refuses_a_network_library_nothing_imports() {
    let stage = Fake::new(&[
        (PROGRAM, &["Qt6Core.dll"]),
        ("Qt6Core.dll", &[]),
        ("Qt6Network.dll", &["Qt6Core.dll"]),
    ]);
    let unused = DevError::Unused("Qt6Network.dll".to_string());
    assert_eq!(stage.check(), Err(unused));
}

#[test]
fn a_network_library_the_qml_library_imports_passes() {
    let stage = Fake::new(&[
        (PROGRAM, &["Qt6Qml.dll", "KERNEL32.dll"]),
        ("Qt6Qml.dll", &["Qt6Network.dll", "Qt6Core.dll"]),
        ("Qt6Network.dll", &["Qt6Core.dll"]),
        (
            "Qt6Core.dll",
            &["VCRUNTIME140.dll", "api-ms-win-crt-heap-l1-1-0.dll"],
        ),
    ]);
    assert_eq!(stage.check(), Ok(Summary { files: 4, bytes: 0 }));
}

#[test]
fn an_import_found_nowhere_is_refused() {
    let stage = Fake::new(&[(PROGRAM, &["missing.dll"])]);
    let err = DevError::Unresolved {
        by: PROGRAM.to_string(),
        lib: "missing.dll".to_string(),
    };
    assert_eq!(stage.check(), Err(err));
}

#[test]
fn an_import_with_a_folder_never_resolves() {
    for lib in ["../kernel32.dll", "sub\\kernel32.dll", "C:kernel32.dll"] {
        let stage = Fake::new(&[(PROGRAM, &[lib])]);
        let found = check(
            stage.dir.path(),
            &settings(),
            |p| Ok(stage.read(p)),
            |_| true,
        );
        assert!(
            matches!(found, Err(DevError::Unresolved { .. })),
            "{lib} resolved"
        );
    }
}

#[test]
fn a_plugin_import_must_resolve_too() {
    let stage = Fake::new(&[(PROGRAM, &[]), ("platforms/qwindows.dll", &["Qt6Gui.dll"])]);
    let err = DevError::Unresolved {
        by: shown("platforms/qwindows.dll"),
        lib: "Qt6Gui.dll".to_string(),
    };
    assert_eq!(stage.check(), Err(err));
}

#[test]
fn a_library_in_a_subfolder_never_satisfies_an_import() {
    let stage = Fake::new(&[(PROGRAM, &["Qt6Svg.dll"]), ("imageformats/Qt6Svg.dll", &[])]);
    assert!(matches!(stage.check(), Err(DevError::Unresolved { .. })));
}

#[test]
fn a_library_only_a_plugin_imports_is_used() {
    let stage = Fake::new(&[
        (PROGRAM, &["Qt6Core.dll"]),
        ("Qt6Core.dll", &[]),
        ("Qt6Svg.dll", &["Qt6Core.dll"]),
        ("imageformats/qsvg.dll", &["Qt6Svg.dll"]),
    ]);
    assert!(stage.check().is_ok());
}

#[test]
fn import_names_match_in_any_letter_case() {
    let stage = Fake::new(&[(PROGRAM, &["QT6CORE.DLL"]), ("Qt6Core.dll", &[])]);
    assert!(stage.check().is_ok());
}

#[test]
fn libraries_that_import_each_other_end_the_walk() {
    let stage = Fake::new(&[
        (PROGRAM, &["Qt6A.dll"]),
        ("Qt6A.dll", &["Qt6B.dll"]),
        ("Qt6B.dll", &["Qt6A.dll"]),
    ]);
    assert!(stage.check().is_ok());
}

#[test]
fn a_stage_without_the_program_is_refused() {
    let stage = Fake::new(&[("Qt6Core.dll", &[])]);
    assert!(matches!(stage.check(), Err(DevError::NoExe(_))));
}

#[test]
fn a_file_that_is_not_a_program_fails_the_check() {
    let stage = Fake::new(&[(PROGRAM, &[])]);
    let read = |p: &Path| Err(DevError::BadPe(paths::text(p)));
    let result = check(stage.dir.path(), &settings(), read, |_| true);
    assert!(matches!(result, Err(DevError::BadPe(_))));
}

#[test]
fn a_link_in_the_stage_is_refused() {
    let stage = Fake::new(&[(PROGRAM, &[])]);
    let outside = TempDir::new("stage-outside");
    make_dir_link(&stage.dir.path().join("plugins"), outside.path());
    assert!(matches!(stage.check(), Err(DevError::Link(_))));
}

#[test]
fn a_stage_over_the_cap_is_refused() {
    let stage = Fake::new(&[(PROGRAM, &[])]);
    let size = usize::try_from(paths::BYTES_PER_MB).unwrap() + 1;
    std::fs::write(stage.dir.path().join(PROGRAM), vec![0u8; size]).unwrap();
    let mut config = settings();
    config.max_mb = 1;
    assert_eq!(
        stage.check_with(&config),
        Err(DevError::TooBig { mb: 2, max: 1 })
    );
    config.max_mb = 2;
    assert!(stage.check_with(&config).is_ok());
}

#[test]
fn the_summary_counts_files_and_bytes() {
    let stage = Fake::new(&[(PROGRAM, &[]), ("qml/QtQuick/qmldir", &[])]);
    std::fs::write(stage.dir.path().join(PROGRAM), [0u8; 7]).unwrap();
    assert_eq!(stage.check(), Ok(Summary { files: 2, bytes: 7 }));
}

#[test]
fn the_summary_rounds_megabytes_up() {
    let summary = Summary {
        files: 1,
        bytes: paths::BYTES_PER_MB + 1,
    };
    assert_eq!(summary.mb(), 2);
}
