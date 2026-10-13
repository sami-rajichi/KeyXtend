//! The latched set on the bus, for modifier + click: told on each change in its fixed order and on `LatchesAsk`, and
//! let go on `LatchesUsed` as the `modifiers` mode says, from any thread.

use super::{Bench, Said, scans};
use crate::rig::STATE;
use kx_module_api::ports::{FaceAction, FaceReply, LatchesAsk, LatchesChanged, LatchesUsed};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

const Q: &str = "c-0x0010";

/// The bench in `stay` mode.
fn stay() -> Bench {
    Bench::quiet(Some(("modifiers", "stay".into())))
}

#[test]
fn latching_tells_the_set_in_its_fixed_order_and_letting_go_tells_it_empty() {
    let b = Bench::quiet(None);
    b.rig.tap("shift").unwrap();
    assert_eq!(b.said(), [Said::Latches(scans(&[0x2A]))]);
    b.rig.tap("shift").unwrap();
    assert_eq!(b.said(), [Said::Latches(Vec::new())]);
    for id in ["ctrl", "rshift", "rshift", "ctrl"] {
        b.rig.tap(id).unwrap();
    }
    let told: [&[u16]; 4] = [&[0x1D], &[0x36, 0x1D], &[0x1D], &[]];
    let told = told.map(|t| Said::Latches(scans(t)));
    assert_eq!(
        b.said(),
        told,
        "Shift comes before Ctrl, whatever the order"
    );
}

#[test]
fn a_typed_key_in_release_mode_tells_the_set_let_go_and_in_stay_mode_tells_nothing() {
    let b = Bench::quiet(None);
    b.rig.tap("shift").unwrap();
    b.said();
    b.rig.tap(Q).unwrap();
    assert_eq!(b.said(), [Said::Latches(Vec::new())]);
    let b = stay();
    b.rig.tap("shift").unwrap();
    b.said();
    b.rig.tap(Q).unwrap();
    assert!(b.said().is_empty());
}

#[test]
fn a_held_key_tells_the_set_let_go_only_when_its_burst_ends() {
    let b = Bench::quiet(None);
    b.rig.tap("shift").unwrap();
    b.said();
    b.rig.press("bksp").unwrap();
    assert!(
        b.said().is_empty(),
        "the latches stay down while it repeats"
    );
    b.rig.let_go("bksp").unwrap();
    assert_eq!(b.said(), [Said::Latches(Vec::new())]);
}

#[test]
fn latches_ask_tells_the_set_again() {
    let b = Bench::quiet(None);
    b.send(&LatchesAsk);
    assert_eq!(b.said(), [Said::Latches(Vec::new())]);
    b.rig.tap("altgr").unwrap();
    b.said();
    b.send(&LatchesAsk);
    assert_eq!(b.said(), [Said::Latches(scans(&[0xE038]))]);
}

#[test]
fn latches_used_in_release_mode_lets_go_tells_it_and_redraws_at_the_next_relabel() {
    let b = Bench::quiet(None);
    b.rig.tap("shift").unwrap();
    b.rig.tap("ctrl").unwrap();
    b.rig.state();
    b.said();
    b.send(&LatchesUsed);
    assert_eq!(b.said(), [Said::Latches(Vec::new())]);
    assert_eq!(b.rig.act(FaceAction::Relabel), Ok(STATE));
    assert_eq!(b.key("shift")["on"], false);
    assert_eq!(b.key("ctrl")["on"], false);
    b.send(&LatchesUsed);
    assert!(b.said().is_empty(), "nothing left to let go");
}

#[test]
fn latches_used_in_stay_mode_keeps_them_and_tells_nothing() {
    let b = stay();
    b.rig.tap("shift").unwrap();
    b.rig.state();
    b.said();
    b.send(&LatchesUsed);
    assert!(b.said().is_empty());
    assert_eq!(b.rig.act(FaceAction::Relabel), Ok(FaceReply::default()));
    assert_eq!(b.key("shift")["on"], true);
}

#[test]
fn a_listener_that_asks_inside_its_handler_hears_the_set_again_with_no_deadlock() {
    let b = Bench::quiet(None);
    let asked = AtomicBool::new(false);
    let bus = b.bus.clone();
    let ask = move |_: &LatchesChanged| {
        if !asked.swap(true, Ordering::SeqCst) {
            bus.publish(&LatchesAsk).unwrap();
        }
    };
    let _ear = b.bus.subscribe(ask).unwrap();
    b.rig.tap("win").unwrap();
    let win = Said::Latches(scans(&[0xE05B]));
    assert_eq!(b.said(), [win.clone(), win]);
}

#[test]
fn the_last_set_told_is_the_set_held_whatever_thread_asks_or_uses() {
    let b = Bench::quiet(None);
    thread::scope(|s| {
        for _ in 0..4 {
            let bus = b.bus.clone();
            s.spawn(move || {
                for i in 0..200 {
                    if i % 2 == 0 {
                        bus.publish(&LatchesAsk).unwrap();
                    } else {
                        bus.publish(&LatchesUsed).unwrap();
                    }
                }
            });
        }
        for _ in 0..200 {
            b.rig.tap("shift").unwrap();
        }
    });
    let held = b.key("shift")["on"] == true;
    let last = b.said().pop().unwrap();
    let want = if held { scans(&[0x2A]) } else { Vec::new() };
    assert_eq!(last, Said::Latches(want));
}
