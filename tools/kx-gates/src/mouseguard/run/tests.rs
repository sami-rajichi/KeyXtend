//! Tests of a run's mouse guard, through a fake hook that applies the filter on the fake clock; no real hook.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use kx_module_api::{Clock, Mono};
use kx_platform_fake::FakeClock;
use serde_json::json;

use super::*;
use crate::mouseguard::{Kind, LEFT};

/// What the fake hooks saw: starts, stops, the last filter, and whether the next start fails.
#[derive(Default)]
struct Seen {
    starts: u32,
    stops: u32,
    filter: Option<Filter>,
    fail: bool,
}

/// A hook that lets go exactly when its filter says, as a hook thread with a timely check would.
#[derive(Clone)]
struct Fake {
    seen: Arc<Mutex<Seen>>,
    clock: FakeClock,
}

impl Fake {
    fn new() -> Self {
        Self {
            seen: Arc::default(),
            clock: FakeClock::new(Mono::from_us(0)),
        }
    }

    fn seen(&self) -> MutexGuard<'_, Seen> {
        self.seen.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Hooks started and stopped so far, read under one lock.
    fn counts(&self) -> (u32, u32) {
        let seen = self.seen();
        (seen.starts, seen.stops)
    }

    /// A physical press of the left button now, as the hook would see it.
    fn press(&self) {
        let now = self.clock.now();
        let mut seen = self.seen();
        let filter = seen.filter.as_mut().expect("a started hook");
        filter.event(false, Kind::Down(LEFT), now);
    }

    fn wait(&self, ms: u64) {
        self.clock.advance(Duration::from_millis(ms));
    }
}

impl Hook for Fake {
    type Live = Fake;

    fn start(&self, filter: Filter) -> Result<Fake, String> {
        let mut seen = self.seen();
        if seen.fail {
            return Err("no hook today".to_string());
        }
        seen.starts += 1;
        seen.filter = Some(filter);
        Ok(self.clone())
    }
}

impl Live for Fake {
    fn ended(&self) -> Option<End> {
        let now = self.clock.now();
        self.seen().filter.as_mut()?.tick(now)
    }

    fn stop(self) -> Option<End> {
        let own = self.ended();
        self.seen().stops += 1;
        own
    }
}

fn cfg() -> MouseGuard {
    crate::config::load()
        .expect("kx-gates.toml loads")
        .mouse_guard
}

fn begun(fake: &Fake) -> Session<Fake, FakeClock> {
    Session::begin(fake.clone(), fake.clock.clone(), &cfg()).expect("the fake hook starts")
}

#[test]
fn the_guard_holds_the_mouse_from_the_start_and_lets_go_when_the_gate_ends() {
    let fake = Fake::new();
    let mut s = begun(&fake);
    assert_eq!(fake.counts(), (1, 0));
    assert_eq!(s.refusal(), Ok(()));
    assert_eq!(s.finish(true), End::Done);
    assert_eq!(fake.seen().stops, 1);
}

#[test]
fn a_gate_that_broke_off_lets_go_with_an_error() {
    let fake = Fake::new();
    assert_eq!(begun(&fake).finish(false), End::Error);
    assert_eq!(fake.seen().stops, 1);
}

#[test]
fn dropping_the_guard_lets_go_so_a_panic_or_an_early_return_never_keeps_the_mouse() {
    let fake = Fake::new();
    drop(begun(&fake));
    assert_eq!(fake.seen().stops, 1);
    let panicked = catch_unwind(AssertUnwindSafe(|| {
        let _guard = begun(&fake);
        panic!("a gate panicked");
    }));
    assert!(panicked.is_err());
    assert_eq!(fake.counts(), (2, 2));
}

