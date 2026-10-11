//! Every stop of a held key's repeats (spec §4.6): the let-go, a leave, a stale table, a failed send, a restart and
//! the list opening; after each, nothing more types and no latch outlives the burst.

#![allow(
    clippy::unwrap_used,
    reason = "these fixtures run only in tests, where a failed setup should fail the test"
)]

use crate::held::{A, hold, text, times};
use crate::host::{Fakes, GOLDEN, id, table};
use crate::rig::{Rig, STATE};
use kx_module_api::keys;
use kx_module_api::ports::{FaceError, FaceReply, InjectError, Scan};
use kx_platform_fake::{FakeLayouts, Stroke};
use std::sync::Arc;

/// Asserts that `beats` more ticks on `r` type nothing and ask for nothing.
fn still(r: &Rig, beats: u64) {
    let (_, every) = times();
    for beat in 0..beats {
        assert_eq!(r.tick_in(every), FaceReply::default(), "beat {beat}");
    }
}

#[test]
fn a_leave_stops_the_burst_and_lets_the_latches_go() {
    let r = Rig::new(&GOLDEN, None);
    r.tap("shift").unwrap();
    hold(&r, A, 1);
    assert_eq!(r.leave(), STATE);
    assert_eq!(r.state()["keys"]["shift"]["on"], false);
    still(&r, 3);
    assert_eq!(r.let_go(A), Ok(FaceReply::default()), "a let-go after it");
    assert_eq!(r.fakes.injector.strokes(), vec![text("A"); 3]);
}

#[test]
fn a_leave_before_the_delay_keeps_only_what_the_press_typed() {
    let r = Rig::new(&GOLDEN, None);
    let (delay, _) = times();
    r.press(A).unwrap();
    r.tick_in(delay / 2);
    assert_eq!(r.leave(), STATE);
    r.tick_in(delay);
    assert_eq!(r.let_go(A), Ok(FaceReply::default()));
    assert_eq!(r.fakes.injector.strokes(), [text("a")]);
}

#[test]
fn a_tick_or_leave_with_nothing_held_does_nothing() {
    let r = Rig::new(&GOLDEN, None);
    still(&r, 3);
    assert_eq!(r.leave(), FaceReply::default());
    assert!(r.fakes.injector.strokes().is_empty());
}

#[test]
fn a_press_on_a_stale_table_types_nothing_and_never_repeats() {
    // After a dead key the next key goes as a scan code, so the app's own layout must match the one shown.
    let r = Rig::new(&["fr-fr", "en-us"], None);
    r.tap("c-0x001A").unwrap();
    r.fakes.source.set_foreground(Some(id("en-us")));
    r.fakes.layouts.switch(id("en-us"));
    let e = "c-0x0012";
    assert_eq!(r.press(e), Ok(STATE), "it sends nothing and redraws");
    assert_eq!(r.fakes.layouts.refreshes(), 1);
    r.tick_in(times().0);
    still(&r, 3);
    assert_eq!(r.let_go(e), Ok(FaceReply::default()));
    assert_eq!(r.fakes.injector.strokes(), [Stroke::Tap(Scan::new(0x1A))]);
}

/// The notice for keys not sent, with the state redrawn.
const WARNED: FaceReply = FaceReply {
    notice: Some(keys::KEYS_NOT_SENT),
    ..STATE
};

#[test]
fn a_failed_send_stops_the_burst_with_one_notice() {
    let r = Rig::new(&GOLDEN, None);
    let (_, every) = times();
    hold(&r, A, 0);
    let refused = InjectError::Refused { sent: 0, total: 1 };
    r.fakes.injector.set_error(Some(refused));
    assert_eq!(r.tick_in(every), WARNED);
    r.fakes.injector.set_error(None);
    still(&r, 3);
    assert_eq!(r.let_go(A), Ok(FaceReply::default()));
    assert_eq!(r.fakes.injector.strokes(), [text("a"), text("a")]);
}

#[test]
fn a_press_not_sent_says_so_and_never_repeats() {
    let r = Rig::new(&GOLDEN, None);
    r.tap("shift").unwrap();
    let refused = InjectError::Refused { sent: 0, total: 1 };
    r.fakes.injector.set_error(Some(refused));
    assert_eq!(r.press(A), Ok(WARNED));
    r.fakes.injector.set_error(None);
    r.tick_in(times().0);
    still(&r, 3);
    assert_eq!(r.let_go(A), Ok(FaceReply::default()));
    assert!(r.fakes.injector.strokes().is_empty());
    assert_eq!(r.state()["keys"]["shift"]["on"], true, "nothing went");
}

#[test]
fn a_repeat_sent_in_part_stops_the_burst_and_lets_shift_go() {
    let r = Rig::new(&GOLDEN, None);
    let (delay, _) = times();
    r.tap("shift").unwrap();
    r.press(A).unwrap();
    let part = InjectError::Refused { sent: 1, total: 2 };
    r.fakes.injector.set_error(Some(part));
    assert_eq!(r.tick_in(delay), WARNED, "one notice, and the beat stops");
    r.fakes.injector.set_error(None);
    assert_eq!(r.state()["keys"]["shift"]["on"], false, "part of it went");
    still(&r, 3);
    assert_eq!(r.let_go(A), Ok(FaceReply::default()));
}

