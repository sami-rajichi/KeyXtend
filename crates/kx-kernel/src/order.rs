//! The dependency order: which modules start, in what order, and which cannot start and why.

use kx_module_api::{ModuleId, ServiceId};
use std::collections::{BTreeMap, BTreeSet};

/// A module as the order sees it: its name and the services it requires and provides.
#[derive(Copy, Clone, Debug)]
pub struct Node<'a> {
    /// The module's name.
    pub id: ModuleId,
    /// Services it consumes.
    pub requires: &'a [ServiceId],
    /// Services it registers.
    pub provides: &'a [ServiceId],
}

/// Why a module cannot start.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Reason {
    /// It requires a service that neither a module nor the platform provides.
    Missing(ServiceId),
    /// It provides a service already provided by this earlier module, which keeps it.
    DuplicateProvider(ServiceId, ModuleId),
    /// It is on a dependency cycle, which includes requiring its own service.
    Cycle,
    /// A module whose service it requires cannot start.
    DependsOnFailed(ModuleId),
}

/// The start order, providers first, and the modules that cannot start.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plan {
    start: Vec<ModuleId>,
    failed: Vec<(ModuleId, Reason)>,
    /// The providers each started module waits for, in step with `start`.
    uses: Vec<Vec<ModuleId>>,
}

impl Plan {
    /// Modules to start, in order: each one after the providers of what it requires.
    #[must_use]
    pub fn start(&self) -> &[ModuleId] {
        &self.start
    }

    /// Modules that cannot start, in insertion order, each with its reason.
    #[must_use]
    pub fn failed(&self) -> &[(ModuleId, Reason)] {
        &self.failed
    }
}

/// Orders `nodes`, given in insertion order, knowing the services the `platform` provides.
///
/// A module with several problems gets the first of `Cycle`, `DuplicateProvider` and `Missing`.
/// Requiring a platform service never waits for a module, even one that also provides it.
#[must_use]
pub fn plan(nodes: &[Node<'_>], platform: &BTreeSet<ServiceId>) -> Plan {
    let owners = owners(nodes);
    let edges: Vec<Vec<usize>> = nodes
        .iter()
        .map(|n| providers(n, &owners, platform))
        .collect();
    let mut reasons: Vec<Option<Reason>> = (0..nodes.len())
        .map(|m| problem(nodes, &owners, &edges, platform, m))
        .collect();
    let start = kahn(nodes, &edges, &mut reasons);
    Plan {
        start: start.iter().map(|&m| nodes[m].id).collect(),
        uses: start
            .iter()
            .map(|&m| edges[m].iter().map(|&p| nodes[p].id).collect())
            .collect(),
        failed: (nodes.iter().zip(reasons))
            .filter_map(|(n, r)| Some((n.id, r?)))
            .collect(),
    }
}

/// The order to stop in: the start order reversed, so dependents stop first.
#[must_use]
pub fn stop_order(plan: &Plan) -> Vec<ModuleId> {
    plan.start.iter().rev().copied().collect()
}

/// The started modules that transitively require a service of `id`, in stop order.
#[must_use]
pub fn dependents(plan: &Plan, id: ModuleId) -> Vec<ModuleId> {
    let mut hit = BTreeSet::from([id]);
    let mut found = Vec::new();
    for (&m, uses) in plan.start.iter().zip(&plan.uses) {
        if uses.iter().any(|p| hit.contains(p)) {
            hit.insert(m);
            found.push(m);
        }
    }
    found.reverse();
    found
}

/// The first module, in insertion order, providing each service.
fn owners(nodes: &[Node<'_>]) -> BTreeMap<ServiceId, usize> {
    let mut owners = BTreeMap::new();
    for (m, node) in nodes.iter().enumerate() {
        for &s in node.provides {
            owners.entry(s).or_insert(m);
        }
    }
    owners
}

/// The modules `node` waits for, once each, in the order of its `requires`.
fn providers(
    node: &Node<'_>,
    owners: &BTreeMap<ServiceId, usize>,
    platform: &BTreeSet<ServiceId>,
) -> Vec<usize> {
    let mut found = Vec::new();
    for s in node.requires.iter().filter(|s| !platform.contains(s)) {
        if let Some(&p) = owners.get(s).filter(|p| !found.contains(*p)) {
            found.push(p);
        }
    }
    found
}

/// Module `m`'s own reason not to start, before looking at what it depends on.
fn problem(
    nodes: &[Node<'_>],
    owners: &BTreeMap<ServiceId, usize>,
    edges: &[Vec<usize>],
    platform: &BTreeSet<ServiceId>,
    m: usize,
) -> Option<Reason> {
    if on_cycle(edges, m) {
        return Some(Reason::Cycle);
    }
    let first = |s: &ServiceId| owners.get(s).copied().filter(|&p| p != m);
    if let Some((&s, p)) = nodes[m].provides.iter().find_map(|s| Some((s, first(s)?))) {
        return Some(Reason::DuplicateProvider(s, nodes[p].id));
    }
    let missing = |s: &&ServiceId| !platform.contains(s) && !owners.contains_key(s);
    nodes[m]
        .requires
        .iter()
        .find(missing)
        .map(|&s| Reason::Missing(s))
}

/// True when module `m` transitively waits for itself: it sits in a strongly connected
/// component that holds a cycle.
fn on_cycle(edges: &[Vec<usize>], m: usize) -> bool {
    let mut seen = vec![false; edges.len()];
    let mut todo = edges[m].clone();
    while let Some(v) = todo.pop() {
        if v == m {
            return true;
        }
        if !std::mem::replace(&mut seen[v], true) {
            todo.extend(&edges[v]);
        }
    }
    false
}

/// Kahn's algorithm over the modules off any cycle, taking the earliest inserted ready module
/// first; it fails a module whose provider failed and returns the start order.
fn kahn(nodes: &[Node<'_>], edges: &[Vec<usize>], reasons: &mut [Option<Reason>]) -> Vec<usize> {
    let off_cycle = |m: usize| reasons[m] != Some(Reason::Cycle);
    let mut waiting = vec![0_usize; nodes.len()];
    let mut users = vec![Vec::new(); nodes.len()];
    for (m, to) in edges.iter().enumerate().filter(|&(m, _)| off_cycle(m)) {
        for &p in to.iter().filter(|&&p| off_cycle(p)) {
            waiting[m] += 1;
            users[p].push(m);
        }
    }
    let mut ready: BTreeSet<usize> = (0..nodes.len())
        .filter(|&m| off_cycle(m) && waiting[m] == 0)
        .collect();
    let mut start = Vec::new();
    while let Some(m) = ready.pop_first() {
        if reasons[m].is_none() {
            let failed = edges[m].iter().find(|&&p| reasons[p].is_some());
            reasons[m] = failed.map(|&p| Reason::DependsOnFailed(nodes[p].id));
        }
        if reasons[m].is_none() {
            start.push(m);
        }
        for &u in &users[m] {
            waiting[u] -= 1;
            if waiting[u] == 0 {
                ready.insert(u);
            }
        }
    }
    start
}

#[cfg(test)]
mod props;
#[cfg(test)]
mod tests;
