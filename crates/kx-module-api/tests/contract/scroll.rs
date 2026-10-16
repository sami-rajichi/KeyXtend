//! The scroll target port and the pad's events: the `Scroll` gate, a run reached through it that closes when dropped,
//! and events any module may send.

use kx_module_api::Capability::{InjectInput, Scroll};
use kx_module_api::ports::{
    Aim, Dir, Point, PointerPressed, Reach, ScrollAsk, ScrollChanged, ScrollConfig, ScrollError,
    ScrollPage, ScrollRun, ScrollStop, ScrollTap, ScrollTarget, ScrollTargetKey, TargetId,
};
use kx_module_api::{Capability, Host, Manifest, ModuleCx, ModuleId, ServiceError, ServiceKey};
use kx_test_support::host::FakeHost;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

/// A scroll module declaring `Scroll` and `InjectInput`; each test grants it some, as the kernel's policy would.
static SCROLLER: Manifest = Manifest {
    id: ModuleId::new("scroll"),
    version: env!("CARGO_PKG_VERSION"),
    requires: &[ScrollTargetKey::ID],
    provides: &[],
    capabilities: &[Scroll, InjectInput],
    settings: None,
};

/// Test values; the real ones come from the scroll module's settings.
const CONFIG: ScrollConfig = ScrollConfig {
    check_ms: 1,
    read_ms: 2,
    edge_px: 3,
};

/// The one target the stub finds.
const AIM: Aim = Aim {
    at: Point { x: 5, y: 6 },
    on_pointer: false,
    area: None,
    ours: false,
    id: TargetId::new(1),
};

/// A target that finds `AIM` and counts its open runs.
#[derive(Default)]
struct Stub {
    open: Arc<AtomicUsize>,
}

/// A run that turns only `AIM` and is always at its end.
struct Run(Arc<AtomicUsize>);

impl ScrollTarget for Stub {
    fn open(&self, _: ScrollConfig) -> Result<Box<dyn ScrollRun>, ScrollError> {
        self.open.fetch_add(1, Ordering::SeqCst);
        Ok(Box::new(Run(Arc::clone(&self.open))))
    }
}

impl ScrollRun for Run {
    fn aim(&mut self) -> Result<Aim, ScrollError> {
        Ok(AIM)
    }

    fn turn(&mut self, aim: &Aim, _: Dir, _: f64) -> Result<(), ScrollError> {
        if aim.id == AIM.id {
            Ok(())
        } else {
            Err(ScrollError::Gone)
        }
    }

    fn reach(&mut self, _: &Aim, _: Dir) -> Reach {
        Reach::End
    }
}

impl Drop for Run {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}

/// A scroll host granted `granted`, holding the stub as the platform would provide it.
fn host(granted: &[Capability]) -> (FakeHost, Arc<AtomicUsize>) {
    let stub = Stub::default();
    let open = Arc::clone(&stub.open);
    let api: Arc<dyn ScrollTarget> = Arc::new(stub);
    let mut host = FakeHost::new(&SCROLLER);
    host.granted.extend(granted);
    let stored = (ScrollTargetKey::CAPABILITY, Box::new(api) as _);
    host.services.insert(ScrollTargetKey::ID, stored);
    (host, open)
}

#[test]
fn a_module_without_scroll_cannot_get_the_target() {
    let (mut host, _open) = host(&[InjectInput]);
    let cx = ModuleCx::new(&mut host);
    let refused = ServiceError::NotGranted(ScrollTargetKey::ID, Scroll);
    assert_eq!(cx.service::<ScrollTargetKey>().err(), Some(refused));
}

#[test]
fn a_run_reached_through_the_gate_closes_when_dropped() {
    let (mut host, open) = host(&[Scroll]);
    let target = ModuleCx::new(&mut host)
        .service::<ScrollTargetKey>()
        .unwrap();
    let mut run = target.open(CONFIG).unwrap();
    let aim = run.aim().unwrap();
    assert_eq!(run.turn(&aim, Dir::Down, 0.25), Ok(()));
    let other = Aim {
        id: TargetId::new(2),
        ..aim
    };
    assert_eq!(run.turn(&other, Dir::Up, 1.0), Err(ScrollError::Gone));
    assert_eq!(run.reach(&aim, Dir::Down), Reach::End);
    assert_eq!(open.load(Ordering::SeqCst), 1);
    drop(run);
    assert_eq!(open.load(Ordering::SeqCst), 0, "the drop closed it");
}

#[test]
fn the_pads_events_pass_on_a_bus_with_no_grants() {
    let (host, _open) = host(&[]);
    let bus = host.bus();
    assert_eq!(bus.publish(&ScrollTap { dir: Dir::Left }), Ok(()));
    assert_eq!(bus.publish(&ScrollStop), Ok(()));
    assert_eq!(bus.publish(&ScrollPage { back: true }), Ok(()));
    assert_eq!(bus.publish(&ScrollChanged::OFF), Ok(()));
    assert_eq!(bus.publish(&ScrollAsk), Ok(()));
}

#[test]
fn a_module_holding_only_scroll_hears_a_press() {
    let (host, _open) = host(&[Scroll]);
    let seen = Arc::new(Mutex::new(Vec::new()));
    let log = Arc::clone(&seen);
    let _sub = host
        .bus()
        .subscribe(move |p: &PointerPressed| log.lock().unwrap().push(p.ours))
        .unwrap();
    host.bus().publish(&PointerPressed { ours: false }).unwrap();
    assert_eq!(*seen.lock().unwrap(), [false]);
}