#[test]
fn a_press_sent_in_part_never_repeats_and_lets_shift_go() {
    let r = Rig::new(&GOLDEN, None);
    r.tap("shift").unwrap();
    let part = InjectError::Refused { sent: 1, total: 2 };
    r.fakes.injector.set_error(Some(part));
    assert_eq!(r.press(A), Ok(WARNED));
    r.fakes.injector.set_error(None);
    assert_eq!(r.state()["keys"]["shift"]["on"], false, "part of it went");
    r.tick_in(times().0);
    still(&r, 3);
    assert_eq!(r.let_go(A), Ok(FaceReply::default()));
}

#[test]
fn a_repeat_on_a_stale_table_sends_nothing_refreshes_once_and_ends_the_burst() {
    // A key the shown table lacks goes as its scan code, so the app's own layout must match the one shown.
    let mut fakes = Fakes::golden(&["fr-fr", "en-us"]);
    let mut fr = table("fr-fr");
    fr.keys.remove(&Scan::new(0x56));
    fakes.layouts = Arc::new(FakeLayouts::new(vec![fr, table("en-us")]));
    let r = Rig::on(fakes, None);
    r.tap("shift").unwrap();
    let iso = "c-0x0056";
    hold(&r, iso, 0);
    let typed = r.fakes.injector.strokes();
    assert_eq!(typed.len(), 2, "the press and the first repeat");
    r.fakes.source.set_foreground(Some(id("en-us")));
    r.fakes.layouts.switch(id("en-us"));
    let refreshed = r.fakes.layouts.refreshes();
    assert_eq!(r.tick_in(times().1), STATE, "it sends nothing and redraws");
    assert_eq!(r.fakes.layouts.refreshes(), refreshed + 1, "one refresh");
    still(&r, 3);
    assert_eq!(r.let_go(iso), Ok(FaceReply::default()));
    assert_eq!(r.fakes.injector.strokes(), typed, "nothing more types");
    assert_eq!(
        r.state()["keys"]["shift"]["on"],
        false,
        "the press used Shift, so it lets go with the burst"
    );
}

#[test]
fn a_stale_press_keeps_shift_for_the_next_key() {
    let r = Rig::new(&["fr-fr", "en-us"], None);
    r.tap("c-0x001A").unwrap();
    r.tap("shift").unwrap();
    r.fakes.source.set_foreground(Some(id("en-us")));
    r.fakes.layouts.switch(id("en-us"));
    let e = "c-0x0012";
    assert_eq!(r.press(e), Ok(STATE), "it sends nothing and redraws");
    r.tick_in(times().0);
    still(&r, 3);
    assert_eq!(r.let_go(e), Ok(FaceReply::default()));
    assert_eq!(
        r.state()["keys"]["shift"]["on"],
        true,
        "nothing went, so Shift waits"
    );
    assert_eq!(r.fakes.injector.strokes(), [Stroke::Tap(Scan::new(0x1A))]);
}

#[test]
fn a_restart_drops_the_held_key_and_ends_its_burst() {
    let mut r = Rig::new(&GOLDEN, None);
    r.tap("shift").unwrap();
    hold(&r, A, 0);
    r.restart(None);
    still(&r, 3);
    assert_eq!(r.state()["keys"]["shift"]["on"], false, "the burst ended");
    assert_eq!(r.let_go(A), Ok(FaceReply::default()));
    assert_eq!(r.fakes.injector.strokes(), vec![text("A"); 2]);
}

#[test]
fn opening_the_list_stops_the_burst() {
    // A second input, such as a reader's Invoke on the language key, opens the list while an arrow repeats.
    let r = Rig::new(&GOLDEN, Some(("lang_key", "list".into())));
    hold(&r, "down", 0);
    let typed = r.fakes.injector.strokes();
    assert_eq!(typed.len(), 2, "the press and the first repeat");
    assert_eq!(r.tap("lang"), Ok(STATE));
    still(&r, 3);
    assert_eq!(r.let_go("down"), Ok(FaceReply::default()));
    assert_eq!(r.fakes.injector.strokes(), typed, "nothing reached the app");
}

#[test]
fn a_click_ends_a_burst_left_open_so_its_latches_never_ride_on() {
    // No leave came, as after a window rebuilt for a lost graphics device, or a reader's Invoke mid-burst.
    let r = Rig::new(&GOLDEN, None);
    r.tap("shift").unwrap();
    hold(&r, A, 1);
    assert_eq!(r.tap("tab"), Ok(STATE));
    let strokes = r.fakes.injector.strokes();
    assert_eq!(
        strokes.last(),
        Some(&Stroke::Tap(Scan::new(0x0F))),
        "a plain Tab"
    );
    assert_eq!(r.state()["keys"]["shift"]["on"], false);
    still(&r, 3);
}

#[test]
fn a_key_not_on_the_board_is_refused_and_changes_nothing() {
    let (_, every) = times();
    for bad in [Rig::press, Rig::let_go] {
        let r = Rig::new(&GOLDEN, None);
        r.tap("shift").unwrap();
        hold(&r, A, 0);
        assert_eq!(bad(&r, "nope"), Err(FaceError::UnknownKey));
        assert_eq!(
            r.state()["keys"]["shift"]["on"],
            true,
            "no change the UI is not told of"
        );
        r.tick_in(every);
        assert_eq!(
            r.fakes.injector.strokes(),
            vec![text("A"); 3],
            "the hold goes on"
        );
        assert_eq!(r.leave(), STATE, "its own end redraws");
    }
}
