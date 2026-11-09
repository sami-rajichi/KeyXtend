//! Tests of Num Lock's put-back: only into our own terminal in front, to the first state, also on a panic.

use std::cell::{Cell, RefCell};
use std::panic::{AssertUnwindSafe, catch_unwind};

use super::*;
use crate::apps::AppKind;
use crate::ownkeys::fake::{Fake, hwnd};

/// The gate's terminal as the tests open it: window `hwnd(1)`, no tab to check.
fn terminal() -> Opened {
    Opened {
        kind: AppKind::Terminal,
        hwnd: hwnd(1),
        child: None,
        file: None,
        title_has: None,
    }
}

/// Num Lock put back to on, from `reads` in turn, on a desk with window `front` in front; the result and clicks.
fn back(front: usize, reads: &[Result<bool, String>]) -> (Result<(), String>, usize) {
    let desk = Fake::new(hwnd(front), &["Windows PowerShell"]);
    let reads = RefCell::new(reads.to_vec());
    let on = || reads.borrow_mut().remove(0);
    let got = back_on(&desk, &terminal(), (true, 0), &on, &|| Ok(()));
    (got, desk.clicks.get())
}

#[test]
fn num_lock_is_tapped_back_only_into_our_own_terminal_in_front() {
    assert_eq!(back(1, &[Ok(false), Ok(true)]), (Ok(()), 1));
    assert_eq!(back(1, &[Ok(true), Ok(true)]), (Ok(()), 0), "already back");
    let (got, clicks) = back(2, &[Ok(false), Ok(false)]);
    assert!(
        clicks == 0 && got.is_err_and(|e| e.contains("Num Lock not put back")),
        "another window in front"
    );
    let (got, clicks) = back(1, &[Err("unread".into())]);
    assert!(clicks == 0 && got.is_err(), "Num Lock unread");
    let (got, _) = back(1, &[Ok(false), Ok(false)]);
    assert!(got.is_err_and(|e| e.contains("stayed changed")));
}

#[test]
fn every_put_back_error_says_which_state_to_restore_and_how() {
    for reads in [&[Err("unread".into())][..], &[Ok(false), Ok(false)]] {
        let (got, _) = back(1, reads);
        let e = got.expect_err("not back");
        assert!(e.ends_with(&restore(true)), "{e}");
    }
    let (got, _) = back(2, &[Ok(false)]);
    assert!(
        got.is_err_and(|e| e.ends_with(&restore(true))),
        "not in front"
    );
    assert_eq!(
        restore(true),
        "Num Lock was on before the run; turn it back on with the on-screen keyboard"
    );
    assert!(restore(false).contains("was off") && restore(false).contains("back off"));
}

#[test]
fn the_second_terminal_keeps_the_state_the_first_found() {
    let read = Cell::new(0);
    let now = || {
        read.set(read.get() + 1);
        Ok(false)
    };
    assert_eq!(first(Some(true), now), Ok(true), "carried, though now off");
    assert_eq!(read.get(), 0, "never read again");
    assert_eq!(first(None, now), Ok(false));
    assert_eq!(read.get(), 1);
}

#[test]
fn num_lock_goes_back_once_after_the_drive_even_when_it_panics() {
    let backs = Cell::new(0);
    let back = || {
        backs.set(backs.get() + 1);
        Err("Num Lock not put back".to_string())
    };
    assert_eq!(
        then_back(|| 7, back),
        (7, Err("Num Lock not put back".into()))
    );
    assert_eq!(backs.get(), 1, "once after a drive");
    let panicked = catch_unwind(AssertUnwindSafe(|| {
        then_back(|| -> u8 { panic!("a step broke") }, back)
    }));
    assert!(panicked.is_err());
    assert_eq!(backs.get(), 2, "once after a panic");
}

#[test]
fn a_terminal_that_left_num_lock_changed_fails() {
    let v = finished(json!({ "ok": true }), Ok(()));
    assert_eq!(
        (v["ok"].as_bool(), v["numlock_back"].as_bool()),
        (Some(true), Some(true))
    );
    let v = finished(json!({ "ok": true }), Err("why".into()));
    assert_eq!(
        (&v["ok"], &v["numlock_error"]),
        (&json!(false), &json!("why"))
    );
    assert_eq!(finished(json!({ "ok": false }), Ok(()))["ok"], false);
}

#[test]
fn a_put_back_error_reaches_the_clean_up_even_when_a_later_step_fails() {
    let mut left = Vec::new();
    let run = |left: &mut Vec<String>| -> Result<Value, String> {
        let v = settled(Ok(json!({ "ok": true })), Err("not back".into()), left)?;
        assert_eq!(v["numlock_error"], "not back");
        Err("the console failed".into())
    };
    assert!(run(&mut left).is_err());
    assert_eq!(left, ["not back"], "kept past the failed step");
    let mut left = Vec::new();
    assert!(settled(Err("drive".into()), Err("not back".into()), &mut left).is_err());
    assert_eq!(left, ["not back"]);
    let mut left = Vec::new();
    let v = settled(Ok(json!({ "ok": true })), Ok(()), &mut left);
    assert!(v.is_ok_and(|v| v["ok"] == true) && left.is_empty());
}
