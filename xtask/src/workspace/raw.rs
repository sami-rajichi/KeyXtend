//! Field-for-field mirror of `cargo metadata --format-version 1` JSON.
//!
//! Kept separate from the domain model in the parent module: these structs
//! only describe the wire shape, with no validation or conversion logic.

use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::{Map, Value};

/// Top-level `cargo metadata` output, trimmed to the fields tidy and DCO use.
#[derive(Deserialize)]
pub(super) struct RawOutput {
    pub(super) packages: Vec<RawPackage>,
    pub(super) workspace_members: Vec<String>,
    pub(super) workspace_root: String,
    #[serde(default)]
    pub(super) resolve: Option<RawResolve>,
    /// The `workspace.metadata` table, left as raw JSON so a bad section is reported by name.
    #[serde(default)]
    pub(super) metadata: Option<Map<String, Value>>,
}

/// One package entry, workspace member or dependency.
#[derive(Deserialize)]
pub(super) struct RawPackage {
    pub(super) id: String,
    pub(super) name: String,
    pub(super) manifest_path: String,
    #[serde(default)]
    pub(super) dependencies: Vec<RawDependency>,
    #[serde(default)]
    pub(super) targets: Vec<RawTarget>,
    /// The features its manifest declares, each with what it turns on.
    #[serde(default)]
    pub(super) features: BTreeMap<String, Vec<String>>,
}

/// A dependency as declared in a package's manifest.
#[derive(Deserialize)]
pub(super) struct RawDependency {
    pub(super) name: String,
    #[serde(default)]
    pub(super) kind: Option<String>,
    /// The features the declaration turns on in it.
    #[serde(default)]
    pub(super) features: Vec<String>,
}

/// A build target's kinds and root source file.
#[derive(Deserialize)]
pub(super) struct RawTarget {
    #[serde(default)]
    pub(super) kind: Vec<String>,
    pub(super) src_path: String,
}

/// The resolved dependency graph.
#[derive(Deserialize)]
pub(super) struct RawResolve {
    pub(super) nodes: Vec<RawNode>,
}

/// One package's resolved dependencies and turned-on features.
#[derive(Deserialize)]
pub(super) struct RawNode {
    pub(super) id: String,
    #[serde(default)]
    pub(super) deps: Vec<RawNodeDep>,
    #[serde(default)]
    pub(super) features: Vec<String>,
}

/// One resolved dependency edge, before it is split by kind.
#[derive(Deserialize)]
pub(super) struct RawNodeDep {
    pub(super) pkg: String,
    #[serde(default)]
    pub(super) dep_kinds: Vec<RawDepKind>,
}

/// The kind half of a resolved dependency edge.
#[derive(Deserialize)]
pub(super) struct RawDepKind {
    #[serde(default)]
    pub(super) kind: Option<String>,
}

/// Just the workspace root and the raw `workspace.metadata` table, for loading one section.
#[derive(Deserialize)]
pub(super) struct RawSections {
    pub(super) workspace_root: String,
    #[serde(default)]
    pub(super) metadata: Option<Map<String, Value>>,
}

/// Just cargo's output folder.
#[derive(Deserialize)]
pub(super) struct RawTargetDir {
    pub(super) target_directory: String,
}
