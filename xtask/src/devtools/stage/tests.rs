use super::*;
use crate::config::ConfigError;
use crate::test_support::{TempDir, stage_config as config, stage_json};

/// The error `validate` gives after `change` is applied to the test settings.
fn invalid(change: impl FnOnce(&mut StageConfig)) -> Result<(), ConfigError> {
    let mut c = config();
    change(&mut c);
    c.validate()
}

/// Qt's folders under one made-up install.
fn qt() -> QtDirs {
    let root = std::env::temp_dir().join("qt");
    QtDirs {
        version: "6.11.2".to_string(),
        bins: root.join("bin"),
        plugins: root.join("plugins"),
        qml: root.join("qml"),
    }
}

#[test]
fn test_settings_pass_validation() {
    assert_eq!(config().validate(), Ok(()));
}

#[test]
fn the_cap_must_not_be_zero() {
    assert_eq!(
        invalid(|c| c.max_mb = 0),
        Err(ConfigError::ZeroValue("max_mb"))
    );
}

#[test]
fn names_must_be_plain() {
    assert_eq!(
        invalid(|c| c.app = "../x".to_string()),
        Err(ConfigError::NotPlain("app"))
    );
    assert_eq!(
        invalid(|c| c.worker = "bin/w".to_string()),
        Err(ConfigError::NotPlain("worker"))
    );
    assert_eq!(
        invalid(|c| c.libs = vec!["bin/Qt6Core.dll".to_string()]),
        Err(ConfigError::NotPlain("libs"))
    );
    assert_eq!(
        invalid(|c| c.banned = vec!["a/b".to_string()]),
        Err(ConfigError::NotPlain("banned"))
    );
}

#[test]
fn folders_must_be_relative_and_plain() {
    assert_eq!(
        invalid(|c| c.out_dir = "../out".to_string()),
        Err(ConfigError::NotRelative("out_dir"))
    );
    assert_eq!(
        invalid(|c| c.plugins = vec!["../platforms/q.dll".to_string()]),
        Err(ConfigError::NotRelative("plugins"))
    );
    assert_eq!(
        invalid(|c| c.qml = vec!["C:/qml/qmldir".to_string()]),
        Err(ConfigError::NotRelative("qml"))
    );
}

#[test]
fn required_lists_must_not_be_empty() {
    assert_eq!(
        invalid(|c| c.libs.clear()),
        Err(ConfigError::EmptyList("libs"))
    );
    assert_eq!(
        invalid(|c| c.banned.clear()),
        Err(ConfigError::EmptyList("banned"))
    );
    assert_eq!(
        invalid(|c| c.api_sets = vec![" ".to_string()]),
        Err(ConfigError::EmptyValue("api_sets"))
    );
}

#[test]
fn the_qmake_variable_and_qt_version_must_be_named() {
    assert_eq!(
        invalid(|c| c.qmake_env = String::new()),
        Err(ConfigError::EmptyValue("qmake_env"))
    );
    assert_eq!(
        invalid(|c| c.qt_version = " ".to_string()),
        Err(ConfigError::EmptyValue("qt_version"))
    );
}

#[test]
fn the_gate_flag_must_be_named() {
    assert_eq!(
        invalid(|c| c.gate_flag = " ".to_string()),
        Err(ConfigError::EmptyValue("gate_flag"))
    );
}

#[test]
fn the_network_library_and_its_importers_must_be_plain_names() {
    assert_eq!(
        invalid(|c| c.network.lib = "bin/Qt6Network.dll".to_string()),
        Err(ConfigError::NotPlain("network"))
    );
    assert_eq!(
        invalid(|c| c.network.by.clear()),
        Err(ConfigError::EmptyList("network"))
    );
    assert_eq!(
        invalid(|c| c.network.by = vec!["../Qt6Qml.dll".to_string()]),
        Err(ConfigError::NotPlain("network"))
    );
}

#[test]
fn the_network_library_and_its_importers_must_be_staged_libraries() {
    let unlisted = Err(ConfigError::NotListed("network", "libs"));
    assert_eq!(
        invalid(|c| c.network.lib = "Qt6Net.dll".to_string()),
        unlisted
    );
    assert_eq!(
        invalid(|c| c.network.by.push("Qt6Gui.dll".to_string())),
        unlisted
    );
}

#[test]
fn libraries_and_plugins_must_be_dlls() {
    assert_eq!(
        invalid(|c| c.libs.push("helper.exe".to_string())),
        Err(ConfigError::NotType("libs", check::LIB_EXT))
    );
    assert_eq!(
        invalid(|c| c.plugins = vec!["platforms/qwindows".to_string()]),
        Err(ConfigError::NotType("plugins", check::LIB_EXT))
    );
    assert_eq!(invalid(|c| c.libs.push("Qt6Svg.DLL".to_string())), Ok(()));
}

#[test]
fn load_reads_the_section() {
    let json =
        serde_json::json!({ "workspace_root": "/ws", "metadata": { "stage": stage_json() } });
    assert_eq!(load(&json.to_string()).unwrap(), config());
}

#[test]
fn load_names_a_missing_section() {
    let json = r#"{"workspace_root": "/ws", "metadata": {}}"#;
    assert!(matches!(
        load(json),
        Err(LoadError::MissingSection("stage"))
    ));
}

