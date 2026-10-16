//! D7: a build-only feature, such as kx-theme's `files` that reads media files, never turns on in the app's tree.

use std::collections::{BTreeMap, HashSet};

use super::super::{D7, Severity, Violation};
use crate::workspace::{DependencyKind, Package, Workspace};

/// Cargo's feature that a dependency turns on unless it opts out.
const DEFAULT: &str = "default";

/// D7: one error for each way a crate in the app's normal-dependency tree can turn on a build-only feature: its
/// dependency declaration, one of its own features, or that crate's defaults.
pub(crate) fn d7_build_only_features(ws: &Workspace) -> Vec<Violation> {
    let only = &ws.tidy.build_only_features;
    let mut out = Vec::new();
    for package in app_tree(ws) {
        out.extend(by_dependency(package, only));
        out.extend(by_feature(package, only));
        out.extend(by_default(package, only));
    }
    out
}

/// The app and every package its normal dependencies reach, each once.
fn app_tree(ws: &Workspace) -> Vec<&Package> {
    let Some(app) = ws.find_member(&ws.tidy.app) else {
        return Vec::new();
    };
    let mut seen = HashSet::from([app.id.as_str()]);
    let mut todo = vec![app.id.as_str()];
    let mut tree = Vec::new();
    while let Some(id) = todo.pop() {
        tree.extend(ws.package_by_id(id));
        for edge in ws.graph.get(id).into_iter().flatten() {
            if edge.kind == DependencyKind::Normal && seen.insert(edge.id.as_str()) {
                todo.push(edge.id.as_str());
            }
        }
    }
    tree
}

/// Each normal dependency declaration of `package` that turns on a build-only feature.
fn by_dependency(package: &Package, only: &BTreeMap<String, Vec<String>>) -> Vec<Violation> {
    let normal = package
        .dependencies
        .iter()
        .filter(|d| d.kind == DependencyKind::Normal);
    normal
        .flat_map(|d| {
            d.features
                .iter()
                .map(move |f| (d.name.as_str(), f.as_str()))
        })
        .filter(|(dep, f)| banned(only, dep, f))
        .map(|(dep, f)| violation(package, &format!("{} turns on {dep}/{f}", package.name)))
        .collect()
}

/// Each feature of `package` that turns on a build-only feature of a dependency, as `crate/feature` or
/// `crate?/feature`.
fn by_feature(package: &Package, only: &BTreeMap<String, Vec<String>>) -> Vec<Violation> {
    let mut out = Vec::new();
    for (name, enables) in &package.declared {
        for (dep, f) in enables.iter().filter_map(|item| item.split_once('/')) {
            let dep = dep.trim_end_matches('?');
            if banned(only, dep, f) {
                let what = format!("{}'s feature {name} turns on {dep}/{f}", package.name);
                out.push(violation(package, &what));
            }
        }
    }
    out
}

/// The build-only features of `package` that its own defaults reach.
fn by_default(package: &Package, only: &BTreeMap<String, Vec<String>>) -> Vec<Violation> {
    let Some(features) = only.get(&package.name) else {
        return Vec::new();
    };
    let mut seen = HashSet::new();
    let mut todo = vec![DEFAULT];
    while let Some(name) = todo.pop() {
        if seen.insert(name) {
            todo.extend(
                package
                    .declared
                    .get(name)
                    .into_iter()
                    .flatten()
                    .map(String::as_str),
            );
        }
    }
    features
        .iter()
        .filter(|f| seen.contains(f.as_str()))
        .map(|f| violation(package, &format!("{}'s defaults turn on {f}", package.name)))
        .collect()
}

/// True when `feature` of crate `dep` is build-only.
fn banned(only: &BTreeMap<String, Vec<String>>, dep: &str, feature: &str) -> bool {
    only.get(dep)
        .is_some_and(|fs| fs.iter().any(|f| f == feature))
}

/// One D7 error at `package`, saying `what` turns the feature on.
fn violation(package: &Package, what: &str) -> Violation {
    Violation {
        rule: D7,
        place: package.name.clone(),
        message: format!("{what}, a build-only feature, in the app's normal tree"),
        severity: Severity::Error,
    }
}

#[cfg(test)]
mod tests;
