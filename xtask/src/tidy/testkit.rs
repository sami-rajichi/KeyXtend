//! Builds small [`Workspace`] values in code, for the tidy rule tests (plan edge cases 1-12).

use std::collections::HashMap;
use std::path::PathBuf;

use crate::config::TidyConfig;
use crate::test_support::{dco_config, tidy_config, unique_temp_dir};
use crate::workspace::{
    Dependency, DependencyKind, Package, ResolvedDependency, ResolvedGraph, Target, Workspace,
};

/// Shorthand for building one [`Dependency`].
pub(crate) fn dep(name: &str, kind: DependencyKind) -> Dependency {
    Dependency {
        name: name.to_string(),
        kind,
    }
}

/// Builds a [`Workspace`] from packages and resolved edges added one at a time.
pub(crate) struct WorkspaceBuilder {
    root_dir: PathBuf,
    packages: Vec<Package>,
    member_ids: Vec<String>,
    graph: ResolvedGraph,
    tidy: TidyConfig,
}

impl WorkspaceBuilder {
    /// Starts a builder with the trimmed [`tidy_config`] and a root folder that never exists.
    pub(crate) fn new() -> Self {
        Self {
            root_dir: unique_temp_dir("no-root"),
            packages: Vec::new(),
            member_ids: Vec::new(),
            graph: HashMap::new(),
            tidy: tidy_config(),
        }
    }

    /// Overrides the workspace root, e.g. to a real temp directory for file-reading tests.
    /// Call before `member`/`target` so their paths resolve under the new root.
    pub(crate) fn root(mut self, dir: PathBuf) -> Self {
        self.root_dir = dir;
        self
    }

    /// Adds a workspace member under `<root>/<sub_dir>/<name>`, with these dependencies.
    pub(crate) fn member(mut self, name: &str, sub_dir: &str, deps: Vec<Dependency>) -> Self {
        let id = name.to_string();
        self.packages.push(Package {
            id: id.clone(),
            name: name.to_string(),
            manifest_path: self.root_dir.join(sub_dir).join(name).join("Cargo.toml"),
            dependencies: deps,
            targets: Vec::new(),
        });
        self.member_ids.push(id);
        self
    }

    /// Adds an external (non-member) package, so the graph walk can resolve its id.
    pub(crate) fn external(mut self, name: &str) -> Self {
        self.packages.push(Package {
            id: name.to_string(),
            name: name.to_string(),
            manifest_path: PathBuf::from("/external").join(name),
            dependencies: Vec::new(),
            targets: Vec::new(),
        });
        self
    }

    /// Adds a build target to the most recently added package, at `<root>/<rel_path>`.
    ///
    /// No-op if no package was added yet: a test bug, not something to panic over.
    pub(crate) fn target(mut self, kinds: &[&str], rel_path: &str) -> Self {
        let src_path = self.root_dir.join(rel_path);
        if let Some(package) = self.packages.last_mut() {
            package.targets.push(Target {
                kinds: kinds.iter().map(|k| (*k).to_string()).collect(),
                src_path,
            });
        }
        self
    }

    /// Adds one resolved edge from `from` to `to`, used as `kind`.
    pub(crate) fn edge(mut self, from: &str, to: &str, kind: DependencyKind) -> Self {
        self.graph
            .entry(from.to_string())
            .or_default()
            .push(ResolvedDependency {
                id: to.to_string(),
                kind,
            });
        self
    }

    /// Builds the [`Workspace`].
    pub(crate) fn build(self) -> Workspace {
        Workspace {
            root_dir: self.root_dir,
            packages: self.packages,
            member_ids: self.member_ids,
            graph: self.graph,
            tidy: self.tidy,
            dco: dco_config(),
        }
    }
}
