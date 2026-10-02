//! A module's live settings handle: its own section only, checked, saved at once, never a restart.

use crate::record::{Did, Grab};
use crate::rig::{Rig, Seen};
use crate::samples::{LEVEL_KEY, LEVEL_SPEC, MAX_LEVEL, NEW_LEVEL, sample};
use kx_module_api::ModuleState::{Active, Stopped};
use kx_module_api::{
    Manifest, Module, ModuleCx, ModuleError, ModuleId, SettingsChanged, SettingsError, WriteError,
    keys,
};
use std::fs;
use toml::{Table, Value};

const A: ModuleId = ModuleId::new("a");
const B: ModuleId = ModuleId::new("b");
/// A key the sample settings do not have.
const UNKNOWN_KEY: &str = "speed";

/// A module that saves its own level while it starts, as the UI saves its size and place.
struct Writer;

static WRITER: Manifest = Manifest {
    id: ModuleId::new("writer"),
    version: env!("CARGO_PKG_VERSION"),
    requires: &[],
    provides: &[],
    capabilities: &[],
    settings: Some(&LEVEL_SPEC),
};

impl Module for Writer {
    fn manifest(&self) -> &'static Manifest {
        &WRITER
    }

    fn start(&mut self, cx: &mut ModuleCx<'_>) -> Result<(), ModuleError> {
        let saved = cx.settings_handle().set(LEVEL_KEY, level(NEW_LEVEL));
        assert_eq!(
            saved,
            Err(WriteError::Unsaved),
            "the data folder is blocked"
        );
        Ok(())
    }
}

/// A tuned by its level and B with no settings, each leaving its handle in its grab.
fn grabbed() -> (Rig, Grab, Grab) {
    let (mut rig, a, b) = (Rig::new(), Grab::default(), Grab::default());
    rig.add(sample(A).settings(&LEVEL_SPEC).grab(&a));
    rig.add(sample(B).grab(&b));
    (rig, a, b)
}

fn level(value: i64) -> Value {
    Value::Integer(value)
}

#[test]
fn a_module_write_is_saved_at_once_with_no_restart_and_no_announcement() {
    let (mut rig, a, _) = grabbed();
    let (changed, _watch) = Seen::<SettingsChanged>::watch(&rig.kernel.bus());
    rig.kernel.boot().unwrap();
    a.get().set(LEVEL_KEY, level(NEW_LEVEL)).unwrap();
    let text = fs::read_to_string(rig.files().settings).unwrap();
    assert_eq!(text, format!("[a]\nversion = 1\nlevel = {NEW_LEVEL}\n"));
    assert_eq!(rig.log.starts(), [A, B], "each started once");
    assert_eq!(changed.all(), [], "nothing announced");
    assert_eq!(rig.state(A), Some(Active));
}

#[test]
fn an_unknown_key_or_a_refused_value_writes_nothing() {
    let (mut rig, a, _) = grabbed();
    rig.kernel.boot().unwrap();
    let unknown = a.get().set(UNKNOWN_KEY, level(NEW_LEVEL));
    assert!(matches!(
        unknown,
        Err(WriteError::Invalid(SettingsError::Parse(_)))
    ));
    let high = a.get().set(LEVEL_KEY, level(MAX_LEVEL + 1));
    assert!(matches!(
        high,
        Err(WriteError::Invalid(SettingsError::Invalid(_)))
    ));
    assert!(!rig.files().settings.exists(), "nothing saved");
}

#[test]
fn set_all_saves_every_value_or_none() {
    let (mut rig, a, _) = grabbed();
    rig.kernel.boot().unwrap();
    let good = (LEVEL_KEY.to_owned(), level(NEW_LEVEL));
    let mixed = Table::from_iter([good.clone(), (UNKNOWN_KEY.to_owned(), level(1))]);
    let refused = a.get().set_all(mixed);
    assert!(matches!(
        refused,
        Err(WriteError::Invalid(SettingsError::Parse(_)))
    ));
    assert!(!rig.files().settings.exists(), "not even the good value");
    a.get().set_all(Table::from_iter([good])).unwrap();
    let text = fs::read_to_string(rig.files().settings).unwrap();
    assert_eq!(text, format!("[a]\nversion = 1\nlevel = {NEW_LEVEL}\n"));
}

#[test]
fn a_handle_cannot_write_another_module_section() {
    let (mut rig, _, b) = grabbed();
    rig.kernel.boot().unwrap();
    let refused = b.get().set(LEVEL_KEY, level(NEW_LEVEL));
    assert_eq!(refused, Err(WriteError::Refused(B)));
    assert!(!rig.files().settings.exists(), "A's level is untouched");
}

#[test]
fn a_failed_save_is_unsaved_and_told_once() {
    let (mut rig, a, _) = grabbed();
    fs::write(rig.data(), "a file where the data folder should be").unwrap();
    rig.kernel.boot().unwrap();
    let first = a.get().set(LEVEL_KEY, level(NEW_LEVEL));
    let second = a.get().set(LEVEL_KEY, level(NEW_LEVEL + 1));
    assert_eq!(
        [first, second],
        [Err(WriteError::Unsaved), Err(WriteError::Unsaved)]
    );
    rig.kernel.set_enabled(A, false).unwrap();
    rig.kernel.set_enabled(A, true).unwrap();
    assert!(
        rig.log.all().contains(&Did::Saw(A, NEW_LEVEL + 1)),
        "the unsaved value holds for the run"
    );
    assert_eq!(rig.notice_keys(), [keys::SETTINGS_UNSAVED]);
}

#[test]
fn a_failed_save_while_booting_is_in_the_boot_list_and_on_the_bus_once() {
    let mut rig = Rig::new();
    rig.kernel.add(Box::new(Writer)).unwrap();
    fs::write(rig.data(), "a file where the data folder should be").unwrap();
    let notices = rig.kernel.boot().unwrap();
    let told: Vec<_> = notices.iter().map(|n| n.key).collect();
    assert_eq!(told, [keys::SETTINGS_UNSAVED]);
    assert_eq!(rig.notice_keys(), [keys::SETTINGS_UNSAVED]);
    assert_eq!(rig.state(WRITER.id), Some(Active));
}

#[test]
fn a_handle_keeps_working_after_its_module_stops() {
    let (mut rig, a, _) = grabbed();
    rig.kernel.boot().unwrap();
    rig.kernel.set_enabled(A, false).unwrap();
    assert_eq!(rig.state(A), Some(Stopped));
    a.get().set(LEVEL_KEY, level(NEW_LEVEL)).unwrap();
    rig.kernel.set_enabled(A, true).unwrap();
    assert!(
        rig.log.all().contains(&Did::Saw(A, NEW_LEVEL)),
        "the next start sees it"
    );
    rig.kernel.stop_all();
    a.get().set(LEVEL_KEY, level(NEW_LEVEL + 1)).unwrap();
    let text = fs::read_to_string(rig.files().settings).unwrap();
    assert!(
        text.contains(&format!("level = {}", NEW_LEVEL + 1)),
        "{text}"
    );
}