#[test]
fn the_guard_lets_go_at_its_time_limit_and_the_gate_then_stops() {
    let fake = Fake::new();
    let mut s = begun(&fake);
    fake.wait(cfg().limit_ms - 1);
    assert_eq!(s.refusal(), Ok(()));
    fake.wait(1);
    let e = s.refusal().expect_err("the limit ran out");
    assert!(e.contains(End::Limit.why()), "{e}");
    assert_eq!(s.finish(true), End::Limit);
    assert_eq!(fake.seen().stops, 1);
}

#[test]
fn a_button_held_for_the_escape_time_stops_the_gate_as_the_owners() {
    let fake = Fake::new();
    let mut s = begun(&fake);
    fake.wait(1_000);
    fake.press();
    fake.wait(cfg().escape_ms);
    let e = s.refusal().expect_err("the owner took the mouse back");
    assert!(e.contains(End::Owner.why()), "{e}");
    assert_eq!(s.finish(false), End::Owner, "the owner's stop wins");
}

#[test]
fn an_admin_prompt_lets_go_and_the_guard_comes_back_with_its_first_deadline() {
    let (fake, limit) = (Fake::new(), cfg().limit_ms);
    let mut s = begun(&fake);
    fake.wait(limit / 2);
    s.pause();
    assert_eq!(fake.counts(), (1, 1));
    s.resume();
    assert_eq!(fake.seen().starts, 2);
    fake.wait(limit - limit / 2);
    assert!(
        s.refusal().is_err(),
        "the limit counts from the first start"
    );
    assert_eq!(s.finish(true), End::Limit);
}

#[test]
fn a_prompt_that_outlasts_the_limit_leaves_the_guard_off_and_stops_the_gate() {
    let fake = Fake::new();
    let mut s = begun(&fake);
    s.pause();
    fake.wait(cfg().limit_ms);
    s.resume();
    assert_eq!(fake.seen().starts, 1, "it did not start again");
    assert!(s.refusal().is_err());
    assert_eq!(s.finish(true), End::Limit);
}

#[test]
fn a_guard_that_cannot_start_refuses_the_gate_or_stops_it_after_a_prompt() {
    let fake = Fake::new();
    fake.seen().fail = true;
    let refused = Session::begin(fake.clone(), fake.clock.clone(), &cfg());
    assert!(refused.is_err());
    fake.seen().fail = false;
    let mut s = begun(&fake);
    s.pause();
    fake.seen().fail = true;
    s.resume();
    let e = s.refusal().expect_err("it could not come back");
    assert!(e.contains(End::Lost.why()), "{e}");
    assert_eq!(s.finish(true), End::Lost);
}

#[test]
fn a_guard_switched_off_starts_no_hook_and_refuses_nothing() {
    // Only a switched-off guard is begun here, so no test can ever install a real hook.
    let off = MouseGuard { on: false, ..cfg() };
    assert_eq!(
        begin(&off, "g8").expect("no hook to start").finish(true),
        None
    );
    assert_eq!(allowed(), Ok(()), "no guard, nothing refused");
    let mut prompts = 0;
    assert_eq!(
        around_prompt(|| {
            prompts += 1;
            prompts
        }),
        1,
        "the prompt runs once"
    );
}

#[test]
fn a_stopped_run_fails_and_says_why_and_a_guarded_run_says_how_it_ended() {
    let mut owner = json!({ "gate": "G8", "pass": true });
    mark(&mut owner, Some(End::Owner));
    assert_eq!(owner["pass"], json!(false));
    assert_eq!(owner["mouse_guard"]["ended"], json!("owner"));
    assert_eq!(owner["mouse_guard"]["why"], json!(End::Owner.why()));
    let mut done = json!({ "gate": "G2", "pass": "partial" });
    mark(&mut done, Some(End::Done));
    assert_eq!(
        done["pass"],
        json!("partial"),
        "a gate that ended keeps its verdict"
    );
    assert_eq!(done["mouse_guard"]["ended"], json!("done"));
    let mut free = json!({ "gate": "G13", "pass": true });
    mark(&mut free, None);
    assert_eq!(free, json!({ "gate": "G13", "pass": true }));
}
