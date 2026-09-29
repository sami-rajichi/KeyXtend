//! The phase guard: setup calls only before boot, controls only while running, nothing after stop.

use crate::record::Mode;
use crate::rig::Rig;
use crate::samples::{Alpha, LEVEL_SPEC, PLATFORM, sample, service};
use kx_kernel::{KernelError, Phase};
use kx_module_api::ModuleState::{Active, Failed};
use kx_module_api::{ModuleId, ServiceKey};
use toml::Value;

const A: ModuleId = ModuleId::new("a");
const B: ModuleId = ModuleId::new("b");
const LEVEL: &str = "level";
const NEW_LEVEL: i64 = 5;

/// True when `result` is the refusal naming `phase`.
fn refused<T>(result: &Result<T, KernelError>, phase: Phase) -> bool {
    matches!(result, Err(KernelError::WrongPhase(p)) if *p == phase)
}

/// Tries every runtime control on A, B and A's level, and checks each is refused in `phase`.
fn controls_refused(rig: &mut Rig, phase: Phase) {
    assert!(refused(&rig.kernel.retry(A), phase));
    assert!(refused(&rig.kernel.set_enabled(B, false), phase));
    let level = Value::Integer(NEW_LEVEL);
    assert!(refused(&rig.kernel.set_setting(B, LEVEL, level), phase));
}

#[test]
fn controls_are_refused_before_boot() {
    let mut rig = Rig::new();
    rig.add(sample(A).mode(Mode::Fail));
    rig.add(sample(B).settings(&LEVEL_SPEC));
    controls_refused(&mut rig, Phase::Setup);
    rig.kernel.boot().unwrap();
    assert_eq!([A, B].map(|m| rig.state(m)), [Some(Failed), Some(Active)]);
}

#[test]
fn setup_calls_and_a_second_boot_are_refused_while_running() {
    let mut rig = Rig::new();
    rig.add(sample(A));
    rig.kernel.boot().unwrap();
    let late = rig.kernel.add(sample(B).build(&rig.log));
    assert!(refused(&late, Phase::Running));
    let alpha = service(Alpha::ID, PLATFORM, &rig.log);
    let given = rig.kernel.provide_platform::<Alpha>(alpha);
    assert!(refused(&given, Phase::Running));
    assert!(refused(&rig.kernel.boot(), Phase::Running));
    assert_eq!(rig.state(B), None);
    assert_eq!(rig.log.starts(), [A], "the second boot started nothing");
}

#[test]
fn only_a_second_stop_is_allowed_after_stop_all() {
    let mut rig = Rig::new();
    rig.add(sample(A).mode(Mode::Fail));
    rig.add(sample(B).settings(&LEVEL_SPEC));
    rig.kernel.boot().unwrap();
    rig.kernel.stop_all();
    rig.log.clear();
    controls_refused(&mut rig, Phase::Stopped);
    let late = rig
        .kernel
        .add(sample(ModuleId::new("late")).build(&rig.log));
    assert!(refused(&late, Phase::Stopped));
    assert!(refused(&rig.kernel.boot(), Phase::Stopped));
    rig.kernel.stop_all();
    assert_eq!(rig.log.all(), [], "nothing started or stopped");
}
