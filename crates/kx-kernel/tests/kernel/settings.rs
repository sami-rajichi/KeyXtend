//! Settings through the kernel: changes saved alone, restarts, unwritable folders and every notice.

use crate::record::{Did, Mode, Switch};
use crate::rig::{Rig, Seen, failed};
use crate::samples::{Alpha, DEFAULT_LEVEL, LEVEL_KEY, LEVEL_SPEC, MAX_LEVEL, NEW_LEVEL, sample};
use kx_kernel::{KERNEL, KernelError, Phase};
use kx_module_api::ModuleState::{Active, Failed};
use kx_module_api::{ModuleId, ServiceKey, SettingsChanged, keys};
use kx_settings::{ARG_KEYS, StoreError};
use std::fs;
use toml::Value;

const A: ModuleId = ModuleId::new("a");
const B: ModuleId = ModuleId::new("b");
const C: ModuleId = ModuleId::new("c");

/// A tuned by its level, B needing A's service, and C on its own.
fn tuned() -> Rig {
    let mut rig = Rig::new();
    rig.add(sample(A).provides(&[Alpha::ID]).settings(&LEVEL_SPEC));
    rig.add(sample(B).requires(&[Alpha::ID]));
    rig.add(sample(C));
    rig
}

fn set(rig: &mut Rig, level: i64) -> Result<(), KernelError> {
    rig.kernel.set_setting(A, LEVEL_KEY, Value::Integer(level))
}

fn count(rig: &Rig, key: &str) -> usize {
    rig.notice_keys().iter().filter(|&&k| k == key).count()
}

#[test]
fn a_setting_is_saved_alone_announced_and_seen_after_a_restart() {
    let mut rig = tuned();
    let (changed, _watch) = Seen::<SettingsChanged>::watch(&rig.kernel.bus());
    rig.kernel.boot().unwrap();
    assert!(rig.log.all().contains(&Did::Saw(A, DEFAULT_LEVEL)));
    rig.log.clear();
    set(&mut rig, NEW_LEVEL).unwrap();
    let text = fs::read_to_string(rig.files().settings).unwrap();
    assert_eq!(text, format!("[a]\nversion = 1\nlevel = {NEW_LEVEL}\n"));
    assert_eq!(changed.all(), [SettingsChanged { module: A }]);
    let restart = [
        Did::Stop(B),
        Did::Stop(A),
        Did::Dropped(Alpha::ID),
        Did::Start(A),
        Did::Saw(A, NEW_LEVEL),
        Did::Start(B),
        Did::Got(B, A.as_str()),
    ];
    assert_eq!(rig.log.all(), restart, "C keeps running");
    assert!([A, B, C].iter().all(|&m| rig.state(m) == Some(Active)));
}

#[test]
fn a_provider_that_fails_after_a_setting_change_fails_its_dependents_and_try_again_recovers() {
    let (mut rig, a) = (Rig::new(), Switch::new(Mode::Succeed));
    rig.add(
        sample(A)
            .provides(&[Alpha::ID])
            .settings(&LEVEL_SPEC)
            .switch(&a),
    );
    rig.add(sample(B).requires(&[Alpha::ID]));
    rig.add(sample(C));
    rig.kernel.boot().unwrap();
    a.set(Mode::Fail);
    set(&mut rig, NEW_LEVEL).unwrap();
    let states = [A, B, C].map(|m| rig.state(m));
    assert_eq!(states, [Some(Failed), Some(Failed), Some(Active)]);
    assert_eq!(rig.notices(), [failed(A), failed(B)]);
    a.set(Mode::Succeed);
    rig.log.clear();
    rig.kernel.retry(A).unwrap();
    assert!([A, B, C].iter().all(|&m| rig.state(m) == Some(Active)));
    assert!(rig.log.all().contains(&Did::Saw(A, NEW_LEVEL)));
}

#[test]
fn setting_the_current_value_restarts_nothing_and_announces_nothing() {
    let mut rig = tuned();
    let (changed, _watch) = Seen::<SettingsChanged>::watch(&rig.kernel.bus());
    rig.kernel.boot().unwrap();
    rig.log.clear();
    set(&mut rig, DEFAULT_LEVEL).unwrap();
    assert_eq!(rig.log.all(), [], "nothing restarted");
    assert_eq!(changed.all(), [], "nothing announced");
    assert!(!rig.files().settings.exists(), "nothing saved");
}

#[test]
fn a_refused_setting_changes_nothing() {
    let mut rig = tuned();
    assert!(matches!(
        set(&mut rig, NEW_LEVEL),
        Err(KernelError::WrongPhase(Phase::Setup))
    ));
    rig.kernel.boot().unwrap();
    rig.log.clear();
    let high = set(&mut rig, MAX_LEVEL + 1);
    assert!(matches!(
        high,
        Err(KernelError::Settings(StoreError::Settings(_)))
    ));
    let list = Value::Array(vec![A.as_str().into()]);
    let off = rig.kernel.set_setting(KERNEL, "disabled", list);
    assert!(matches!(off, Err(KernelError::SwitchKey)));
    assert_eq!(rig.log.all(), [], "nothing restarted");
    assert!(!rig.files().settings.exists(), "nothing saved");
}

