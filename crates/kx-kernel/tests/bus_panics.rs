//! Contained handler panics: later handlers still run, and one warning names only the handler.
//! Setup runs once: it silences only the planned panics and captures every log line globally.

use kx_kernel::bus::KernelBus;
use kx_module_api::{Bus, BusCore, Event, Flow, Subscription};
use std::any::{Any, TypeId};
use std::collections::BTreeMap;
use std::fmt;
use std::panic;
use std::sync::{Arc, Mutex, Once, PoisonError};
use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id, Record};
use tracing::{Level, Metadata, Subscriber};

/// The payload of every planned handler panic; it must never reach a log line.
const PLANNED: &str = "secret planned failure";

/// The id the capture hands every span; the bus opens none.
const SPAN_ID: u64 = 1;

/// One captured log line: its level and its fields by name.
type Line = (Level, BTreeMap<&'static str, String>);

static SETUP: Once = Once::new();
static LINES: Mutex<Vec<Line>> = Mutex::new(Vec::new());

/// Installs the quiet hook and the global capture before any test logs, so no callsite misses it.
fn setup() {
    SETUP.call_once(|| {
        let default = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            if info.payload().downcast_ref::<&str>() != Some(&PLANNED) {
                default(info);
            }
        }));
        let installed = tracing::subscriber::set_global_default(Capture);
        assert!(
            installed.is_ok(),
            "only this setup sets the global subscriber"
        );
    });
}

fn fail() -> ! {
    panic::panic_any(PLANNED)
}

/// An event that handlers stamp with their tags, in the order they run.
#[derive(Default)]
struct Trace(Vec<&'static str>);

impl Event for Trace {
    const NAME: &'static str = "trace";
}

/// An event only the log test publishes, so it can pick its own lines.
struct Probe;

impl Event for Probe {
    const NAME: &'static str = "probe";
}

fn bus() -> Bus {
    setup();
    Bus::new(Arc::new(KernelBus::new()))
}

fn stamp(bus: &Bus, order: i32, tag: &'static str) -> Subscription {
    bus.add_interceptor(order, move |trace: &mut Trace| {
        trace.0.push(tag);
        Flow::Continue
    })
}

#[test]
fn a_panicking_notify_handler_lets_later_handlers_run() {
    let (bus, log) = (bus(), Arc::new(Mutex::new(Vec::new())));
    let tagged = |tag: &'static str| {
        let log = Arc::clone(&log);
        move |_: &Trace| log.lock().unwrap_or_else(PoisonError::into_inner).push(tag)
    };
    let _subs = [
        bus.subscribe(tagged("before")),
        bus.subscribe(|_: &Trace| fail()),
        bus.subscribe(tagged("after")),
    ];
    bus.publish(&Trace::default());
    bus.publish(&Trace::default());
    assert_eq!(*log.lock().unwrap(), ["before", "after", "before", "after"]);
}

#[test]
fn a_panicking_interceptor_counts_as_continue() {
    let bus = bus();
    let _subs = [
        stamp(&bus, 1, "before"),
        bus.add_interceptor(2, |_: &mut Trace| -> Flow { fail() }),
        stamp(&bus, 3, "after"),
    ];
    let mut trace = Trace::default();
    assert_eq!(bus.intercept(&mut trace), Flow::Continue);
    assert_eq!(trace.0, ["before", "after"]);
}

/// The global subscriber: it keeps every log line in `LINES`.
struct Capture;

#[derive(Default)]
struct Fields(BTreeMap<&'static str, String>);

impl Visit for Fields {
    fn record_str(&mut self, field: &Field, value: &str) {
        self.0.insert(field.name(), value.to_owned());
    }

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
        let mut fields = Fields::default();
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

#[test]
fn a_panic_logs_one_warning_naming_only_the_event_and_the_id() {
    setup();
    let core = KernelBus::new();
    let fails = Box::new(|_: &(dyn Any + Send + Sync)| fail());
    let id = core.subscribe(TypeId::of::<Probe>(), Probe::NAME, fails);
    core.publish(TypeId::of::<Probe>(), Probe::NAME, &Probe);
    let lines = LINES.lock().unwrap().clone();
    let probe = |(_, fields): &&Line| fields.get("event").is_some_and(|e| e == Probe::NAME);
    let probes: Vec<&Line> = lines.iter().filter(probe).collect();
    let [(level, fields)] = probes.as_slice() else {
        panic!("expected one probe line in {lines:?}");
    };
    assert_eq!(*level, Level::WARN);
    let names: Vec<_> = fields.keys().copied().collect();
    assert_eq!(names, ["event", "message", "subscription"]);
    assert_eq!(fields["subscription"], id.get().to_string());
    assert!(fields.values().all(|value| !value.contains(PLANNED)));
}
