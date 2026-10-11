//! Tests of a run's mouse guard, through a fake hook that applies the filter on the fake clock; no real hook.

use std::panic::{AssertUnwindSafe, catch_unwind};

use kx_platform_fake::FakeClock;

use super::*;
use crate::mouseguard::Loss;
use crate::mouseguard::fake::{Fake, begun, cfg};

/// Clicks kx-gates sends now, noted as `slot::own_input` notes them.
fn send_clicks(s: &mut Session<Fake, FakeClock>) {
    let base = s.own();
    s.sent(base);
}

#[test]
fn own_clicks_the_hook_counts_in_time_keep_the_gate_going() {
    let (fake, alive) = (Fake::new(), cfg().alive_ms);
    let mut s = begun(&fake);
    send_clicks(&mut s);
    fake.wait(alive - 1);
    fake.mine();
    fake.wait(alive * 10);
    assert_eq!(s.refusal(), Ok(()));
    assert_eq!(s.finish(true).end, End::Done);
}

#[test]
fn own_clicks_the_hook_misses_for_alive_ms_end_the_guard_as_lost_and_let_go_of_it() {
    let (fake, alive) = (Fake::new(), cfg().alive_ms);
    let mut s = begun(&fake);
    send_clicks(&mut s);
    fake.wait(alive - 1);
    assert_eq!(s.refusal(), Ok(()), "still in time");
    fake.wait(1);
    let e = s.refusal().expect_err("Windows switched the hook off");
    assert!(e.contains("Windows switched the mouse guard off"), "{e}");
    assert_eq!(fake.counts(), (1, 1), "the guard removed its hook");
    assert_eq!(s.finish(true).end, End::Lost(Loss::Off));
}

#[test]
fn a_lost_hook_is_caught_as_the_gate_ends_or_lets_go_after_its_pointer_work() {
    for idle in [false, true] {
        let (fake, alive) = (Fake::new(), cfg().alive_ms);
        let mut s = begun(&fake);
        send_clicks(&mut s);
        fake.wait(alive);
        if idle {
            s.release();
        }
        assert_eq!(s.finish(true).end, End::Lost(Loss::Off), "idle: {idle}");
    }
}

#[test]
fn clicks_sent_while_the_guard_let_go_or_around_a_prompt_are_never_judged() {
    let (fake, alive) = (Fake::new(), cfg().alive_ms);
    let mut s = begun(&fake);
    send_clicks(&mut s);
    s.pause();
    send_clicks(&mut s);
    fake.wait(alive * 2);
    s.resume();
    assert_eq!(s.refusal(), Ok(()), "the guard removed that hook itself");
    s.release();
    send_clicks(&mut s);
    fake.wait(alive * 2);
    assert_eq!(s.refusal(), Ok(()), "no hook after the pointer work");
    assert_eq!(s.finish(true).end, End::Done);
}

#[test]
fn the_guard_holds_the_mouse_from_the_start_and_lets_go_when_the_gate_ends() {
    let fake = Fake::new();
    let mut s = begun(&fake);
    assert_eq!(fake.counts(), (1, 0));
    assert_eq!(s.refusal(), Ok(()));
    assert_eq!(s.finish(true).end, End::Done);
    assert_eq!(fake.seen().stops, 1);
}

#[test]
fn a_gate_that_broke_off_lets_go_with_an_error() {
    let fake = Fake::new();
    assert_eq!(begun(&fake).finish(false).end, End::Error);
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
    assert_eq!(s.finish(true).end, End::Limit);
    assert_eq!(fake.seen().stops, 1);
}

#[test]
fn one_click_stops_the_gate_at_once_as_the_owners() {
    let fake = Fake::new();
    let mut s = begun(&fake);
    fake.wait(1_000);
    fake.moved();
    assert_eq!(s.refusal(), Ok(()), "a move is no escape");
    fake.press();
    let e = s.refusal().expect_err("the owner took the mouse back");
    assert!(e.contains(End::Owner.why()), "{e}");
    assert_eq!(s.finish(false).end, End::Owner, "the owner's stop wins");
}

#[test]
fn an_admin_prompt_lets_go_and_its_time_does_not_count_against_the_limit() {
    let (fake, limit) = (Fake::new(), cfg().limit_ms);
    let mut s = begun(&fake);
    fake.wait(limit / 2);
    s.pause();
    assert_eq!(fake.counts(), (1, 1));
    fake.wait(limit * 10);
    s.resume();
    assert_eq!(fake.counts(), (2, 1), "it came back");
    fake.wait(limit - limit / 2 - 1);
    assert_eq!(s.refusal(), Ok(()), "only held time counts");
    fake.wait(1);
    assert!(
        s.refusal().is_err(),
        "the mouse was held for the whole limit"
    );
    assert_eq!(s.finish(true).end, End::Limit);
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
    assert!(e.contains(End::Lost(Loss::Prompt).why()), "{e}");
    assert_eq!(s.finish(true).end, End::Lost(Loss::Prompt));
}

#[test]
fn a_guard_let_go_after_the_pointer_work_never_comes_back_and_the_gate_goes_on() {
    let fake = Fake::new();
    let mut s = begun(&fake);
    s.release();
    assert_eq!(fake.counts(), (1, 1));
    s.pause();
    s.resume();
    assert_eq!(fake.counts(), (1, 1), "no prompt brings it back");
    assert_eq!(s.refusal(), Ok(()));
    assert_eq!(s.finish(true).end, End::Done);
}

#[test]
fn the_guard_adds_up_what_each_of_its_hooks_counted_and_gives_it_when_the_gate_ends() {
    let fake = Fake::new();
    let mut s = begun(&fake);
    fake.moved();
    fake.mine();
    s.pause();
    s.resume();
    fake.mine();
    fake.mine();
    let counts = Counts {
        dropped: 1,
        passed: 3,
    };
    let end = End::Done;
    assert_eq!(s.finish(true), Outcome { end, counts });
}