#[test]
fn an_unwritable_data_folder_runs_on_defaults_and_warns_once() {
    let mut rig = tuned();
    fs::write(rig.data(), "a file where the data folder should be").unwrap();
    rig.kernel.boot().unwrap();
    assert!([A, B, C].iter().all(|&m| rig.state(m) == Some(Active)));
    assert!(rig.log.all().contains(&Did::Saw(A, DEFAULT_LEVEL)));
    set(&mut rig, NEW_LEVEL).unwrap();
    set(&mut rig, NEW_LEVEL + 1).unwrap();
    assert!(rig.log.all().contains(&Did::Saw(A, NEW_LEVEL + 1)));
    assert_eq!(
        count(&rig, keys::SETTINGS_UNSAVED),
        1,
        "{:?}",
        rig.notices()
    );
}

#[test]
fn stopping_makes_one_last_save_of_a_change_that_failed_to_save() {
    let mut rig = tuned();
    fs::write(rig.data(), "a file where the data folder should be").unwrap();
    rig.kernel.boot().unwrap();
    set(&mut rig, NEW_LEVEL).unwrap();
    fs::remove_file(rig.data()).unwrap();
    rig.kernel.stop_all();
    let text = fs::read_to_string(rig.files().settings).unwrap();
    assert_eq!(
        text,
        format!(
            "[a]
version = 1
level = {NEW_LEVEL}
"
        )
    );
    fs::remove_file(rig.files().settings).unwrap();
    rig.kernel.stop_all();
    assert!(
        !rig.files().settings.exists(),
        "a second stop saves nothing"
    );
}

#[test]
fn stopping_saves_nothing_when_nothing_changed() {
    let mut rig = tuned();
    rig.kernel.boot().unwrap();
    rig.kernel.stop_all();
    assert!(!rig.files().settings.exists(), "nothing saved");
}

#[test]
fn a_last_save_that_fails_again_is_tried_once_and_told_once() {
    let mut rig = tuned();
    fs::write(rig.data(), "a file where the data folder should be").unwrap();
    rig.kernel.boot().unwrap();
    set(&mut rig, NEW_LEVEL).unwrap();
    rig.kernel.stop_all();
    rig.kernel.stop_all();
    assert_eq!(
        count(&rig, keys::SETTINGS_UNSAVED),
        1,
        "{:?}",
        rig.notices()
    );
    assert!(rig.data().is_file(), "the blocker is untouched");
}

#[test]
fn a_repair_whose_save_fails_warns_once_and_a_later_save_catches_up() {
    let mut rig = tuned();
    let files = rig.files();
    fs::create_dir_all(&files.temp).unwrap();
    fs::write(&files.settings, format!("[a]\nlevel = {}\n", MAX_LEVEL + 1)).unwrap();
    let notices = rig.kernel.boot().unwrap();
    let got: Vec<_> = notices.iter().map(|n| n.key).collect();
    assert_eq!(got, [keys::SETTINGS_RESET, keys::SETTINGS_UNSAVED]);
    set(&mut rig, NEW_LEVEL).unwrap();
    assert_eq!(count(&rig, keys::SETTINGS_UNSAVED), 1, "told once");
    fs::remove_dir(&files.temp).unwrap();
    set(&mut rig, NEW_LEVEL + 1).unwrap();
    let text = fs::read_to_string(&files.settings).unwrap();
    assert_eq!(
        text,
        format!("[a]\nversion = 1\nlevel = {}\n", NEW_LEVEL + 1)
    );
}

#[test]
fn every_notice_reaches_a_subscriber_registered_before_boot() {
    let mut rig = Rig::new();
    fs::create_dir_all(rig.data()).unwrap();
    let bad = "[kernel]\ndisabled = [\"\"]\nlog_level = \"loud\"\nlog_max_mb = 0\n";
    fs::write(rig.files().settings, bad).unwrap();
    rig.add(sample(A).mode(Mode::Fail));
    rig.add(sample(B));
    let notices = rig.kernel.boot().unwrap();
    assert_eq!(notices, rig.notices());
    let [reset, fail] = notices.as_slice() else {
        panic!("expected a reset and a failure: {notices:?}");
    };
    assert_eq!(
        (reset.key, reset.module),
        (keys::SETTINGS_RESET, Some(KERNEL))
    );
    let every = (ARG_KEYS, "disabled, log_level, log_max_mb".to_owned());
    assert!(reset.args.contains(&every), "{:?}", reset.args);
    assert_eq!(*fail, failed(A));
}
