//! The `resolve` section turned into the domain model: dependency edges and turned-on features.

use std::collections::HashMap;

use super::raw::{RawNodeDep, RawResolve};
use super::{DependencyKind, ResolvedDependency, ResolvedGraph};

/// Builds the resolved dependency graph from the `resolve` section.
pub(super) fn graph(resolve: &RawResolve) -> ResolvedGraph {
    resolve
        .nodes
        .iter()
        .map(|node| (node.id.clone(), node.deps.iter().flat_map(edges).collect()))
        .collect()
}

/// The features the resolve turns on, by package id.
pub(super) fn features(resolve: &RawResolve) -> HashMap<String, Vec<String>> {
    resolve
        .nodes
        .iter()
        .map(|node| (node.id.clone(), node.features.clone()))
        .collect()
}

/// Expands one resolved dependency into one edge per kind it's used as.
fn edges(dep: &RawNodeDep) -> Vec<ResolvedDependency> {
    if dep.dep_kinds.is_empty() {
        return vec![ResolvedDependency {
            id: dep.pkg.clone(),
            kind: DependencyKind::Normal,
        }];
    }
    dep.dep_kinds
        .iter()
        .map(|dk| ResolvedDependency {
            id: dep.pkg.clone(),
            kind: DependencyKind::from_raw(dk.kind.as_deref()),
        })
        .collect()
}
