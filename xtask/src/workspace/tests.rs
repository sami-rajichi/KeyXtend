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
        vec!["path+file:///C:/repo/xtask#0.0.0"]
    );
    assert_eq!(workspace.packages.len(), 1);

    let package = &workspace.packages[0];
    assert_eq!(package.name, "xtask");
    assert_eq!(
        package.targets,
        vec![Target {
            kinds: vec!["bin".to_string()],
            src_path: PathBuf::from("C:\\repo\\xtask\\src\\main.rs"),
        }]
    );
    assert_eq!(
        package.dependencies,
        vec![
            Dependency {
                name: "serde".into(),
                kind: DependencyKind::Normal,
                features: vec!["derive".into()],
            },
            Dependency {
                name: "proptest".into(),
                kind: DependencyKind::Dev,
                features: Vec::new(),
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
fn minimal_fixture_loads_the_resolved_features_and_the_banned_ones() {
    let workspace = load(MINIMAL).unwrap();
    assert_eq!(workspace.packages[0].features, vec!["default".to_string()]);
    let banned = &workspace.tidy.banned_features["toml"];
    assert_eq!(banned, &vec!["unbounded".to_string()]);
    let build_only = &workspace.tidy.build_only_features["kx-theme"];
    assert_eq!(build_only, &vec!["files".to_string()]);
}

#[test]
fn minimal_fixture_loads_the_declared_feature_table() {
    let workspace = load(MINIMAL).unwrap();
    let declared = &workspace.packages[0].declared;
    assert_eq!(declared["default"], vec!["std".to_string()]);
    assert!(declared["std"].is_empty());
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

#[test]
fn app_and_tool_follow_the_manifest_folder() {
    use crate::tidy::testkit::WorkspaceBuilder;

    let ws = WorkspaceBuilder::new()
        .member("keyxtend", "apps", vec![])
        .member("kx-tool", "tools", vec![])
        .member("kx-lib", "crates", vec![])
        .build();
    let flags = |name: &str| {
        let package = ws.find_member(name).unwrap();
        (ws.is_app(package), ws.is_tool(package))
    };
    assert_eq!(flags("keyxtend"), (true, false));
    assert_eq!(flags("kx-tool"), (false, true));
    assert_eq!(flags("kx-lib"), (false, false));
}

#[test]
fn target_dir_reads_cargo_output_folder() {
    let json = r#"{"workspace_root": "/ws", "target_directory": "/ws/target"}"#;
    assert_eq!(target_dir(json).unwrap(), PathBuf::from("/ws/target"));
}

#[test]
fn target_dir_without_the_folder_is_a_json_error() {
    let err = target_dir(r#"{"workspace_root": "/ws"}"#).unwrap_err();
    assert!(matches!(err, LoadError::Json(_)), "got: {err}");
}
