//! The workspace model that tidy and DCO checks run against.

mod raw;

use std::fmt;
use std::path::PathBuf;

use crate::config::{ConfigError, DcoConfig, TidyConfig};

/// The `cargo` arguments that print the workspace as JSON for [`load`].
const METADATA_ARGS: [&str; 3] = ["metadata", "--format-version", "1"];

/// How a package uses one of its dependencies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DependencyKind {
    /// Used by the package's normal build.
    Normal,
    /// Used only by tests, examples or benches.
    Dev,
    /// Used only by a build script.
    Build,
}

impl DependencyKind {
    /// Maps a `cargo metadata` kind string (`null`, `"dev"` or `"build"`).
    fn from_raw(kind: Option<&str>) -> Self {
        match kind {
            Some("dev") => Self::Dev,
            Some("build") => Self::Build,
            _ => Self::Normal,
        }
    }
}

/// A dependency as declared in a package's manifest.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Dependency {
    /// The dependency's crate name.
    pub name: String,
    /// How the package uses it.
    pub kind: DependencyKind,
}

/// One build target of a package: its kinds (`lib`, `bin`, `test`, ...) and root source file.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Target {
    /// The target's `cargo metadata` kinds, e.g. `["lib"]` or `["bin"]`.
    pub kinds: Vec<String>,
    /// The target's root source file.
    pub src_path: PathBuf,
}

/// One workspace or external package from `cargo metadata`.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Package {
    /// The package's unique `cargo metadata` id.
    pub id: String,
    /// The crate name.
    pub name: String,
    /// Path to the package's `Cargo.toml`.
    pub manifest_path: PathBuf,
    /// Dependencies declared in the manifest.
    pub dependencies: Vec<Dependency>,
    /// The package's build targets.
    pub targets: Vec<Target>,
}

/// A resolved dependency edge: another package id, with the kind it's used as.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ResolvedDependency {
    /// The id of the package depended on.
    pub id: String,
    /// How it's used.
    pub kind: DependencyKind,
}

/// The resolved dependency graph: package id to its resolved dependencies.
pub(crate) type ResolvedGraph = std::collections::HashMap<String, Vec<ResolvedDependency>>;

/// The full workspace model tidy and DCO checks run against.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Workspace {
    /// The workspace root directory.
    pub root_dir: PathBuf,
    /// Every package: workspace members and their dependencies.
    pub packages: Vec<Package>,
    /// Ids of the packages that are workspace members.
    pub member_ids: Vec<String>,
    /// The resolved dependency graph.
    pub graph: ResolvedGraph,
    /// Tidy check settings.
    pub tidy: TidyConfig,
    /// DCO check settings.
    pub dco: DcoConfig,
}

impl Workspace {
    /// Workspace-member packages, excluding external dependencies.
    pub(crate) fn members(&self) -> impl Iterator<Item = &Package> {
        self.packages
            .iter()
            .filter(|p| self.member_ids.contains(&p.id))
    }

    /// True if `name` is a workspace member's crate name.
    pub(crate) fn is_member(&self, name: &str) -> bool {
        self.members().any(|p| p.name == name)
    }

    /// The workspace member named exactly `name`, if any.
    pub(crate) fn find_member(&self, name: &str) -> Option<&Package> {
        self.members().find(|p| p.name == name)
    }

    /// Any package, member or external, with this `cargo metadata` id.
    pub(crate) fn package_by_id(&self, id: &str) -> Option<&Package> {
        self.packages.iter().find(|p| p.id == id)
    }

    /// True if `package`'s manifest lies under the configured app directory.
    pub(crate) fn is_app(&self, package: &Package) -> bool {
        self.is_under(package, &self.tidy.app_dir)
    }

    /// True if `package`'s manifest lies under the configured tool directory.
    pub(crate) fn is_tool(&self, package: &Package) -> bool {
        self.is_under(package, &self.tidy.tool_dir)
    }

    /// True if `package`'s manifest lies under `dir`, a folder relative to the workspace root.
    fn is_under(&self, package: &Package, dir: &str) -> bool {
        package.manifest_path.starts_with(self.root_dir.join(dir))
    }
}

