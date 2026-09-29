//! Tests for F1 and F2 on test tools: they are scanned, and may carry the deny attribute.

use std::fs;

use super::*;
use crate::test_support::unique_temp_dir;
use crate::tidy::testkit::WorkspaceBuilder;

#[test]
fn long_file_under_tools_is_an_f1_error() {
    let root = unique_temp_dir("f1-tools");
    fs::create_dir_all(root.join("tools/kx-tool/src")).unwrap();
    fs::write(
        root.join("tools/kx-tool/src/big.rs"),
        "// a line\n".repeat(401),
    )
    .unwrap();

    let ws = WorkspaceBuilder::new().root(root.clone()).build();
    let violations = f1_file_length(&ws);

    fs::remove_dir_all(&root).ok();

    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].rule, F1);
    assert_eq!(violations[0].severity, Severity::Error);
    assert_eq!(violations[0].place, "tools/kx-tool/src/big.rs");
}

/// Runs F2 on one package whose lib root holds `source`, under `sub_dir` (`tools` or `crates`).
fn f2_for_lib(label: &str, sub_dir: &str, source: &str) -> Vec<Violation> {
    let root = unique_temp_dir(label);
    let rel = format!("{sub_dir}/kx-foo/src/lib.rs");
    fs::create_dir_all(root.join(&rel).parent().unwrap()).unwrap();
    fs::write(root.join(&rel), source).unwrap();

    let ws = WorkspaceBuilder::new()
        .root(root.clone())
        .member("kx-foo", sub_dir, vec![])
        .target(&["lib"], &rel)
        .build();
    let violations = f2_unsafe_attr(&ws);

    fs::remove_dir_all(&root).ok();
    violations
}

#[test]
fn tool_root_with_the_deny_attr_passes() {
    let violations = f2_for_lib(
        "f2-tool-deny",
        "tools",
        "#![deny(unsafe_code)]\nfn x() {}\n",
    );
    assert_eq!(violations, vec![]);
}

#[test]
fn tool_root_with_the_forbid_attr_passes() {
    let violations = f2_for_lib(
        "f2-tool-forbid",
        "tools",
        "#![forbid(unsafe_code)]\nfn x() {}\n",
    );
    assert_eq!(violations, vec![]);
}

#[test]
fn tool_root_with_neither_attr_is_an_f2_error() {
    let violations = f2_for_lib("f2-tool-none", "tools", "fn x() {}\n");
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].rule, F2);
    assert_eq!(
        violations[0].message,
        "missing #![deny(unsafe_code)] or #![forbid(unsafe_code)]"
    );
}

#[test]
fn non_tool_root_with_only_the_deny_attr_is_an_f2_error() {
    let violations = f2_for_lib("f2-deny", "crates", "#![deny(unsafe_code)]\nfn x() {}\n");
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].rule, F2);
    assert_eq!(violations[0].message, "missing #![forbid(unsafe_code)]");
}
