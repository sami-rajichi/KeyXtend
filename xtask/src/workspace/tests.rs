//! Unit tests for [`super::load`], against small hand-written `cargo metadata` fixtures.

use super::*;

const MINIMAL: &str = include_str!("../../tests/fixtures/minimal.json");
const MISSING_TIDY: &str = include_str!("../../tests/fixtures/missing_tidy.json");
const MISSING_DCO: &str = include_str!("../../tests/fixtures/missing_dco.json");
const BAD_CONFIG_TYPE: &str = include_str!("../../tests/fixtures/bad_config_type.json");
const BAD_CONFIG_WARN_GT_MAX: &str =
    include_str!("../../tests/fixtures/bad_config_warn_gt_max.json");
const NULL_RESOLVE: &str = include_str!("../../tests/fixtures/null_resolve.json");
const INVALID: &str = include_str!("../../tests/fixtures/invalid.json");

#[test]
fn minimal_fixture_loads_the_package_and_config() {
    let workspace = load(MINIMAL).unwrap();
    assert_eq!(
        workspace.member_ids,
        vec!["path+file:///D:/Projects/KeyXtend/xtask#0.0.0"]
    );
    assert_eq!(workspace.packages.len(), 1);

    let package = &workspace.packages[0];
    assert_eq!(package.name, "xtask");
    assert_eq!(
        package.targets,
        vec![Target {
            kinds: vec!["bin".to_string()],
            src_path: PathBuf::from("D:\\Projects\\KeyXtend\\xtask\\src\\main.rs"),
        }]
    );
    assert_eq!(
        package.dependencies,
        vec![
            Dependency {
                name: "serde".into(),
                kind: DependencyKind::Normal
            },
            Dependency {
                name: "proptest".into(),
                kind: DependencyKind::Dev
            },
        ]
    );

    let edges = &workspace.graph[&package.id];
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].kind, DependencyKind::Normal);

    assert_eq!(workspace.tidy.max_file_lines, 400);
    assert_eq!(workspace.dco.trailer, "Signed-off-by");
}

#[test]
fn missing_tidy_section_names_it() {
    let err = load(MISSING_TIDY).unwrap_err();
    assert!(
        matches!(err, LoadError::MissingSection("tidy")),
        "got: {err}"
    );
}

#[test]
fn missing_dco_section_names_it() {
    let err = load(MISSING_DCO).unwrap_err();
    assert!(
        matches!(err, LoadError::MissingSection("dco")),
        "got: {err}"
    );
}

#[test]
fn wrong_field_type_is_a_bad_config_error() {
    let err = load(BAD_CONFIG_TYPE).unwrap_err();
    assert!(matches!(err, LoadError::BadConfig(_)), "got: {err}");
}

#[test]
fn warn_greater_than_max_is_a_bad_config_error() {
    let err = load(BAD_CONFIG_WARN_GT_MAX).unwrap_err();
    assert!(matches!(err, LoadError::BadConfig(_)), "got: {err}");
}

#[test]
fn null_resolve_is_a_missing_resolve_error() {
    let err = load(NULL_RESOLVE).unwrap_err();
    assert!(matches!(err, LoadError::MissingResolve), "got: {err}");
}

#[test]
fn invalid_json_is_a_json_error() {
    let err = load(INVALID).unwrap_err();
    assert!(matches!(err, LoadError::Json(_)), "got: {err}");
}

#[test]
fn load_never_panics_on_empty_input() {
    assert!(load("").is_err());
    assert!(load("null").is_err());
    assert!(load("{}").is_err());
}
