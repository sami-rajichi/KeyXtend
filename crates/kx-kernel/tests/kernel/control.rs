//! Try again, stop order and switching modules off and on, saved in `[kernel] disabled`.

use crate::record::{Did, Mode, Switch};
use crate::rig::{Rig, failed};
use crate::samples::{Alpha, Beta, Gamma, sample};
use kx_kernel::{KERNEL, KernelError, Phase};
use kx_module_api::ModuleState::{Active, Failed, Starting, Stopped};
use kx_module_api::{ModuleId, ServiceKey};
use std::fs;

const A: ModuleId = ModuleId::new("a");
const B: ModuleId = ModuleId::new("b");
const C: ModuleId = ModuleId::new("c");
const D: ModuleId = ModuleId::new("d");
const NOBODY: ModuleId = ModuleId::new("nobody");

/// A chain C needs B needs A, added dependents first, and D on its own.
fn chain(rig: &mut Rig, a: &Switch) {
    chain_of(rig, a, &Switch::new(Mode::Succeed));
}

/// The chain, with B steered by `b` too.
fn chain_of(rig: &mut Rig, a: &Switch, b: &Switch) {
    rig.add(sample(C).requires(&[Beta::ID]));
    let b_shape = sample(B).requires(&[Alpha::ID]).provides(&[Beta::ID]);
    rig.add(b_shape.switch(b));
    rig.add(sample(A).provides(&[Alpha::ID]).switch(a));
    rig.add(sample(D));
}

/// True when every one of `ids` is `Active`.
fn all_active(rig: &Rig, ids: &[ModuleId]) -> bool {
    ids.iter().all(|&m| rig.state(m) == Some(Active))
}

#[test]
fn try_again_restarts_what_a_failed_switch_on_left_stopped() {
    let (mut rig, a) = (Rig::new(), Switch::new(Mode::Succeed));
    chain(&mut rig, &a);
    rig.kernel.boot().unwrap();
    rig.kernel.set_enabled(A, false).unwrap();
    a.set(Mode::Fail);
    rig.kernel.set_enabled(A, true).unwrap();
    let states = [A, B, C].map(|m| rig.state(m));
    assert_eq!(states, [Some(Failed), Some(Stopped), Some(Stopped)]);
    a.set(Mode::Succeed);
    rig.kernel.retry(A).unwrap();
    assert!(all_active(&rig, &[A, B, C, D]), "{:?}", rig.moves());
}

#[test]
fn try_again_on_a_dependent_restarts_what_waited_behind_it() {
    let (mut rig, b) = (Rig::new(), Switch::new(Mode::Succeed));
    chain_of(&mut rig, &Switch::new(Mode::Succeed), &b);
    rig.kernel.boot().unwrap();
    rig.kernel.set_enabled(A, false).unwrap();
    b.set(Mode::Fail);
    rig.kernel.set_enabled(A, true).unwrap();
    let states = [A, B, C].map(|m| rig.state(m));
    assert_eq!(states, [Some(Active), Some(Failed), Some(Stopped)]);
    b.set(Mode::Succeed);
    rig.kernel.retry(B).unwrap();
    assert!(all_active(&rig, &[A, B, C, D]), "{:?}", rig.moves());
}

#[test]
fn try_again_never_starts_a_dependent_the_user_switched_off() {
    let (mut rig, a) = (Rig::new(), Switch::new(Mode::Fail));
    fs::create_dir_all(rig.data()).unwrap();
    let off = "[kernel]\nversion = 1\ndisabled = [\"b\"]\n";
    fs::write(rig.files().settings, off).unwrap();
    chain(&mut rig, &a);
    rig.kernel.boot().unwrap();
    a.set(Mode::Succeed);
    rig.kernel.retry(A).unwrap();
    let states = [A, B, C, D].map(|m| rig.state(m));
    let want = [Some(Active), Some(Stopped), Some(Stopped), Some(Active)];
    assert_eq!(states, want, "B stays off and C waits for it");
    assert_eq!(rig.log.starts(), [A, D, A], "B and C never ran");
}

#[test]
fn try_again_restarts_the_module_and_the_dependents_that_failed_with_it() {
    let (mut rig, a) = (Rig::new(), Switch::new(Mode::Fail));
    chain(&mut rig, &a);
    rig.kernel.boot().unwrap();
    let states = [A, B, C, D].map(|m| rig.state(m));
    assert_eq!(
        states,
        [Some(Failed), Some(Failed), Some(Failed), Some(Active)]
    );
    assert_eq!(rig.notices(), [failed(A), failed(B), failed(C)]);
    rig.log.clear();
    rig.kernel.retry(A).unwrap();
    assert_eq!(
        rig.log.starts(),
        [A],
        "dependents wait while it still fails"
    );
    assert_eq!(rig.notices().last(), Some(&failed(A)));
    a.set(Mode::Succeed);
    rig.log.clear();
    rig.kernel.retry(A).unwrap();
    assert_eq!(rig.log.starts(), [A, B, C]);
    assert!(rig.log.all().contains(&Did::Got(B, A.as_str())));
    assert!([A, B, C, D].iter().all(|&m| rig.state(m) == Some(Active)));
}

