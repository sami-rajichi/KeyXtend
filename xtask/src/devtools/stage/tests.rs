use super::*;
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
fn the_qmake_variable_must_be_named() {
    assert_eq!(
        invalid(|c| c.qmake_env = String::new()),
        Err(ConfigError::EmptyValue("qmake_env"))
    );
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
fn the_plan_puts_the_program_libraries_plugins_and_qml_in_place() {
    let exe = std::env::temp_dir().join("out").join("keyxtend.exe");
    let (q, c) = (qt(), config());
    let rel = |p: &str| paths::join_rel(PathBuf::new(), p);
    let expected = vec![
        Copy {
            from: exe.clone(),
            to: PathBuf::from(c.program()),
        },
        Copy {
            from: q.bins.join("Qt6Core.dll"),
            to: rel("Qt6Core.dll"),
        },
        Copy {
            from: q.bins.join("Qt6Qml.dll"),
            to: rel("Qt6Qml.dll"),
        },
        Copy {
            from: q.bins.join("Qt6Network.dll"),
            to: rel("Qt6Network.dll"),
        },
        Copy {
            from: paths::join_rel(q.plugins.clone(), "platforms/qwindows.dll"),
            to: rel("platforms/qwindows.dll"),
        },
        Copy {
            from: paths::join_rel(q.qml.clone(), "QtQuick/qmldir"),
            to: rel("qml/QtQuick/qmldir"),
        },
        Copy {
            from: paths::join_rel(q.qml.clone(), "QtQuick/qtquick2plugin.dll"),
            to: rel("qml/QtQuick/qtquick2plugin.dll"),
        },
    ];
    assert_eq!(plan(&c, &q, &exe), expected);
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
