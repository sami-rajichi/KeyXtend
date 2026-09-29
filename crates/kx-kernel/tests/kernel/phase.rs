//! The phase guard: setup calls only before load, start only after it, controls only while
//! running, nothing after stop.

use crate::record::Mode;
use crate::rig::{Rig, failed};
use crate::samples::{
    Alpha, BROKEN_SPEC, LEVEL_KEY, LEVEL_SPEC, NEW_LEVEL, PLATFORM, sample, service,
};
use kx_kernel::log::LOG_LEVELS;
use kx_kernel::{KernelError, Phase};
use kx_module_api::ModuleState::{Active, Failed, Pending};
use kx_module_api::{ModuleId, ServiceKey};
use std::fs;
use toml::Value;

const A: ModuleId = ModuleId::new("a");
const B: ModuleId = ModuleId::new("b");
const C: ModuleId = ModuleId::new("c");

/// True when `result` is the refusal naming `phase`.
fn refused<T>(result: &Result<T, KernelError>, phase: Phase) -> bool {
    matches!(result, Err(KernelError::WrongPhase(p)) if *p == phase)
}

/// Tries every runtime control on A, B and A's level, and checks each is refused in `phase`.
fn controls_refused(rig: &mut Rig, phase: Phase) {
    assert!(refused(&rig.kernel.retry(A), phase));
    assert!(refused(&rig.kernel.set_enabled(B, false), phase));
    let level = Value::Integer(NEW_LEVEL);
    assert!(refused(&rig.kernel.set_setting(B, LEVEL_KEY, level), phase));
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

#[test]
fn load_reads_the_kernel_section_and_fails_only_what_cannot_start() {
    let mut rig = Rig::new();
    let level = LOG_LEVELS[LOG_LEVELS.len() - 1];
    fs::create_dir_all(rig.data()).unwrap();
    let text = format!("[kernel]\nlog_level = \"{level}\"\n");
    fs::write(rig.files().settings, text).unwrap();
    rig.add(sample(A));
    rig.add(sample(B).settings(&LEVEL_SPEC));
    rig.add(sample(C).settings(&BROKEN_SPEC));
    assert_eq!(rig.kernel.load().unwrap(), [failed(C)]);
    let states = [A, B, C].map(|m| rig.state(m));
    assert_eq!(states, [Some(Pending), Some(Pending), Some(Failed)]);
    assert_eq!(rig.log.all(), [], "nothing started");
    let own = rig.kernel.kernel_settings().unwrap();
    assert_eq!(own.log_level, level);
    assert_eq!(rig.kernel.start().unwrap(), []);
    assert_eq!([A, B].map(|m| rig.state(m)), [Some(Active), Some(Active)]);
}

#[test]
fn start_needs_a_load_first_and_each_runs_once() {
    let mut rig = Rig::new();
    rig.add(sample(A).mode(Mode::Fail));
    rig.add(sample(B).settings(&LEVEL_SPEC));
    assert!(refused(&rig.kernel.start(), Phase::Setup));
    assert_eq!(rig.kernel.kernel_settings(), None);
    rig.kernel.load().unwrap();
    assert!(refused(&rig.kernel.load(), Phase::Loaded));
    assert!(refused(&rig.kernel.boot(), Phase::Loaded));
    let late = rig.kernel.add(sample(C).build(&rig.log));
    assert!(refused(&late, Phase::Loaded));
    controls_refused(&mut rig, Phase::Loaded);
    assert_eq!(rig.kernel.start().unwrap(), [failed(A)]);
    assert!(refused(&rig.kernel.start(), Phase::Running));
    assert_eq!([A, B].map(|m| rig.state(m)), [Some(Failed), Some(Active)]);
}

#[test]
fn stopping_after_a_load_saves_its_repairs_and_starts_nothing() {
    let mut rig = Rig::new();
    fs::create_dir_all(rig.data()).unwrap();
    fs::write(rig.files().settings, "[b]\nlevel = \"high\"\n").unwrap();
    rig.add(sample(B).settings(&LEVEL_SPEC));
    rig.kernel.load().unwrap();
    rig.kernel.stop_all();
    assert_eq!(fs::read_to_string(rig.files().settings).unwrap(), "");
    assert_eq!(rig.log.all(), [], "nothing started or stopped");
}