#[test]
fn try_again_refuses_unknown_running_and_blocked_modules() {
    let mut rig = Rig::new();
    rig.add(sample(A));
    rig.add(sample(B).requires(&[Gamma::ID]));
    rig.kernel.boot().unwrap();
    let retry = |rig: &mut Rig, id| rig.kernel.retry(id);
    assert!(matches!(
        retry(&mut rig, NOBODY),
        Err(KernelError::Unknown(NOBODY))
    ));
    assert!(matches!(retry(&mut rig, A), Err(KernelError::NotFailed(A))));
    assert!(matches!(
        retry(&mut rig, B),
        Err(KernelError::NotStartable(B))
    ));
}

#[test]
fn stop_all_stops_dependents_before_their_providers_and_is_safe_twice() {
    let mut rig = Rig::new();
    chain(&mut rig, &Switch::new(Mode::Succeed));
    rig.kernel.boot().unwrap();
    rig.log.clear();
    rig.kernel.stop_all();
    let want = [
        Did::Stop(D),
        Did::Stop(C),
        Did::Stop(B),
        Did::Dropped(Beta::ID),
        Did::Stop(A),
        Did::Dropped(Alpha::ID),
    ];
    assert_eq!(rig.log.all(), want);
    assert!([A, B, C, D].iter().all(|&m| rig.state(m) == Some(Stopped)));
    rig.kernel.stop_all();
    assert_eq!(rig.log.all(), want, "a second stop does nothing");
}

#[test]
fn switching_off_a_provider_stops_its_dependents_first_and_saves_it() {
    let mut rig = Rig::new();
    chain(&mut rig, &Switch::new(Mode::Succeed));
    rig.kernel.boot().unwrap();
    rig.log.clear();
    rig.kernel.set_enabled(A, false).unwrap();
    assert_eq!(rig.log.stops(), [C, B, A]);
    let states = [A, B, C, D].map(|m| rig.state(m));
    assert_eq!(
        states,
        [Some(Stopped), Some(Stopped), Some(Stopped), Some(Active)]
    );
    let text = fs::read_to_string(rig.files().settings).unwrap();
    let file: toml::Table = text.parse().unwrap();
    let off = toml::Value::Array(vec![A.as_str().into()]);
    assert_eq!(file["kernel"]["disabled"], off, "{text}");
}

#[test]
fn a_module_switched_off_in_the_file_stays_stopped_until_switched_on() {
    let mut rig = Rig::new();
    fs::create_dir_all(rig.data()).unwrap();
    let off = "[kernel]\nversion = 1\ndisabled = [\"a\"]\n";
    fs::write(rig.files().settings, off).unwrap();
    rig.add(sample(A).provides(&[Alpha::ID]));
    rig.add(sample(B).requires(&[Alpha::ID]));
    rig.add(sample(C));
    assert_eq!(
        rig.kernel.boot().unwrap(),
        [],
        "switched off is not a failure"
    );
    let moves = [(A, Stopped), (B, Stopped), (C, Starting), (C, Active)];
    assert_eq!(rig.moves(), moves, "its dependent is skipped with it");
    assert_eq!(rig.log.starts(), [C]);
    rig.kernel.set_enabled(A, true).unwrap();
    assert_eq!(rig.log.starts(), [C, A, B]);
    assert!([A, B, C].iter().all(|&m| rig.state(m) == Some(Active)));
    let text = fs::read_to_string(rig.files().settings).unwrap();
    assert_eq!(text, "", "only defaults are left, so nothing is written");
}

#[test]
fn switching_on_leaves_a_dependent_stopped_while_another_provider_is_off() {
    let mut rig = Rig::new();
    fs::create_dir_all(rig.data()).unwrap();
    let off = "[kernel]\nversion = 1\ndisabled = [\"a\", \"b\"]\n";
    fs::write(rig.files().settings, off).unwrap();
    rig.add(sample(A).provides(&[Alpha::ID]));
    rig.add(sample(B).provides(&[Beta::ID]));
    rig.add(sample(C).requires(&[Alpha::ID, Beta::ID]));
    rig.kernel.boot().unwrap();
    rig.kernel.set_enabled(A, true).unwrap();
    let states = [A, B, C].map(|m| rig.state(m));
    assert_eq!(states, [Some(Active), Some(Stopped), Some(Stopped)]);
    assert_eq!(rig.notices(), [], "waiting for B is no failure");
    rig.kernel.set_enabled(B, true).unwrap();
    assert!([A, B, C].iter().all(|&m| rig.state(m) == Some(Active)));
}

#[test]
fn dropping_the_kernel_stops_every_module() {
    let mut rig = Rig::new();
    chain(&mut rig, &Switch::new(Mode::Succeed));
    rig.kernel.boot().unwrap();
    let Rig { kernel, log, .. } = rig;
    drop(kernel);
    assert_eq!(log.stops(), [D, C, B, A]);
}

#[test]
fn switching_refuses_the_kernel_unknown_modules_and_an_unbooted_kernel() {
    let mut rig = Rig::new();
    rig.add(sample(A));
    let early = rig.kernel.set_enabled(A, false);
    assert!(matches!(early, Err(KernelError::WrongPhase(Phase::Setup))));
    rig.kernel.boot().unwrap();
    let kernel = rig.kernel.set_enabled(KERNEL, false);
    assert!(matches!(kernel, Err(KernelError::Reserved(KERNEL))));
    let nobody = rig.kernel.set_enabled(NOBODY, false);
    assert!(matches!(nobody, Err(KernelError::Unknown(NOBODY))));
    assert_eq!(rig.state(A), Some(Active));
}
