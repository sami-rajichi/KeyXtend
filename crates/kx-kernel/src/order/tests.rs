//! Named cases of the dependency order.

use super::*;
use Reason::{Cycle, DependsOnFailed, DuplicateProvider, Missing};

const A: ModuleId = ModuleId::new("a");
const B: ModuleId = ModuleId::new("b");
const C: ModuleId = ModuleId::new("c");
const D: ModuleId = ModuleId::new("d");
const E: ModuleId = ModuleId::new("e");

const SA: ServiceId = ServiceId::new("sa");
const SB: ServiceId = ServiceId::new("sb");
const SX: ServiceId = ServiceId::new("sx");
const PLATFORM: ServiceId = ServiceId::new("platform");

fn node(
    id: ModuleId,
    requires: &'static [ServiceId],
    provides: &'static [ServiceId],
) -> Node<'static> {
    Node {
        id,
        requires,
        provides,
    }
}

fn no_platform() -> BTreeSet<ServiceId> {
    BTreeSet::new()
}

/// A provides `sa`, B needs it and provides `sb`, C needs `sb`, D needs `sa`, E stands alone.
fn tree() -> Plan {
    let nodes = [
        node(A, &[], &[SA]),
        node(B, &[SA], &[SB]),
        node(C, &[SB], &[]),
        node(D, &[SA], &[]),
        node(E, &[], &[]),
    ];
    plan(&nodes, &no_platform())
}

#[test]
fn a_chain_starts_providers_first() {
    let nodes = [
        node(C, &[SB], &[]),
        node(B, &[SA], &[SB]),
        node(A, &[], &[SA]),
    ];
    let got = plan(&nodes, &no_platform());
    assert_eq!(got.start(), [A, B, C]);
    assert!(got.failed().is_empty());
}

#[test]
fn a_missing_service_fails_the_module_and_its_dependents() {
    let nodes = [
        node(A, &[SX], &[SA]),
        node(B, &[SA], &[SB]),
        node(C, &[SB], &[]),
        node(D, &[], &[]),
    ];
    let got = plan(&nodes, &no_platform());
    assert_eq!(got.start(), [D]);
    let want = [
        (A, Missing(SX)),
        (B, DependsOnFailed(A)),
        (C, DependsOnFailed(B)),
    ];
    assert_eq!(got.failed(), want);
}

#[test]
fn a_second_provider_fails_and_the_first_is_kept() {
    let nodes = [
        node(A, &[], &[SA]),
        node(B, &[], &[SA]),
        node(C, &[SA], &[]),
    ];
    let got = plan(&nodes, &no_platform());
    assert_eq!(got.start(), [A, C]);
    assert_eq!(got.failed(), [(B, DuplicateProvider(SA, A))]);
    assert_eq!(dependents(&got, A), [C]);
}

#[test]
fn a_two_module_cycle_fails_both_and_not_its_dependent() {
    let nodes = [
        node(A, &[SB], &[SA]),
        node(B, &[SA], &[SB]),
        node(C, &[SA], &[]),
        node(D, &[], &[]),
    ];
    let got = plan(&nodes, &no_platform());
    assert_eq!(got.start(), [D]);
    let want = [(A, Cycle), (B, Cycle), (C, DependsOnFailed(A))];
    assert_eq!(got.failed(), want);
}

#[test]
fn a_self_cycle_fails() {
    let nodes = [node(A, &[SA], &[SA]), node(B, &[], &[])];
    let got = plan(&nodes, &no_platform());
    assert_eq!(got.start(), [B]);
    assert_eq!(got.failed(), [(A, Cycle)]);
}

#[test]
fn platform_services_satisfy_requires() {
    let nodes = [node(A, &[PLATFORM], &[SA]), node(B, &[SA, PLATFORM], &[])];
    let got = plan(&nodes, &BTreeSet::from([PLATFORM]));
    assert_eq!(got.start(), [A, B]);
    assert!(got.failed().is_empty());
}

#[test]
fn a_platform_service_never_waits_for_a_module() {
    let nodes = [node(A, &[PLATFORM], &[]), node(B, &[], &[PLATFORM])];
    let got = plan(&nodes, &BTreeSet::from([PLATFORM]));
    assert_eq!(got.start(), [A, B]);
    assert!(dependents(&got, B).is_empty());
}

#[test]
fn ties_keep_insertion_order() {
    let free = [node(C, &[], &[]), node(A, &[], &[]), node(B, &[], &[])];
    assert_eq!(plan(&free, &no_platform()).start(), [C, A, B]);

    let mixed = [
        node(D, &[], &[]),
        node(B, &[SA], &[]),
        node(A, &[], &[SA]),
        node(E, &[], &[]),
    ];
    assert_eq!(plan(&mixed, &no_platform()).start(), [D, A, B, E]);
}

#[test]
fn stop_order_is_the_start_order_reversed() {
    let got = tree();
    assert_eq!(got.start(), [A, B, C, D, E]);
    assert_eq!(stop_order(&got), [E, D, C, B, A]);
}

#[test]
fn dependents_lists_transitive_dependents_first() {
    let got = tree();
    assert_eq!(dependents(&got, A), [D, C, B]);
    assert_eq!(dependents(&got, B), [C]);
    assert!(dependents(&got, E).is_empty());
}

#[test]
fn a_failed_module_has_no_started_dependents() {
    let nodes = [node(A, &[SX], &[SA]), node(B, &[SA], &[])];
    let got = plan(&nodes, &no_platform());
    assert!(dependents(&got, A).is_empty());
    assert!(stop_order(&got).is_empty());
}
