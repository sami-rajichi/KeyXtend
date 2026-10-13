//! Unit tests for the D6 rule: no package turns on a banned feature.

use super::*;
use crate::tidy::testkit::{WorkspaceBuilder, dep};
use crate::tidy::{D6, Severity};
use crate::workspace::DependencyKind;

#[test]
fn a_banned_feature_turned_on_is_a_d6_error() {
    let ws = WorkspaceBuilder::new()
        .external("toml")
        .features(&["default", "parse", "unbounded"])
        .build();
    assert_eq!(
        d6_banned_features(&ws),
        vec![Violation {
            rule: D6,
            place: "toml".to_string(),
            message: "toml has banned feature unbounded turned on".to_string(),
            severity: Severity::Error,
        }]
    );
}

#[test]
fn allowed_features_and_other_crates_pass() {
    let ws = WorkspaceBuilder::new()
        .external("toml")
        .features(&["default", "parse"])
        .external("other")
        .features(&["unbounded"])
        .build();
    assert_eq!(d6_banned_features(&ws), vec![]);
}

#[test]
fn a_workspace_member_and_each_copy_of_a_crate_are_checked() {
    let mut ws = WorkspaceBuilder::new()
        .member(
            "kx-lib",
            "crates",
            vec![dep("toml", DependencyKind::Normal)],
        )
        .features(&["fast"])
        .external("toml")
        .features(&["unbounded"])
        .external("toml")
        .features(&["unbounded"])
        .build();
    ws.tidy
        .banned_features
        .insert("kx-lib".to_string(), vec!["fast".to_string()]);
    let places: Vec<String> = d6_banned_features(&ws)
        .into_iter()
        .map(|v| v.place)
        .collect();
    assert_eq!(places, ["kx-lib", "toml", "toml"]);
}

#[test]
fn no_bans_means_no_errors() {
    let mut ws = WorkspaceBuilder::new()
        .external("toml")
        .features(&["unbounded"])
        .build();
    ws.tidy.banned_features.clear();
    assert_eq!(d6_banned_features(&ws), vec![]);
}
