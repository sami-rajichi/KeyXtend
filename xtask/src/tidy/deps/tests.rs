//! Unit tests for the D1-D4 dependency rules (plan edge cases 1-7).

use super::*;
use crate::tidy::testkit::{WorkspaceBuilder, dep};
use crate::workspace::DependencyKind;

#[test]
fn kx_mod_depending_on_another_module_is_a_d1_error() {
    let ws = WorkspaceBuilder::new()
        .member(
            "kx-mod-a",
            "crates",
            vec![dep("kx-mod-b", DependencyKind::Normal)],
        )
        .member("kx-mod-b", "crates", vec![])
        .build();
    let violations = d1_module_isolation(&ws);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].rule, D1);
    assert_eq!(violations[0].place, "kx-mod-a");
}

#[test]
fn kx_mod_depending_on_kernel_or_platform_is_a_d1_error() {
    let ws = WorkspaceBuilder::new()
        .member(
            "kx-mod-a",
            "crates",
            vec![
                dep("kx-kernel", DependencyKind::Normal),
                dep("kx-platform-windows", DependencyKind::Normal),
            ],
        )
        .member("kx-kernel", "crates", vec![])
        .member("kx-platform-windows", "crates", vec![])
        .build();
    let violations = d1_module_isolation(&ws);
    assert_eq!(violations.len(), 2);
    assert!(violations.iter().all(|v| v.rule == D1));
    assert!(violations.iter().any(|v| v.message.contains("kx-kernel")));
    assert!(
        violations
            .iter()
            .any(|v| v.message.contains("kx-platform-windows"))
    );
}

#[test]
fn kx_mod_dev_dependency_on_platform_is_not_a_d1_violation() {
    let ws = WorkspaceBuilder::new()
        .member(
            "kx-mod-a",
            "crates",
            vec![dep("kx-platform-fake", DependencyKind::Dev)],
        )
        .member("kx-platform-fake", "crates", vec![])
        .build();
    assert_eq!(d1_module_isolation(&ws), vec![]);
}

#[test]
fn kx_mod_listing_one_dependency_as_normal_and_build_is_one_d1_error() {
    let ws = WorkspaceBuilder::new()
        .member(
            "kx-mod-a",
            "crates",
            vec![
                dep("kx-kernel", DependencyKind::Normal),
                dep("kx-kernel", DependencyKind::Build),
            ],
        )
        .member("kx-kernel", "crates", vec![])
        .build();
    assert_eq!(d1_module_isolation(&ws).len(), 1);
}

#[test]
fn non_app_depending_on_a_module_is_a_d2_error() {
    let ws = WorkspaceBuilder::new()
        .member(
            "kx-lib",
            "crates",
            vec![dep("kx-mod-a", DependencyKind::Normal)],
        )
        .member("kx-mod-a", "crates", vec![])
        .build();
    let violations = d2_app_only_platform_and_module(&ws);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].rule, D2);
    assert_eq!(violations[0].place, "kx-lib");
}

#[test]
fn platform_depending_on_another_platform_is_a_d2_error() {
    let ws = WorkspaceBuilder::new()
        .member(
            "kx-platform-a",
            "crates",
            vec![dep("kx-platform-b", DependencyKind::Normal)],
        )
        .member("kx-platform-b", "crates", vec![])
        .build();
    let violations = d2_app_only_platform_and_module(&ws);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].rule, D2);
    assert_eq!(violations[0].place, "kx-platform-a");
}

#[test]
fn platform_depending_on_the_platform_ports_crate_is_not_a_d2_violation() {
    // `kx-platform` has no trailing dash, so it does not match `platform_prefix`.
    let ws = WorkspaceBuilder::new()
        .member(
            "kx-platform-windows",
            "crates",
            vec![dep("kx-platform", DependencyKind::Normal)],
        )
        .member("kx-platform", "crates", vec![])
        .build();
    assert_eq!(d2_app_only_platform_and_module(&ws), vec![]);
}

#[test]
fn non_app_depending_on_platform_is_a_d2_error() {
    let ws = WorkspaceBuilder::new()
        .member(
            "kx-lib",
            "crates",
            vec![dep("kx-platform-windows", DependencyKind::Normal)],
        )
        .member("kx-platform-windows", "crates", vec![])
        .build();
    let violations = d2_app_only_platform_and_module(&ws);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].rule, D2);
}

#[test]
fn app_depending_on_platform_is_not_a_d2_violation() {
    let ws = WorkspaceBuilder::new()
        .member(
            "keyxtend",
            "apps",
            vec![dep("kx-platform-windows", DependencyKind::Normal)],
        )
        .member("kx-platform-windows", "crates", vec![])
        .build();
    assert_eq!(d2_app_only_platform_and_module(&ws), vec![]);
}

#[test]
fn transitive_network_dependency_of_app_is_a_d3_error() {
    let ws = WorkspaceBuilder::new()
        .member("keyxtend", "apps", vec![])
        .member("kx-foo", "crates", vec![])
        .external("reqwest")
        .edge("keyxtend", "kx-foo", DependencyKind::Normal)
        .edge("kx-foo", "reqwest", DependencyKind::Normal)
        .build();
    let violations = d3_no_network_in_app(&ws);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].rule, D3);
    assert_eq!(violations[0].severity, Severity::Error);
    assert_eq!(violations[0].message, "keyxtend → kx-foo → reqwest");
}

#[test]
fn network_dependency_outside_app_closure_is_not_a_d3_violation() {
    let ws = WorkspaceBuilder::new()
        .member("keyxtend", "apps", vec![])
        .member("keyxtend-worker", "apps", vec![])
        .external("reqwest")
        .edge("keyxtend-worker", "reqwest", DependencyKind::Normal)
        .build();
    assert_eq!(d3_no_network_in_app(&ws), vec![]);
}

#[test]
fn network_dev_dependency_of_app_is_not_a_d3_violation() {
    let ws = WorkspaceBuilder::new()
        .member("keyxtend", "apps", vec![])
        .external("reqwest")
        .edge("keyxtend", "reqwest", DependencyKind::Dev)
        .build();
    assert_eq!(d3_no_network_in_app(&ws), vec![]);
}

#[test]
fn network_build_dependency_of_app_is_not_a_d3_violation() {
    let ws = WorkspaceBuilder::new()
        .member("keyxtend", "apps", vec![])
        .external("reqwest")
        .edge("keyxtend", "reqwest", DependencyKind::Build)
        .build();
    assert_eq!(d3_no_network_in_app(&ws), vec![]);
}

#[test]
fn transitive_media_dependency_of_app_is_a_d4_error() {
    let ws = WorkspaceBuilder::new()
        .member("keyxtend", "apps", vec![])
        .external("image")
        .edge("keyxtend", "image", DependencyKind::Normal)
        .build();
    let violations = d4_no_media_in_app(&ws);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].rule, D4);
    assert_eq!(violations[0].severity, Severity::Error);
    assert_eq!(violations[0].message, "keyxtend → image");
}

#[test]
fn workspace_with_only_xtask_has_no_dependency_violations() {
    let ws = WorkspaceBuilder::new().member("xtask", "", vec![]).build();
    assert!(d1_module_isolation(&ws).is_empty());
    assert!(d2_app_only_platform_and_module(&ws).is_empty());
    assert!(d3_no_network_in_app(&ws).is_empty());
    assert!(d4_no_media_in_app(&ws).is_empty());
}
