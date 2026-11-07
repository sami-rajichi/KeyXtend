//! A run that keeps nothing (ADR-0037): a fresh kernel reads no settings file, saves none and keeps its off list off
//! for that run; a normal kernel saves the hidden copies the user kept.

use crate::rig::Rig;
use crate::samples::{LEVEL_KEY, LEVEL_SPEC, NEW_LEVEL, sample};
use kx_kernel::{KEPT_KEY, KERNEL, KernelError, Phase};
use kx_module_api::ModuleId;
use kx_module_api::ModuleState::{Active, Stopped};
use std::fs;
use toml::Value;

const A: ModuleId = ModuleId::new("a");
const B: ModuleId = ModuleId::new("b");
/// A package folder's name, as `kept_copies` holds it.
const PACKAGE: &str = "Pkg_x";

#[test]
fn a_fresh_kernel_reads_no_settings_file_and_saves_nothing() {
    let mut rig = Rig::new();
    rig.add(sample(A).settings(&LEVEL_SPEC));
    rig.off_in_file(&[A]);
    let file = fs::read_to_string(rig.files().settings).unwrap();
    rig.kernel.fresh(&[]).unwrap();
    assert_eq!(rig.kernel.boot().unwrap(), []);
    assert_eq!(
        rig.state(A),
        Some(Active),
        "the file's off list is never read"
    );
    let level = Value::Integer(NEW_LEVEL);
    rig.kernel.set_setting(A, LEVEL_KEY, level).unwrap();
    rig.kernel.set_enabled(A, false).unwrap();
    assert_eq!(fs::read_to_string(rig.files().settings).unwrap(), file);
    assert_eq!(
        fs::read_dir(rig.data()).unwrap().count(),
        1,
        "no copy or temp file"
    );
}

#[test]
fn a_fresh_kernel_keeps_its_off_list_off_for_that_run_only() {
    let mut rig = Rig::new();
    rig.add(sample(A));
    rig.add(sample(B));
    rig.kernel.fresh(&[A]).unwrap();
    assert_eq!(rig.kernel.boot().unwrap(), [], "kept off is no failure");
    assert_eq!([A, B].map(|m| rig.state(m)), [Some(Stopped), Some(Active)]);
    let own = rig.kernel.kernel_settings().unwrap();
    assert_eq!(own.disabled, [A.as_str()]);
    let refused = rig.kernel.set_enabled(A, true);
    assert!(
        matches!(refused, Err(KernelError::KeptOff(m)) if m == A),
        "{refused:?}"
    );
    let own = rig.kernel.kernel_settings().unwrap();
    assert_eq!(own.disabled, [A.as_str()], "the settings still show it off");
    assert_eq!(rig.state(A), Some(Stopped), "it stays off");
    rig.kernel.set_enabled(B, false).unwrap();
    rig.kernel.set_enabled(B, true).unwrap();
    assert_eq!(
        rig.state(B),
        Some(Active),
        "a module not kept off switches as before"
    );
    assert!(!rig.data().exists(), "nothing was written");
}

#[test]
fn fresh_is_set_only_before_load() {
    let mut rig = Rig::new();
    rig.kernel.load().unwrap();
    let late = rig.kernel.fresh(&[A]);
    assert!(matches!(late, Err(KernelError::WrongPhase(Phase::Loaded))));
}

#[test]
fn a_kept_copy_is_saved_in_the_kernels_section() {
    let mut rig = Rig::new();
    rig.kernel.boot().unwrap();
    let kept = Value::Array(vec![Value::from(PACKAGE)]);
    rig.kernel.set_setting(KERNEL, KEPT_KEY, kept).unwrap();
    let text = fs::read_to_string(rig.files().settings).unwrap();
    assert!(text.contains(PACKAGE), "{text}");
    let own = rig.kernel.kernel_settings().unwrap();
    assert_eq!(own.kept_copies, [PACKAGE]);
}
