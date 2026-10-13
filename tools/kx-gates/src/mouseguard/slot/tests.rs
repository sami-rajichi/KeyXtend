//! Tests of this process's guard slot and the result line, through a fake hook; no real hook.

use serde_json::json;
use windows::Win32::Foundation::HWND;

use super::*;
use crate::keymap::Reader;
use crate::mouseguard::fake::{Fake, cfg, held, slot_lock};
use crate::mouseguard::{Counts, End, Loss, Mouse};

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
    for mouse in [Mouse::Held, Mouse::Drives] {
        assert_eq!(begin(&off, mouse).expect("no hook").finish(true), None);
    }
    assert_eq!(
        begin(&cfg(), Mouse::Free).expect("no hook").finish(true),
        None
    );
    assert_eq!(allowed(), Ok(()), "no guard, nothing refused");
    assert_eq!(around_prompt(|| Ok(1)), Ok(1), "the prompt runs once");
    assert_eq!(own_input(|| Ok(2)), Ok(2), "a send without a guard");
}

#[test]
fn a_hook_that_misses_own_clicks_refuses_the_next_press_but_not_a_restore_and_fails_the_run() {
    let _slot = slot_lock();
    let (fake, alive) = (Fake::new(), cfg().alive_ms);
    let scope = held(&fake);
    let counted = own_input(|| {
        fake.mine();
        Ok(1)
    });
    assert_eq!(counted, Ok(1));
    fake.wait(alive * 10);
    assert_eq!(allowed(), Ok(()), "the hook counted them");
    let failed = own_input(|| Err::<(), _>("SendInput failed".to_string()));
    assert!(failed.is_err());
    fake.wait(alive * 10);
    assert_eq!(allowed(), Ok(()), "nothing went out, so nothing is watched");
    assert_eq!(own_input(|| Ok(())), Ok(()), "sent, but never counted");
    fake.wait(alive);
    let e = allowed().expect_err("Windows switched the hook off");
    assert!(e.contains(End::Lost(Loss::Off).why()), "{e}");
    let reader = Reader::new().expect("a UI Automation client");
    let press = reader.invoke(HWND::default(), "").expect_err("refused");
    assert!(press.contains(End::Lost(Loss::Off).why()), "{press}");
    let back = reader.restore(HWND::default(), "").expect_err("no key");
    assert!(
        !back.contains(End::Lost(Loss::Off).why()),
        "never refused: {back}"
    );
    let mut result = json!({ "gate": "G2", "pass": true });
    mark(&mut result, scope.finish(true));
    assert_eq!(result["pass"], json!(false));
    assert_eq!(result["mouse_guard"]["ended"], json!("lost"));
}

/// An outcome that ended with `end` after counting `dropped` and `passed`.
fn outcome(end: End, dropped: u64, passed: u64) -> Outcome {
    let counts = Counts { dropped, passed };
    Outcome { end, counts }
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
