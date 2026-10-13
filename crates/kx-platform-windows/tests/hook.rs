//! The real hooks go in, let go on `release_now` and stay, then come out; only the gate tool's simulated input reaches
//! the engine, so a hand on the mouse cannot count and nothing moves. One test per binary, as one hook runs per
//! process, and only with the `sim-input` feature.
#![cfg(all(windows, feature = "sim-input"))]
#![allow(clippy::unwrap_used, reason = "a failed setup should fail the test")]

use std::sync::mpsc::{self, Sender};
use std::time::Duration;

use kx_module_api::Mono;
use kx_module_api::ports::{HookConfig, HookError, PointerAct, PointerEngine, PointerEvent};
use kx_platform_windows::{WindowsPlatform, release_now};

/// A check every second with half a second of slack, and a quarter second for an echo: the module's defaults.
const CFG: HookConfig = HookConfig {
    health_ms: 1_000,
    health_slack_ms: 500,
    echo_ms: 250,
};
/// How long the test waits for a release; far past `RELEASE_WAIT`, so a stalled runner cannot fail it.
const WAIT: Duration = Duration::from_secs(10);

/// An engine that never acts and says each release.
struct Quiet(Sender<()>);

impl PointerEngine for Quiet {
    fn on(&mut self, _: PointerEvent, _: Mono, _: &mut Vec<PointerAct>) -> bool {
        false
    }

    fn settle(&mut self, _: Mono, _: &mut Vec<PointerAct>) {}

    fn deadline(&self) -> Option<Mono> {
        None
    }

    fn release(&mut self, _: &mut Vec<PointerAct>) {
        let _ = self.0.send(());
    }

    fn max_acts(&self) -> usize {
        1
    }
}

#[test]
fn the_hooks_go_in_let_go_on_release_now_and_stay_until_the_drop_lets_go_again() {
    let hook = WindowsPlatform::new().unwrap().sim_pointer_hook();
    let (said, released) = mpsc::channel();
    let run = hook.start(Box::new(Quiet(said)), CFG).unwrap();
    let again = hook.start(Box::new(Quiet(mpsc::channel().0)), CFG);
    assert!(matches!(again, Err(HookError::Busy)), "one run per process");
    run.wake();
    assert_eq!(run.stats().events, 0);
    release_now();
    assert!(released.recv_timeout(WAIT).is_ok(), "release_now let go");
    let still = hook.start(Box::new(Quiet(mpsc::channel().0)), CFG);
    assert!(matches!(still, Err(HookError::Busy)), "the run goes on");
    drop(run);
    assert!(released.recv_timeout(WAIT).is_ok(), "the drop let go again");
    assert!(released.try_recv().is_err(), "and nothing more");
}
