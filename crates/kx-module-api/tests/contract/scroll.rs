//! The scroll target port and the pad's events: the `Scroll` gate, a run reached through it on the shared fake
//! target that closes when dropped, and events any module may send.

use kx_module_api::Capability::{InjectInput, Scroll};
use kx_module_api::ports::{
    Aim, Dir, Point, PointerPressed, Reach, Rect, ScrollAimed, ScrollAsk, ScrollChanged,
    ScrollConfig, ScrollError, ScrollPage, ScrollPeek, ScrollStop, ScrollTap, ScrollTarget,
    ScrollTargetKey, TargetId,
};
use kx_module_api::{Capability, Host, Manifest, ModuleCx, ModuleId, ServiceError, ServiceKey};
use kx_platform_fake::{FakeScrollTarget, Turn};
use kx_test_support::host::FakeHost;
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

/// The one target the fake finds.
const AIM: Aim = Aim {
    at: Point { x: 5, y: 6 },
    on_pointer: false,
    area: Some(Rect {
        x: 1,
        y: 2,
        w: 3,
        h: 4,
    }),
    ours: false,
    id: TargetId::new(1),
};

/// A scroll host granted `granted`, holding the shared fake target as the platform would provide it.
fn host(granted: &[Capability]) -> (FakeHost, FakeScrollTarget) {
    let target = FakeScrollTarget::new();
    let api: Arc<dyn ScrollTarget> = Arc::new(target.clone());
    let mut host = FakeHost::new(&SCROLLER);
    host.granted.extend(granted);
    let stored = (ScrollTargetKey::CAPABILITY, Box::new(api) as _);
    host.services.insert(ScrollTargetKey::ID, stored);
    (host, target)
}

#[test]
fn a_module_without_scroll_cannot_get_the_target() {
    let (mut host, _target) = host(&[InjectInput]);
    let cx = ModuleCx::new(&mut host);
    let refused = ServiceError::NotGranted(ScrollTargetKey::ID, Scroll);
    assert_eq!(cx.service::<ScrollTargetKey>().err(), Some(refused));
}

#[test]
fn a_run_reached_through_the_gate_closes_when_dropped() {
    let (mut host, fake) = host(&[Scroll]);
    fake.push_aim(Ok(AIM));
    fake.set_reach(Dir::Down, Reach::End);
    let target = ModuleCx::new(&mut host)
        .service::<ScrollTargetKey>()
        .unwrap();
    let mut run = target.open(CONFIG).unwrap();
    assert_eq!(fake.config(), Some(CONFIG));
    let aim = run.aim().unwrap();
    assert_eq!(run.turn(&aim, Dir::Down, 0.25), Ok(()));
    fake.set_turn_error(Some(ScrollError::Gone));
    assert_eq!(run.turn(&aim, Dir::Up, 1.0), Err(ScrollError::Gone));
    assert_eq!(run.reach(&aim, Dir::Down), Reach::End);
    let took = Turn {
        id: AIM.id,
        dir: Dir::Down,
        lines: 0.25,
    };
    assert_eq!(fake.turns(), [took]);
    assert!(fake.running());
    drop(run);
    assert!(!fake.running(), "the drop closed it");
}

#[test]
fn the_pads_events_pass_on_a_bus_with_no_grants() {
    let (host, _target) = host(&[]);
    let bus = host.bus();
    assert_eq!(bus.publish(&ScrollTap { dir: Dir::Left }), Ok(()));
    assert_eq!(bus.publish(&ScrollStop), Ok(()));
    assert_eq!(bus.publish(&ScrollPeek { on: true }), Ok(()));
    assert_eq!(bus.publish(&ScrollPage { back: true }), Ok(()));
    assert_eq!(bus.publish(&ScrollChanged::OFF), Ok(()));
    assert_eq!(bus.publish(&ScrollAimed { area: AIM.area }), Ok(()));
    assert_eq!(bus.publish(&ScrollAsk), Ok(()));
}

#[test]
fn a_module_holding_only_scroll_hears_a_press() {
    let (host, _target) = host(&[Scroll]);
    let seen = Arc::new(Mutex::new(Vec::new()));
    let log = Arc::clone(&seen);
    let _sub = host
        .bus()
        .subscribe(move |p: &PointerPressed| log.lock().unwrap().push(p.ours))
        .unwrap();
    host.bus().publish(&PointerPressed { ours: false }).unwrap();
    assert_eq!(*seen.lock().unwrap(), [false]);
}
