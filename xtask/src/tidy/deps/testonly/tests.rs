//! Unit tests for the D5 rule: shipped crates never depend on tools or test-only crates.

use super::*;
use crate::tidy::testkit::{WorkspaceBuilder, dep};
use crate::workspace::{Dependency, DependencyKind};

/// A workspace with a shipped `kx-lib` holding `deps`, plus a tool and both test-only crates.
fn shipped_with(deps: Vec<Dependency>) -> Workspace {
    WorkspaceBuilder::new()
        .member("kx-lib", "crates", deps)
        .member("kx-tool", "tools", vec![])
        .member("kx-test-support", "crates", vec![])
        .member("kx-platform-fake", "crates", vec![])
        .build()
}

#[test]
fn shipped_normal_dependency_on_a_tool_is_a_d5_error() {
    let ws = shipped_with(vec![dep("kx-tool", DependencyKind::Normal)]);
    let violations = d5_test_only_deps(&ws);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].rule, D5);
    assert_eq!(violations[0].severity, Severity::Error);
    assert_eq!(violations[0].place, "kx-lib");
    assert_eq!(
        violations[0].message,
        "kx-lib is not a tool but depends on tool kx-tool"
    );
}

#[test]
fn shipped_normal_dependency_on_a_test_only_crate_is_a_d5_error() {
    for name in ["kx-test-support", "kx-platform-fake"] {
        let ws = shipped_with(vec![dep(name, DependencyKind::Normal)]);
        let violations = d5_test_only_deps(&ws);
        assert_eq!(violations.len(), 1, "{name}: {violations:?}");
        assert_eq!(violations[0].rule, D5);
        assert_eq!(
            violations[0].message,
            format!("kx-lib is not a tool but depends on test-only crate {name}")
        );
    }
}

#[test]
fn shipped_build_dependency_on_a_tool_or_test_only_crate_is_a_d5_error() {
    for name in ["kx-tool", "kx-test-support", "kx-platform-fake"] {
        let ws = shipped_with(vec![dep(name, DependencyKind::Build)]);
        assert_eq!(d5_test_only_deps(&ws).len(), 1, "{name}");
    }
}

#[test]
fn shipped_dev_dependency_on_a_tool_or_test_only_crate_is_fine() {
    let ws = shipped_with(vec![
        dep("kx-tool", DependencyKind::Dev),
        dep("kx-test-support", DependencyKind::Dev),
        dep("kx-platform-fake", DependencyKind::Dev),
    ]);
    assert_eq!(d5_test_only_deps(&ws), vec![]);
}

#[test]
fn one_dependency_listed_as_normal_and_build_is_one_d5_error() {
    let ws = shipped_with(vec![
        dep("kx-test-support", DependencyKind::Normal),
        dep("kx-test-support", DependencyKind::Build),
    ]);
    assert_eq!(d5_test_only_deps(&ws).len(), 1);
}

#[test]
fn the_app_depending_on_a_tool_is_a_d5_error() {
    let ws = WorkspaceBuilder::new()
        .member(
            "keyxtend",
            "apps",
            vec![dep("kx-tool", DependencyKind::Normal)],
        )
        .member("kx-tool", "tools", vec![])
        .build();
    let violations = d5_test_only_deps(&ws);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].place, "keyxtend");
}

#[test]
fn a_test_only_name_is_flagged_even_when_it_is_not_a_workspace_member() {
    let ws = WorkspaceBuilder::new()
        .member(
            "kx-lib",
            "crates",
            vec![dep("kx-test-support", DependencyKind::Normal)],
        )
        .build();
    assert_eq!(d5_test_only_deps(&ws).len(), 1);
}

#[test]
fn a_dependency_on_an_ordinary_crate_is_fine() {
    let ws = shipped_with(vec![
        dep("serde", DependencyKind::Normal),
        dep("kx-platform", DependencyKind::Normal),
    ]);
    assert_eq!(d5_test_only_deps(&ws), vec![]);
}

#[test]
fn a_test_only_crate_may_depend_on_another_test_only_crate() {
    let ws = WorkspaceBuilder::new()
        .member(
            "kx-platform-fake",
            "crates",
            vec![dep("kx-test-support", DependencyKind::Normal)],
        )
        .member("kx-test-support", "crates", vec![])
        .build();
    assert_eq!(d5_test_only_deps(&ws), vec![]);
}

#[test]
fn a_tool_may_depend_on_test_only_crates_and_other_tools() {
    let ws = WorkspaceBuilder::new()
        .member(
            "kx-tool",
            "tools",
            vec![
                dep("kx-test-support", DependencyKind::Normal),
                dep("kx-platform-fake", DependencyKind::Build),
                dep("kx-other-tool", DependencyKind::Normal),
            ],
        )
        .member("kx-other-tool", "tools", vec![])
        .member("kx-test-support", "crates", vec![])
        .member("kx-platform-fake", "crates", vec![])
        .build();
    assert_eq!(d5_test_only_deps(&ws), vec![]);
}
