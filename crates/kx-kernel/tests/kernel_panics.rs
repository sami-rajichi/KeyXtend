//! A panicking start or stop is contained: that module fails or stops and the rest keep running.
//! Setup runs once: it hides only the sample panics and captures every log line globally.

#![allow(
    clippy::unwrap_used,
    reason = "these tests fail on a broken setup, which is what a failed unwrap reports"
)]

#[allow(dead_code, reason = "this binary uses only part of the shared samples")]
#[path = "kernel/record.rs"]
mod record;
#[allow(dead_code, reason = "this binary uses only part of the shared samples")]
#[path = "kernel/rig.rs"]
mod rig;
#[allow(dead_code, reason = "this binary uses only part of the shared samples")]
#[path = "kernel/samples.rs"]
mod samples;

use kx_module_api::ModuleState::{Active, Failed, Stopped};
use kx_module_api::{ModuleId, ServiceKey, SettingsError};
use record::{Did, Mode, SamplePanic};
use rig::{Rig, failed};
use samples::{Alpha, FAILURE, LOOSE_SPEC, Ping, sample};
use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::panic;
use std::sync::{Mutex, Once, PoisonError};
use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id, Record};
use tracing::{Level, Metadata, Subscriber};

const A: ModuleId = ModuleId::new("a");
const B: ModuleId = ModuleId::new("b");
/// Modules only the log tests add, so each can pick its own lines.
const LOUD: ModuleId = ModuleId::new("loud");
const WILD: ModuleId = ModuleId::new("wild");
const LOOSE: ModuleId = ModuleId::new("loose");
/// A file value that no log line may ever show.
const FILE_VALUE: &str = "hunter2";
/// The id the capture hands every span; the kernel opens none.
const SPAN_ID: u64 = 1;

/// A log line's fields by name.
type Fields = BTreeMap<&'static str, String>;
/// One captured log line: its level and its fields.
type Line = (Level, Fields);

static SETUP: Once = Once::new();
static LINES: Mutex<Vec<Line>> = Mutex::new(Vec::new());

/// Installs the quiet hook and the global capture before any test logs.
fn setup() {
    SETUP.call_once(|| {
        let usual = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            if !info.payload().is::<SamplePanic>() {
                usual(info);
            }
        }));
        let installed = tracing::subscriber::set_global_default(Capture);
        assert!(installed.is_ok(), "only this setup sets the subscriber");
    });
}

#[test]
fn a_panicking_start_fails_only_that_module_and_rolls_back() {
    setup();
    let mut rig = Rig::new();
    rig.add(sample(A).provides(&[Alpha::ID]).mode(Mode::Panic));
    rig.add(sample(B));
    let notices = rig.kernel.boot().unwrap();
    assert_eq!([A, B].map(|m| rig.state(m)), [Some(Failed), Some(Active)]);
    assert_eq!(notices, [failed(A)]);
    assert!(rig.log.all().contains(&Did::Dropped(Alpha::ID)));
    rig.kernel.bus().publish(&Ping);
    assert_eq!(rig.log.pings(), [B], "its subscription is gone");
}

#[test]
fn a_panicking_stop_still_stops_and_releases_the_module() {
    setup();
    let mut rig = Rig::new();
    rig.add(sample(A).provides(&[Alpha::ID]).mode(Mode::PanicInStop));
    rig.add(sample(B).requires(&[Alpha::ID]));
    rig.kernel.boot().unwrap();
    rig.log.clear();
    rig.kernel.stop_all();
    let want = [Did::Stop(B), Did::Stop(A), Did::Dropped(Alpha::ID)];
    assert_eq!(rig.log.all(), want);
    assert_eq!([A, B].map(|m| rig.state(m)), [Some(Stopped), Some(Stopped)]);
}

#[test]
fn a_failed_start_logs_one_warning_with_only_the_module_and_the_error() {
    setup();
    let mut rig = Rig::new();
    rig.add(sample(LOUD).mode(Mode::Fail));
    rig.add(sample(WILD).mode(Mode::Panic));
    rig.kernel.boot().unwrap();
    let lines = LINES.lock().unwrap().clone();
    let warns = |id: ModuleId| -> Vec<Fields> {
        let about = |f: &Fields| f.get("module").is_some_and(|m| m == id.as_str());
        let warn = |(level, f): &Line| (*level == Level::WARN && about(f)).then(|| f.clone());
        lines.iter().filter_map(warn).collect()
    };
    for id in [LOUD, WILD] {
        let [fields] = warns(id).try_into().unwrap_or_else(|all| {
            panic!("expected one warning about {id}: {all:?}");
        });
        let names: Vec<_> = fields.keys().copied().collect();
        assert_eq!(names, ["error", "message", "module"]);
    }
    assert!(warns(LOUD)[0]["error"].contains(FAILURE));
}

#[test]
fn a_settings_failure_logs_its_kind_and_never_the_file_value() {
    setup();
    let mut rig = Rig::new();
    fs::create_dir_all(rig.data()).unwrap();
    let text = format!("[{LOOSE}]\nlevel = \"{FILE_VALUE}\"\n");
    fs::write(rig.files().settings, text).unwrap();
    rig.add(sample(LOOSE).settings(&LOOSE_SPEC));
    rig.kernel.boot().unwrap();
    assert_eq!(rig.state(LOOSE), Some(Failed));
    let lines = LINES.lock().unwrap().clone();
    let about = |(_, f): &&Line| f.get("module").is_some_and(|m| m == LOOSE.as_str());
    let [(_, fields)] = lines.iter().filter(about).collect::<Vec<_>>()[..] else {
        panic!("expected one line about {LOOSE}: {lines:?}");
    };
    let kind = SettingsError::Parse(String::new()).kind();
    assert_eq!(fields["error"], kind);
    assert!(!format!("{lines:?}").contains(FILE_VALUE));
}

/// The global subscriber: it keeps every log line in `LINES`.
struct Capture;

/// Collects one event's fields.
#[derive(Default)]
struct Recorder(Fields);

impl Visit for Recorder {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.0.insert(field.name(), format!("{value:?}"));
    }
}

impl Subscriber for Capture {
    fn enabled(&self, _: &Metadata<'_>) -> bool {
        true
    }

    fn new_span(&self, _: &Attributes<'_>) -> Id {
        Id::from_u64(SPAN_ID)
    }

    fn record(&self, _: &Id, _: &Record<'_>) {}

    fn record_follows_from(&self, _: &Id, _: &Id) {}

    fn event(&self, event: &tracing::Event<'_>) {
        let mut fields = Recorder::default();
        event.record(&mut fields);
        let line = (*event.metadata().level(), fields.0);
        LINES
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(line);
    }

    fn enter(&self, _: &Id) {}

    fn exit(&self, _: &Id) {}
}
