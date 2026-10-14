//! Unit tests for the D7 rule: the app's normal tree never turns on a build-only feature.

use super::*;
use crate::tidy::testkit::{WorkspaceBuilder, dep, dep_with};
use DependencyKind::{Build, Dev, Normal};

/// The app over kx-ui over kx-theme, which declares `files`; kx-ui's dependency on kx-theme turns on `features`.
fn tree(features: &[&str]) -> WorkspaceBuilder {
    WorkspaceBuilder::new()
        .member("keyxtend", "apps", vec![dep("kx-ui", Normal)])
        .member(
            "kx-ui",
            "crates",
            vec![dep_with("kx-theme", Normal, features)],
        )
        .member("kx-theme", "crates", vec![])
        .declares("files", &[])
        .edge("keyxtend", "kx-ui", Normal)
        .edge("kx-ui", "kx-theme", Normal)
}

/// The places and messages of `ws`'s D7 errors.
fn found(ws: &Workspace) -> Vec<(String, String)> {
    let all = d7_build_only_features(ws);
    assert!(
        all.iter()
            .all(|v| v.rule == D7 && v.severity == Severity::Error)
    );
    all.into_iter().map(|v| (v.place, v.message)).collect()
}

#[test]
fn a_normal_dependency_that_turns_it_on_is_a_d7_error() {
    let message = "kx-ui turns on kx-theme/files, a build-only feature, in the app's normal tree";
    assert_eq!(
        found(&tree(&["files"]).build()),
        [("kx-ui".to_string(), message.to_string())]
    );
}

#[test]
fn a_build_or_dev_dependency_beside_a_plain_one_may_turn_it_on() {
    for kind in [Build, Dev] {
        let ws = WorkspaceBuilder::new()
            .member("keyxtend", "apps", vec![dep("kx-ui", Normal)])
            .member(
                "kx-ui",
                "crates",
                vec![
                    dep("kx-theme", Normal),
                    dep_with("kx-theme", kind, &["files"]),
                ],
            )
            .member("kx-theme", "crates", vec![])
            .edge("keyxtend", "kx-ui", Normal)
            .edge("kx-ui", "kx-theme", Normal)
            .edge("kx-ui", "kx-theme", kind)
            .build();
        assert_eq!(found(&ws), []);
    }
}

#[test]
fn a_feature_of_a_crate_in_the_tree_that_turns_it_on_is_a_d7_error() {
    let ws = tree(&[])
        .member("kx-mid", "crates", vec![])
        .declares("sounds", &["kx-theme?/files"])
        .edge("keyxtend", "kx-mid", Normal)
        .build();
    let place = found(&ws).into_iter().map(|(p, _)| p).collect::<Vec<_>>();
    assert_eq!(place, ["kx-mid"]);
}

#[test]
fn defaults_that_reach_it_are_a_d7_error() {
    let ws = tree(&[])
        .declares("default", &["extra"])
        .declares("extra", &["files"])
        .build();
    let message =
        "kx-theme's defaults turn on files, a build-only feature, in the app's normal tree";
    assert_eq!(found(&ws), [("kx-theme".to_string(), message.to_string())]);
}

#[test]
fn a_crate_outside_the_apps_tree_may_turn_it_on() {
    let ws = tree(&[])
        .member("xtask", "", vec![dep_with("kx-theme", Normal, &["files"])])
        .declares("all", &["kx-theme/files"])
        .edge("xtask", "kx-theme", Normal)
        .build();
    assert_eq!(found(&ws), []);
}

#[test]
fn no_app_means_no_errors() {
    let mut ws = tree(&["files"]).build();
    ws.tidy.app = "missing".to_string();
    assert_eq!(found(&ws), []);
}
