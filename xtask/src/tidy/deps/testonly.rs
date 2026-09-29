//! D5: shipped crates never depend on test tools or test-only crates.

use super::super::{D5, Severity, Violation};
use super::non_dev_deps;
use crate::workspace::{Package, Workspace};

/// D5: a normal or build dependency on a tool or a `test_only` crate is allowed only for those.
pub(crate) fn d5_test_only_deps(ws: &Workspace) -> Vec<Violation> {
    let mut violations = Vec::new();
    for package in ws.members().filter(|p| !may_use_test_code(p, ws)) {
        for name in non_dev_deps(package) {
            if let Some(kind) = test_code_kind(name, ws) {
                violations.push(d5_violation(package, name, kind));
            }
        }
    }
    violations
}

/// True if `package` is a tool or a `test_only` crate, so it may depend on test code.
fn may_use_test_code(package: &Package, ws: &Workspace) -> bool {
    ws.is_tool(package) || is_test_only(&package.name, ws)
}

/// True if `name` is listed in `test_only`.
fn is_test_only(name: &str, ws: &Workspace) -> bool {
    ws.tidy.test_only.iter().any(|t| t == name)
}

/// What kind of test code `name` is (`tool` or `test-only crate`), or `None` if it is ordinary.
fn test_code_kind(name: &str, ws: &Workspace) -> Option<&'static str> {
    if ws.find_member(name).is_some_and(|p| ws.is_tool(p)) {
        Some("tool")
    } else if is_test_only(name, ws) {
        Some("test-only crate")
    } else {
        None
    }
}

/// Builds one D5 violation for `package` depending on `dep_name`.
fn d5_violation(package: &Package, dep_name: &str, kind: &str) -> Violation {
    Violation {
        rule: D5,
        place: package.name.clone(),
        message: format!(
            "{} is not a tool but depends on {kind} {dep_name}",
            package.name
        ),
        severity: Severity::Error,
    }
}

#[cfg(test)]
mod tests;
