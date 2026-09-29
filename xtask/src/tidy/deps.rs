//! D1-D5: the dependency-shape rules from `ARCHITECTURE.md`; D5 lives in [`testonly`].

pub(crate) mod testonly;

use std::collections::HashSet;

use super::{D1, D2, D3, D4, Severity, Violation};
use crate::workspace::{DependencyKind, Package, Workspace};

/// D1: a `module_prefix` crate may depend on a workspace member only if it's `module_allowed`.
pub(crate) fn d1_module_isolation(ws: &Workspace) -> Vec<Violation> {
    let mut violations = Vec::new();
    for package in ws.members().filter(|p| is_module(p, ws)) {
        for name in member_deps(package, ws) {
            if !ws.tidy.module_allowed.iter().any(|a| a == name) {
                violations.push(Violation {
                    rule: D1,
                    place: package.name.clone(),
                    message: format!(
                        "{} depends on workspace member {name}, which is not in module_allowed",
                        package.name
                    ),
                    severity: Severity::Error,
                });
            }
        }
    }
    violations
}

/// D2: only the app and the test tools may depend on a module or a platform adapter.
///
/// Modules are skipped, because D1 already covers every workspace dependency they have.
pub(crate) fn d2_app_only_platform_and_module(ws: &Workspace) -> Vec<Violation> {
    let mut violations = Vec::new();
    for package in ws.members() {
        if ws.is_app(package) || ws.is_tool(package) || is_module(package, ws) {
            continue;
        }
        for name in member_deps(package, ws) {
            if name.starts_with(ws.tidy.module_prefix.as_str()) {
                violations.push(d2_violation(package, name, "module"));
            } else if name.starts_with(ws.tidy.platform_prefix.as_str()) {
                violations.push(d2_violation(package, name, "platform adapter"));
            }
        }
    }
    violations
}

/// Builds one D2 violation for `package` depending on `dep_name`.
fn d2_violation(package: &Package, dep_name: &str, kind: &str) -> Violation {
    Violation {
        rule: D2,
        place: package.name.clone(),
        message: format!(
            "{} is not the app but depends on {kind} {dep_name}",
            package.name
        ),
        severity: Severity::Error,
    }
}

/// True if `package` is a feature module: its name starts with `module_prefix`.
fn is_module(package: &Package, ws: &Workspace) -> bool {
    package.name.starts_with(ws.tidy.module_prefix.as_str())
}

/// Names of `package`'s normal and build dependencies, each once.
fn non_dev_deps(package: &Package) -> Vec<&str> {
    let mut seen = HashSet::new();
    package
        .dependencies
        .iter()
        .filter(|d| !matches!(d.kind, DependencyKind::Dev))
        .map(|d| d.name.as_str())
        .filter(|name| seen.insert(*name))
        .collect()
}

/// Names of `package`'s normal and build dependencies that are workspace members, each once.
fn member_deps<'a>(package: &'a Package, ws: &Workspace) -> Vec<&'a str> {
    let mut names = non_dev_deps(package);
    names.retain(|name| ws.is_member(name));
    names
}

/// D3: no `banned_network` crate may be reachable from the app.
pub(crate) fn d3_no_network_in_app(ws: &Workspace) -> Vec<Violation> {
    walk_banned(ws, &ws.tidy.banned_network, D3)
}

/// D4: no `banned_media` crate may be reachable from the app.
pub(crate) fn d4_no_media_in_app(ws: &Workspace) -> Vec<Violation> {
    walk_banned(ws, &ws.tidy.banned_media, D4)
}

/// Walks the app's normal-dependency closure, flagging every `banned` crate reached.
fn walk_banned(ws: &Workspace, banned: &[String], rule: &'static str) -> Vec<Violation> {
    let Some(app) = ws.find_member(&ws.tidy.app) else {
        return Vec::new();
    };
    let mut visited = HashSet::new();
    visited.insert(app.id.clone());
    let mut chain = vec![app.name.clone()];
    let mut out = Vec::new();
    walk_from(
        ws,
        &app.id,
        banned,
        rule,
        &mut visited,
        &mut chain,
        &mut out,
    );
    out
}

/// Recursive step of [`walk_banned`]; never follows the same package id twice.
fn walk_from(
    ws: &Workspace,
    id: &str,
    banned: &[String],
    rule: &'static str,
    visited: &mut HashSet<String>,
    chain: &mut Vec<String>,
    out: &mut Vec<Violation>,
) {
    let Some(edges) = ws.graph.get(id) else {
        return;
    };
    for edge in edges {
        if !matches!(edge.kind, DependencyKind::Normal) || !visited.insert(edge.id.clone()) {
            continue;
        }
        let Some(package) = ws.package_by_id(&edge.id) else {
            continue;
        };
        chain.push(package.name.clone());
        if banned.contains(&package.name) {
            out.push(Violation {
                rule,
                place: package.name.clone(),
                message: chain.join(" → "),
                severity: Severity::Error,
            });
        }
        walk_from(ws, &edge.id, banned, rule, visited, chain, out);
        chain.pop();
    }
}

#[cfg(test)]
mod tests;
