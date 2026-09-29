//! Properties of the dependency order over random graphs, checked against a separate model.

use super::*;
use Reason::{Cycle, DependsOnFailed, DuplicateProvider, Missing};
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::sample::subsequence;

const MODULES: [&str; 8] = ["m0", "m1", "m2", "m3", "m4", "m5", "m6", "m7"];
const SERVICES: [&str; 8] = ["s0", "s1", "s2", "s3", "s4", "s5", "s6", "s7"];
/// The one platform service of acyclic graphs.
const PLATFORM: &str = "platform";
/// Most services a random module requires.
const MAX_REQUIRES: usize = 3;
/// Most services a random module provides.
const MAX_PROVIDES: usize = 2;
/// Most services the random platform provides.
const MAX_PLATFORM: usize = 2;

/// A random input: per module, in insertion order, what it requires and provides.
#[derive(Clone, Debug)]
struct Graph {
    requires: Vec<Vec<ServiceId>>,
    provides: Vec<Vec<ServiceId>>,
    platform: BTreeSet<ServiceId>,
}

impl Graph {
    fn plan(&self) -> Plan {
        let nodes: Vec<Node<'_>> = (self.requires.iter().zip(&self.provides))
            .enumerate()
            .map(|(i, (requires, provides))| Node {
                id: id(i),
                requires,
                provides,
            })
            .collect();
        plan(&nodes, &self.platform)
    }

    /// The first module providing `s`, ignoring the platform.
    fn first(&self, s: ServiceId) -> Option<usize> {
        self.provides.iter().position(|p| p.contains(&s))
    }

    /// For each module, the modules it waits for; platform services need none.
    fn edges(&self) -> Vec<Vec<usize>> {
        let wait = |s: &ServiceId| (!self.platform.contains(s)).then(|| self.first(*s))?;
        self.requires
            .iter()
            .map(|r| r.iter().filter_map(wait).collect())
            .collect()
    }
}

fn id(i: usize) -> ModuleId {
    ModuleId::new(MODULES[i])
}

fn index(id: ModuleId) -> usize {
    MODULES.iter().position(|m| *m == id.as_str()).unwrap()
}

fn service(name: &&'static str) -> ServiceId {
    ServiceId::new(name)
}

/// `reach[a][b]` when `a` transitively waits for `b` (Floyd-Warshall).
fn reach(edges: &[Vec<usize>]) -> Vec<Vec<bool>> {
    let n = edges.len();
    let mut r = vec![vec![false; n]; n];
    for (a, to) in edges.iter().enumerate() {
        for &b in to {
            r[a][b] = true;
        }
    }
    for k in 0..n {
        for a in 0..n {
            for b in 0..n {
                r[a][b] = r[a][b] || (r[a][k] && r[k][b]);
            }
        }
    }
    r
}

/// Graphs without cycles, duplicates or missing services, inserted in a shuffled order.
fn acyclic() -> impl Strategy<Value = Graph> {
    (1..=MODULES.len())
        .prop_flat_map(|n| {
            let rank = Just((0..n).collect::<Vec<_>>()).prop_shuffle();
            (rank, vec(any::<bool>(), n * n), vec(any::<bool>(), n))
        })
        .prop_map(|(rank, wants, on_platform)| {
            let n = rank.len();
            let platform = ServiceId::new(PLATFORM);
            let requires = (0..n)
                .map(|a| {
                    let below = (0..n).filter(|&b| rank[b] < rank[a] && wants[a * n + b]);
                    let mut r: Vec<ServiceId> = below.map(|b| service(&SERVICES[b])).collect();
                    r.extend(on_platform[a].then_some(platform));
                    r
                })
                .collect();
            let provides = (0..n).map(|a| vec![service(&SERVICES[a])]).collect();
            let platform = BTreeSet::from([platform]);
            Graph {
                requires,
                provides,
                platform,
            }
        })
}

