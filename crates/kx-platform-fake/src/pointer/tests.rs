//! The fake mouse hook, held to the port: ordered calls, one live run, release on drop and a checked buffer.

use super::*;
use kx_module_api::ports::{Button, Point};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicBool, Ordering};

mod rules;

/// Test values; the real ones come from the mouse module's settings.
const CONFIG: HookConfig = HookConfig {
    health_ms: 1,
    health_slack_ms: 2,
    echo_ms: 3,
};

const T1: Mono = Mono::from_us(10);
const T2: Mono = Mono::from_us(20);
const SPOT: Point = Point { x: 4, y: 5 };
const SEEN: PointerAct = PointerAct::Move(SPOT);
const CLICK: [PointerAct; 2] = [
    PointerAct::Down(Button::Right, SPOT),
    PointerAct::Up(Button::Right, SPOT),
];
const LET_GO: PointerAct = PointerAct::Up(Button::Left, SPOT);

const LEFT: PointerEvent = PointerEvent::Down {
    button: Button::Left,
    at: SPOT,
    ours: false,
    dpi: 96,
};
const MOVE: PointerEvent = PointerEvent::Move { at: SPOT };

/// One call the probe engine took.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Call {
    On(PointerEvent),
    Settle(Mono),
    Release,
}

/// An engine that drops left downs, marks each event and wants `settle` at its time, right-clicks at `settle` when told,
/// and lets go at release.
struct Probe {
    calls: Arc<Mutex<Vec<Call>>>,
    click: Arc<AtomicBool>,
    due: Option<Mono>,
}

impl PointerEngine for Probe {
    fn on(&mut self, event: PointerEvent, now: Mono, acts: &mut Vec<PointerAct>) -> bool {
        self.calls.lock().unwrap().push(Call::On(event));
        self.due = Some(now);
        acts.push(SEEN);
        matches!(
            event,
            PointerEvent::Down {
                button: Button::Left,
                ..
            }
        )
    }

    fn settle(&mut self, now: Mono, acts: &mut Vec<PointerAct>) {
        self.calls.lock().unwrap().push(Call::Settle(now));
        if self.click.swap(false, Ordering::SeqCst) {
            acts.extend(CLICK);
        }
    }

    fn deadline(&self) -> Option<Mono> {
        self.due
    }

    fn has_notes(&self) -> bool {
        false
    }

    fn release(&mut self, acts: &mut Vec<PointerAct>) {
        self.calls.lock().unwrap().push(Call::Release);
        acts.push(LET_GO);
    }

    fn max_acts(&self) -> usize {
        CLICK.len()
    }
}

/// A probe engine, its call log and its right-click command flag.
fn probe() -> (Box<Probe>, Arc<Mutex<Vec<Call>>>, Arc<AtomicBool>) {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let click = Arc::new(AtomicBool::new(false));
    let engine = Probe {
        calls: Arc::clone(&calls),
        click: Arc::clone(&click),
        due: None,
    };
    (Box::new(engine), calls, click)
}

/// An engine that pushes one act more than it declares.
struct Greedy;

impl PointerEngine for Greedy {
    fn on(&mut self, _: PointerEvent, _: Mono, acts: &mut Vec<PointerAct>) -> bool {
        acts.extend([SEEN; 2]);
        false
    }

    fn settle(&mut self, _: Mono, _: &mut Vec<PointerAct>) {}

    fn deadline(&self) -> Option<Mono> {
        None
    }

    fn has_notes(&self) -> bool {
        false
    }

    fn release(&mut self, _: &mut Vec<PointerAct>) {}

    fn max_acts(&self) -> usize {
        1
    }
}

/// The message `run` panics with.
fn panic_text(run: impl FnOnce()) -> String {
    let hit = catch_unwind(AssertUnwindSafe(run)).expect_err("it should panic");
    let text = hit.downcast_ref::<String>().cloned();
    text.or_else(|| hit.downcast_ref::<&str>().map(|s| (*s).to_owned()))
        .unwrap_or_default()
}

