//! Tests of a run's mouse guard, through a fake hook that applies the filter on the fake clock; no real hook.

use std::panic::{AssertUnwindSafe, catch_unwind};

use serde_json::json;
use windows::Win32::Foundation::HWND;

use super::*;
use crate::keymap::Reader;
use crate::mouseguard::fake::{Fake, begun, cfg, held, slot_lock};

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
fn a_button_held_for_the_escape_time_stops_the_gate_as_the_owners() {
    let fake = Fake::new();
    let mut s = begun(&fake);
    fake.wait(1_000);
    fake.press();
    fake.wait(cfg().escape_ms);
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
    assert!(e.contains(End::Lost.why()), "{e}");
    assert_eq!(s.finish(true).end, End::Lost);
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
fn a_stopped_gate_is_refused_a_press_and_a_prompt_but_a_restore_still_goes_out() {
    let _slot = slot_lock();
    let fake = Fake::new();
    let scope = held(&fake);
    assert_eq!(around_prompt(|| Ok(7)), Ok(7), "the prompt ran");
    assert_eq!(fake.counts(), (2, 1), "let go for it, then back");
    fake.wait(cfg().limit_ms);
    assert!(allowed().is_err());
    let mut shown = false;
    let prompt = around_prompt(|| {
        shown = true;
        Ok(())
    });
    assert!(prompt.is_err() && !shown, "no prompt once stopped");
    // An empty id fails at the lookup, so neither call can press anything; only the press is refused before it.
    let reader = Reader::new().expect("a UI Automation client");
    let press = reader.invoke(HWND::default(), "").expect_err("refused");
    assert!(press.contains(End::Limit.why()), "{press}");
    let back = reader
        .restore(HWND::default(), "")
        .expect_err("no such key");
    assert!(
        !back.contains(End::Limit.why()),
        "a restore is never refused: {back}"
    );
    assert_eq!(scope.finish(true).map(|o| o.end), Some(End::Limit));
    assert_eq!(allowed(), Ok(()), "the slot is empty again");
}

#[test]
fn letting_go_after_the_pointer_work_frees_the_mouse_through_the_slot() {
    let _slot = slot_lock();
    let fake = Fake::new();
    let scope = held(&fake);
    let_go();
    assert_eq!(fake.counts(), (1, 1));
    assert_eq!(around_prompt(|| Ok(())), Ok(()));
    assert_eq!(fake.counts(), (1, 1), "it stays let go");
    assert_eq!(scope.finish(true).map(|o| o.end), Some(End::Done));
}

#[test]
fn a_guard_switched_off_or_a_free_gate_starts_no_hook_and_refuses_nothing() {
    let _slot = slot_lock();
    // Only begins that start no hook are tried here, so no test can ever install a real one.
    let off = MouseGuard { on: false, ..cfg() };
    assert_eq!(begin(&off, true).expect("no hook").finish(true), None);
    assert_eq!(begin(&cfg(), false).expect("no hook").finish(true), None);
    assert_eq!(allowed(), Ok(()), "no guard, nothing refused");
    assert_eq!(around_prompt(|| Ok(1)), Ok(1), "the prompt runs once");
}

/// An outcome that ended with `end` after counting `dropped` and `passed`.
fn outcome(end: End, dropped: u32, passed: u32) -> Outcome {
    let counts = Counts { dropped, passed };
    Outcome { end, counts }
}

#[test]
fn the_guard_adds_up_what_each_of_its_hooks_counted_and_gives_it_when_the_gate_ends() {
    let fake = Fake::new();
    let mut s = begun(&fake);
    fake.press();
    fake.mine();
    s.pause();
    s.resume();
    fake.mine();
    fake.mine();
    assert_eq!(s.finish(true), outcome(End::Done, 1, 3));
}

#[test]
fn a_stopped_run_fails_and_says_why_and_a_guarded_run_says_how_it_ended() {
    let mut owner = json!({ "gate": "G8", "pass": true });
    mark(&mut owner, Some(outcome(End::Owner, 0, 0)));
    assert_eq!(owner["pass"], json!(false));
    assert_eq!(owner["mouse_guard"]["ended"], json!("owner"));
    assert_eq!(owner["mouse_guard"]["why"], json!(End::Owner.why()));
    let mut done = json!({ "gate": "G2", "pass": "partial" });
    mark(&mut done, Some(outcome(End::Done, 0, 0)));
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

#[test]
fn a_guarded_run_shows_the_counts_that_prove_its_guard_was_live_and_nothing_more() {
    let mut run = json!({ "gate": "G2", "pass": true });
    mark(&mut run, Some(outcome(End::Done, 41, 600)));
    let guard = &run["mouse_guard"];
    assert_eq!(
        (guard["dropped"].clone(), guard["passed"].clone()),
        (json!(41), json!(600))
    );
    let mut keys: Vec<_> = guard.as_object().expect("an object").keys().collect();
    keys.sort();
    assert_eq!(
        keys,
        ["dropped", "ended", "passed", "why"],
        "numbers only, no positions"
    );
}