/// Any graph: cycles, self-cycles, duplicates, missing and platform services all occur.
fn any_graph() -> impl Strategy<Value = Graph> {
    let pick = |most: usize| subsequence(SERVICES.to_vec(), 0..=most);
    let ids = |names: Vec<&'static str>| names.iter().map(service).collect::<Vec<_>>();
    vec(pick(MAX_REQUIRES), 1..=MODULES.len())
        .prop_flat_map(move |requires| {
            let n = requires.len();
            (
                Just(requires),
                vec(pick(MAX_PROVIDES), n),
                pick(MAX_PLATFORM),
            )
        })
        .prop_map(move |(requires, provides, platform)| Graph {
            requires: requires.into_iter().map(ids).collect(),
            provides: provides.into_iter().map(ids).collect(),
            platform: ids(platform).into_iter().collect(),
        })
}

/// True when the model agrees that `m` cannot start for `reason`.
fn justified(g: &Graph, reach: &[Vec<bool>], failed: &[usize], m: usize, reason: Reason) -> bool {
    match reason {
        Cycle => reach[m][m],
        Missing(s) => {
            g.requires[m].contains(&s) && !g.platform.contains(&s) && g.first(s).is_none()
        }
        DuplicateProvider(s, first) => {
            g.provides[m].contains(&s) && g.first(s) == Some(index(first)) && index(first) != m
        }
        DependsOnFailed(p) => g.edges()[m].contains(&index(p)) && failed.contains(&index(p)),
    }
}

/// Each failure is justified, and a module is marked `Cycle` exactly when it is on a cycle.
fn check_failures(g: &Graph, got: &Plan, reach: &[Vec<bool>]) -> Result<(), TestCaseError> {
    let failed: Vec<usize> = got.failed().iter().map(|&(m, _)| index(m)).collect();
    for &(m, reason) in got.failed() {
        prop_assert!(
            justified(g, reach, &failed, index(m), reason),
            "{m}: {reason:?}"
        );
    }
    for (m, row) in reach.iter().enumerate() {
        let cycle = got.failed().contains(&(id(m), Cycle));
        prop_assert_eq!(cycle, row[m], "module {} cycle mark", m);
    }
    Ok(())
}

/// Every module is started or failed once, and each started module follows its providers.
fn check_started(g: &Graph, got: &Plan) -> Result<(), TestCaseError> {
    let mut seen: Vec<usize> = got.start().iter().copied().map(index).collect();
    seen.extend(got.failed().iter().map(|&(m, _)| index(m)));
    seen.sort_unstable();
    prop_assert_eq!(seen, (0..g.requires.len()).collect::<Vec<_>>());
    let place = |m: usize| got.start().iter().position(|&s| s == id(m));
    for (m, providers) in g.edges().iter().enumerate() {
        let Some(at) = place(m) else { continue };
        for &p in providers {
            prop_assert!(
                place(p).is_some_and(|before| before < at),
                "{} before {}",
                p,
                m
            );
        }
    }
    Ok(())
}

/// Stop order reverses start order, and `dependents` lists exactly what waits on a module.
fn check_stop(got: &Plan, reach: &[Vec<bool>]) -> Result<(), TestCaseError> {
    let stop = stop_order(got);
    let mut reversed = got.start().to_vec();
    reversed.reverse();
    prop_assert_eq!(&stop, &reversed);
    for &m in got.start() {
        let want: Vec<ModuleId> = (stop.iter().copied())
            .filter(|&d| reach[index(d)][index(m)])
            .collect();
        prop_assert_eq!(dependents(got, m), want, "dependents of {}", m);
    }
    Ok(())
}

proptest! {
    #[test]
    fn acyclic_graphs_start_all_with_providers_first(g in acyclic()) {
        let got = g.plan();
        prop_assert!(got.failed().is_empty(), "{:?}", got.failed());
        check_started(&g, &got)?;
    }

    #[test]
    fn any_graph_fails_only_what_it_must(g in any_graph()) {
        let got = g.plan();
        let reach = reach(&g.edges());
        check_started(&g, &got)?;
        check_failures(&g, &got, &reach)?;
        check_stop(&got, &reach)?;
        prop_assert_eq!(g.plan(), got);
    }
}