/// Why loading a workspace from `cargo metadata` JSON failed.
#[derive(Debug)]
pub(crate) enum LoadError {
    /// The text isn't valid JSON.
    Json(serde_json::Error),
    /// The JSON has no `resolve` section.
    MissingResolve,
    /// `workspace.metadata` has no section with this name.
    MissingSection(&'static str),
    /// A `workspace.metadata` value is invalid.
    BadConfig(String),
    /// Running `cargo metadata` itself failed.
    Cargo(String),
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(err) => write!(f, "invalid cargo metadata JSON: {err}"),
            Self::MissingResolve => write!(f, "cargo metadata has no resolve section"),
            Self::MissingSection(name) => write!(f, "workspace.metadata.{name} is missing"),
            Self::BadConfig(reason) => write!(f, "invalid workspace.metadata config: {reason}"),
            Self::Cargo(reason) => write!(f, "cargo metadata failed: {reason}"),
        }
    }
}

/// Loads a [`Workspace`] from `cargo metadata --format-version 1` JSON.
pub(crate) fn load(json: &str) -> Result<Workspace, LoadError> {
    let output: raw::RawOutput = serde_json::from_str(json).map_err(LoadError::Json)?;
    let resolve = output.resolve.ok_or(LoadError::MissingResolve)?;
    let mut meta = output.metadata.unwrap_or_default();
    let tidy = load_section::<TidyConfig>(meta.remove("tidy"), "tidy")?;
    tidy.validate().map_err(config_error)?;
    let dco = load_section::<DcoConfig>(meta.remove("dco"), "dco")?;

    Ok(Workspace {
        root_dir: PathBuf::from(output.workspace_root),
        packages: output.packages.into_iter().map(into_package).collect(),
        member_ids: output.workspace_members,
        graph: build_graph(&resolve),
        tidy,
        dco,
    })
}

/// The workspace root and one deserialized `workspace.metadata` section.
pub(crate) fn root_and_section<T: serde::de::DeserializeOwned>(
    json: &str,
    name: &'static str,
) -> Result<(PathBuf, T), LoadError> {
    let raw: raw::RawSections = serde_json::from_str(json).map_err(LoadError::Json)?;
    let value = raw.metadata.and_then(|mut m| m.remove(name));
    Ok((
        PathBuf::from(raw.workspace_root),
        load_section(value, name)?,
    ))
}

/// Deserializes one `workspace.metadata` section, naming it if missing.
fn load_section<T: serde::de::DeserializeOwned>(
    value: Option<serde_json::Value>,
    name: &'static str,
) -> Result<T, LoadError> {
    let value = value.ok_or(LoadError::MissingSection(name))?;
    serde_json::from_value(value).map_err(|err| LoadError::BadConfig(err.to_string()))
}

/// Wraps a config validation failure as a [`LoadError`].
pub(crate) fn config_error(err: ConfigError) -> LoadError {
    LoadError::BadConfig(err.to_string())
}

/// Converts one raw package entry into the domain model.
fn into_package(raw: raw::RawPackage) -> Package {
    Package {
        id: raw.id,
        name: raw.name,
        manifest_path: PathBuf::from(raw.manifest_path),
        dependencies: raw
            .dependencies
            .into_iter()
            .map(|d| Dependency {
                name: d.name,
                kind: DependencyKind::from_raw(d.kind.as_deref()),
            })
            .collect(),
        targets: raw
            .targets
            .into_iter()
            .map(|t| Target {
                kinds: t.kind,
                src_path: PathBuf::from(t.src_path),
            })
            .collect(),
    }
}

/// Builds the resolved dependency graph from the `resolve` section.
fn build_graph(resolve: &raw::RawResolve) -> ResolvedGraph {
    resolve
        .nodes
        .iter()
        .map(|node| {
            (
                node.id.clone(),
                node.deps.iter().flat_map(resolved_edges).collect(),
            )
        })
        .collect()
}

/// Expands one resolved dependency into one edge per kind it's used as.
fn resolved_edges(dep: &raw::RawNodeDep) -> Vec<ResolvedDependency> {
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

/// Runs `cargo metadata` and loads its output into a [`Workspace`].
///
/// Kept tiny and not unit-tested: it spawns a process.
pub(crate) fn from_cargo() -> Result<Workspace, LoadError> {
    load(&metadata_json()?)
}

/// Runs `cargo metadata` and returns its JSON text.
///
/// Kept tiny and not unit-tested: it spawns a process.
pub(crate) fn metadata_json() -> Result<String, LoadError> {
    let output = crate::cargo::command()
        .args(METADATA_ARGS)
        .output()
        .map_err(|err| LoadError::Cargo(err.to_string()))?;
    if !output.status.success() {
        return Err(LoadError::Cargo(
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

#[cfg(test)]
mod tests;