#[test]
fn unknown_field_is_rejected() {
    let mut value = stage_json();
    value["bogus"] = serde_json::json!(1);
    assert!(serde_json::from_value::<StageConfig>(value).is_err());
}

#[test]
fn the_program_is_named_after_the_app() {
    let expected = format!("keyxtend{}", std::env::consts::EXE_SUFFIX);
    assert_eq!(config().program(), expected);
}

#[test]
fn the_worker_program_is_named_after_its_package() {
    let expected = format!("keyxtend-worker{}", std::env::consts::EXE_SUFFIX);
    assert_eq!(config().worker_program(), expected);
}

#[test]
fn the_plan_puts_the_program_worker_libraries_plugins_and_qml_in_place() {
    let release = std::env::temp_dir().join("out");
    let (q, c) = (qt(), config());
    let rel = |p: &str| paths::join_rel(PathBuf::new(), p);
    let copy = |from: PathBuf, to: &str| Copy { from, to: rel(to) };
    let built = |exe: &str| copy(release.join(exe), exe);
    let lib = |l: &str| copy(q.bins.join(l), l);
    let qml = |q_rel: &str, to: &str| copy(paths::join_rel(q.qml.clone(), q_rel), to);
    let expected = vec![
        built(&c.program()),
        built(&c.worker_program()),
        lib("Qt6Core.dll"),
        lib("Qt6Qml.dll"),
        lib("Qt6Quick.dll"),
        lib("Qt6Network.dll"),
        copy(
            paths::join_rel(q.plugins.clone(), "platforms/qwindows.dll"),
            "platforms/qwindows.dll",
        ),
        qml("QtQuick/qmldir", "qml/QtQuick/qmldir"),
        qml(
            "QtQuick/qtquick2plugin.dll",
            "qml/QtQuick/qtquick2plugin.dll",
        ),
    ];
    assert_eq!(plan(&c, &q, &release), expected);
}

#[test]
fn copy_empties_the_stage_then_copies_every_file() {
    let dir = TempDir::new("copy");
    let source = dir.path().join("a.dll");
    std::fs::write(&source, b"abc").unwrap();
    let stage = dir.path().join("stage");
    std::fs::create_dir_all(&stage).unwrap();
    std::fs::write(stage.join("old.dll"), b"").unwrap();
    let to = Path::new("sub").join("a.dll");
    let files = [Copy {
        from: source,
        to: to.clone(),
    }];
    copy(&stage, &files).unwrap();
    assert_eq!(paths::files(&stage), Ok(vec![to.clone()]));
    assert_eq!(std::fs::read(stage.join(to)).unwrap(), b"abc");
}

#[test]
fn a_missing_source_names_the_file() {
    let dir = TempDir::new("copy-missing");
    let from = dir.path().join("Qt6Missing.dll");
    let files = [Copy {
        from,
        to: PathBuf::from("Qt6Missing.dll"),
    }];
    let err = copy(&dir.path().join("stage"), &files).unwrap_err();
    assert!(err.to_string().contains("Qt6Missing.dll"), "{err}");
}

/// A temp folder with one source file `a.dll` and the stage path, not yet made.
fn fill_case(label: &str) -> (TempDir, PathBuf, [Copy; 1]) {
    let dir = TempDir::new(label);
    let from = dir.path().join("a.dll");
    std::fs::write(&from, b"abc").unwrap();
    let stage = dir.path().join("stage");
    let files = [Copy {
        from,
        to: PathBuf::from("a.dll"),
    }];
    (dir, stage, files)
}

/// A summary of `files` files of `bytes` bytes each way, with no packs.
fn summary(files: usize, bytes: u64) -> check::Summary {
    let deflated = bytes;
    check::Summary {
        files,
        size: check::Size { bytes, deflated },
        packs: check::Size::default(),
    }
}

#[test]
fn a_passing_stage_stays() {
    let (_dir, stage, files) = fill_case("fill-pass");
    let done = fill(&stage, &files, |_| Ok(summary(1, 3)));
    assert_eq!(done, Ok(summary(1, 3)));
    assert!(stage.join("a.dll").is_file());
}

#[test]
fn a_refused_stage_is_removed_and_says_so() {
    let (_dir, stage, files) = fill_case("fill-refused");
    let banned = DevError::Stage(StageError::Banned("tls/x.dll".to_string()));
    let err = fill(&stage, &files, |_| Err(banned.clone())).unwrap_err();
    assert_eq!(err, DevError::StageRemoved(Box::new(banned)));
    assert!(err.to_string().contains("removed"), "{err}");
    assert!(!stage.exists());
}

#[test]
fn a_failed_copy_leaves_no_stage() {
    let (dir, stage, _) = fill_case("fill-copy");
    let from = dir.path().join("missing.dll");
    let files = [Copy {
        from,
        to: PathBuf::from("missing.dll"),
    }];
    let err = fill(&stage, &files, |_| Ok(summary(0, 0))).unwrap_err();
    assert!(matches!(err, DevError::StageRemoved(_)), "{err}");
    assert!(!stage.exists());
}