#[test]
fn events_reach_on_in_order_and_its_acts_go_before_settles() {
    let hook = FakePointerHook::new();
    let (engine, calls, click) = probe();
    let _run = hook.start(engine, CONFIG).unwrap();
    click.store(true, Ordering::SeqCst);
    assert!(hook.feed(LEFT, T1), "a left down drops");
    assert!(!hook.feed(MOVE, T2), "a move passes");
    let want = [
        Call::On(LEFT),
        Call::Settle(T1),
        Call::On(MOVE),
        Call::Settle(T2),
    ];
    assert_eq!(*calls.lock().unwrap(), want);
    assert_eq!(hook.sent(), [SEEN, CLICK[0], CLICK[1], SEEN]);
    assert_eq!(hook.deadline(), Some(T2));
}

#[test]
fn a_wake_never_calls_the_engine_but_settle_does() {
    let hook = FakePointerHook::new();
    let (engine, calls, click) = probe();
    let run = hook.start(engine, CONFIG).unwrap();
    click.store(true, Ordering::SeqCst);
    run.wake();
    run.wake();
    assert_eq!(hook.wakes(), 2);
    assert!(
        calls.lock().unwrap().is_empty(),
        "a wake alone calls nothing"
    );
    hook.settle(T1);
    assert_eq!(*calls.lock().unwrap(), [Call::Settle(T1)]);
    assert_eq!(hook.take_sent(), CLICK);
    assert!(hook.sent().is_empty(), "taken");
}

#[test]
fn a_dropped_run_releases_once_and_the_hook_starts_again() {
    let hook = FakePointerHook::new();
    let (engine, calls, _) = probe();
    let run = hook.start(engine, CONFIG).unwrap();
    assert!(hook.running());
    drop(run);
    assert_eq!(*calls.lock().unwrap(), [Call::Release]);
    assert_eq!(hook.sent(), [LET_GO]);
    assert!(!hook.running());
    assert_eq!((hook.starts(), hook.stops()), (1, 1));
    let _again = hook.start(probe().0, CONFIG).unwrap();
    assert_eq!((hook.starts(), hook.running()), (2, true));
}

#[test]
fn a_second_start_is_busy_and_fail_next_refuses_exactly_one() {
    let hook = FakePointerHook::new();
    let run = hook.start(probe().0, CONFIG).unwrap();
    let (late, late_calls, _) = probe();
    assert_eq!(hook.start(late, CONFIG).err(), Some(HookError::Busy));
    drop(run);
    hook.fail_next(HookError::Refused(5));
    let refused = hook.start(probe().0, CONFIG).err();
    assert_eq!(refused, Some(HookError::Refused(5)));
    assert!(hook.start(probe().0, CONFIG).is_ok(), "only one refusal");
    assert_eq!(hook.starts(), 2, "refusals do not count");
    assert!(late_calls.lock().unwrap().is_empty(), "never ran");
}

#[test]
fn config_and_stats_are_what_the_test_set() {
    let hook = FakePointerHook::new();
    assert_eq!(hook.config(), None);
    let run = hook.start(probe().0, CONFIG).unwrap();
    assert_eq!(hook.config(), Some(CONFIG));
    assert_eq!(run.stats(), HookStats::default());
    let stats = HookStats {
        events: 7,
        reinstalls: 1,
        ..HookStats::default()
    };
    hook.set_stats(stats);
    assert_eq!(run.stats(), stats);
}

#[test]
fn an_engine_that_pushes_past_max_acts_panics() {
    let hook = FakePointerHook::new();
    let run = hook.start(Box::new(Greedy), CONFIG).unwrap();
    let text = panic_text(|| {
        let _ = hook.feed(MOVE, T1);
    });
    assert!(text.contains(OVERRUN), "{text}");
    drop(run);
    assert_eq!(hook.stops(), 1, "the run still stops");
}

#[test]
fn a_call_with_no_live_engine_panics() {
    let hook = FakePointerHook::new();
    let text = panic_text(|| hook.settle(T1));
    assert!(text.contains(NO_RUN), "{text}");
}
