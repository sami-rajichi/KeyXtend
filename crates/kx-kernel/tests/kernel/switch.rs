//! Switched off wins: a module that is off, or waits for one that is off, stops without a failure notice.

use crate::record::{Mode, Switch};
use crate::rig::{Rig, failed};
use crate::samples::{Alpha, BROKEN_SPEC, Beta, Delta, Gamma, sample};
use kx_kernel::KernelError;
use kx_module_api::ModuleState::{Active, Failed, Stopped};
use kx_module_api::{ModuleId, ServiceKey};

const A: ModuleId = ModuleId::new("a");
const B: ModuleId = ModuleId::new("b");
const C: ModuleId = ModuleId::new("c");
const D: ModuleId = ModuleId::new("d");
const E: ModuleId = ModuleId::new("e");
const F: ModuleId = ModuleId::new("f");

/// Boots with `off` switched off in the file, and checks no notice came and each of `stopped` is `Stopped`.
fn boots_quietly(rig: &mut Rig, off: &[ModuleId], stopped: &[ModuleId]) {
    rig.off_in_file(off);
    let notices = rig.kernel.boot().ok();
    assert_eq!(notices, Some(Vec::new()), "switched off is no failure");
    for &id in stopped {
        assert_eq!(rig.state(id), Some(Stopped), "{id}");
    }
}

#[test]
fn a_switched_off_module_missing_a_service_stops_quietly_and_stays_blocked() {
    let mut rig = Rig::new();
    rig.add(sample(A).requires(&[Gamma::ID]));
    boots_quietly(&mut rig, &[A], &[A]);
    rig.kernel.set_enabled(A, true).unwrap();
    assert_eq!(rig.state(A), Some(Failed));
    assert_eq!(
        rig.notices(),
        [failed(A)],
        "switched on, it is still blocked"
    );
    let again = rig.kernel.retry(A);
    assert!(matches!(again, Err(KernelError::NotStartable(A))));
}

#[test]
fn switched_off_modules_on_a_cycle_stop_quietly() {
    let mut rig = Rig::new();
    rig.add(sample(A).requires(&[Beta::ID]).provides(&[Alpha::ID]));
    rig.add(sample(B).requires(&[Alpha::ID]).provides(&[Beta::ID]));
    boots_quietly(&mut rig, &[A, B], &[A, B]);
}

#[test]
fn a_switched_off_second_provider_stops_quietly() {
    let mut rig = Rig::new();
    rig.add(sample(C).provides(&[Alpha::ID]));
    rig.add(sample(A).provides(&[Alpha::ID]));
    boots_quietly(&mut rig, &[A], &[A]);
    assert_eq!(rig.state(C), Some(Active));
}

#[test]
fn a_switched_off_module_with_a_broken_spec_stops_quietly() {
    let mut rig = Rig::new();
    rig.add(sample(A).settings(&BROKEN_SPEC));
    rig.add(sample(B));
    boots_quietly(&mut rig, &[A], &[A]);
    assert_eq!(rig.log.starts(), [B]);
}

#[test]
fn only_modules_blocked_by_switched_off_ones_alone_stop_quietly() {
    let mut rig = Rig::new();
    // Dependents come first, so the order of adding cannot help.
    rig.add(sample(C).requires(&[Beta::ID]));
    rig.add(sample(B).requires(&[Alpha::ID]).provides(&[Beta::ID]));
    rig.add(sample(A).requires(&[Gamma::ID]).provides(&[Alpha::ID]));
    // E is really broken, so D and F, which need it, fail aloud.
    rig.add(sample(E).requires(&[Gamma::ID]).provides(&[Delta::ID]));
    rig.add(sample(D).requires(&[Delta::ID]));
    rig.add(sample(F).requires(&[Alpha::ID, Delta::ID]));
    rig.off_in_file(&[A]);
    let notices = rig.kernel.boot().unwrap();
    assert_eq!(notices, [failed(E), failed(D), failed(F)]);
    let states = [A, B, C, D, E, F].map(|m| rig.state(m));
    let quiet = Some(Stopped);
    let loud = Some(Failed);
    assert_eq!(states, [quiet, quiet, quiet, loud, loud, loud]);
}

#[test]
fn switching_off_a_failed_module_stops_it_without_a_notice() {
    let mut rig = Rig::new();
    rig.add(sample(A).mode(Mode::Fail));
    assert_eq!(rig.kernel.boot().unwrap(), [failed(A)]);
    rig.kernel.set_enabled(A, false).unwrap();
    assert_eq!(rig.state(A), Some(Stopped));
    assert_eq!(rig.notices(), [failed(A)], "only the boot failure");
}

#[test]
fn switching_on_a_module_whose_provider_is_off_waits_quietly() {
    let mut rig = Rig::new();
    rig.add(sample(A).provides(&[Alpha::ID]));
    rig.add(sample(B).requires(&[Alpha::ID]).provides(&[Beta::ID]));
    rig.add(sample(C).requires(&[Beta::ID]));
    rig.kernel.boot().unwrap();
    rig.kernel.set_enabled(A, false).unwrap();
    rig.kernel.set_enabled(B, true).unwrap();
    rig.kernel.set_enabled(C, true).unwrap();
    let states = [A, B, C].map(|m| rig.state(m));
    assert_eq!(
        states,
        [Some(Stopped); 3],
        "C waits behind B, which waits for A"
    );
    assert_eq!(rig.notices(), [], "waiting is no failure");
    rig.kernel.set_enabled(A, true).unwrap();
    assert!([A, B, C].iter().all(|&m| rig.state(m) == Some(Active)));
}

#[test]
fn try_again_refuses_quietly_while_a_provider_is_off() {
    let (mut rig, b) = (Rig::new(), Switch::new(Mode::Fail));
    rig.add(sample(A).provides(&[Alpha::ID]));
    rig.add(sample(B).requires(&[Alpha::ID]).switch(&b));
    rig.kernel.boot().unwrap();
    rig.kernel.set_enabled(A, false).unwrap();
    let again = rig.kernel.retry(B);
    assert!(matches!(again, Err(KernelError::NotStartable(B))));
    assert_eq!(rig.notices(), [failed(B)], "only the boot failure");
    rig.kernel.set_enabled(A, true).unwrap();
    b.set(Mode::Succeed);
    rig.kernel.retry(B).unwrap();
    assert_eq!([A, B].map(|m| rig.state(m)), [Some(Active); 2]);
}
