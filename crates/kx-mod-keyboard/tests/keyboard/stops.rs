//! Every stop of a held key's repeats (spec §4.6): leaving the key, hiding, a stale table, a failed send, a restart
//! and the list opening; after each, in any run of inputs, nothing more types and no latch stays down for the burst.

#![allow(
    clippy::unwrap_used,
    reason = "these fixtures run only in tests, where a failed setup should fail the test"
)]

use crate::held::{A, hold, text, times};
use crate::host::{GOLDEN, id};
use crate::rig::{Rig, STATE};
use kx_module_api::keys;
use kx_module_api::ports::{FaceError, FaceReply, InjectError, Scan};
use kx_platform_fake::Stroke;
use proptest::collection::vec;
use proptest::prelude::*;

/// Asserts that `beats` more ticks on `r` type nothing and ask for nothing.
fn still(r: &Rig, beats: u64) {
    let (_, every) = times();
    for beat in 0..beats {
        assert_eq!(r.tick_in(every), FaceReply::default(), "beat {beat}");
    }
}

#[test]
fn leaving_the_key_stops_the_burst_and_lets_the_latches_go() {
    let r = Rig::new(&GOLDEN, None);
    r.tap("shift").unwrap();
    hold(&r, A, 1);
    assert_eq!(r.leave(), STATE);
    assert_eq!(r.state()["keys"]["shift"]["on"], false);
    still(&r, 3);
    assert_eq!(r.let_go(A), Ok(FaceReply::default()), "let go outside");
    assert_eq!(r.fakes.injector.strokes(), [text("A"), text("A")]);
}

#[test]
fn leaving_before_the_delay_types_nothing() {
    let r = Rig::new(&GOLDEN, None);
    let (delay, _) = times();
    r.press(A).unwrap();
    r.tick_in(delay / 2);
    assert_eq!(r.leave(), FaceReply::default());
    r.tick_in(delay);
    assert_eq!(r.let_go(A), Ok(FaceReply::default()));
    assert!(r.fakes.injector.strokes().is_empty());
}

#[test]
fn a_tick_or_leave_with_nothing_held_does_nothing() {
    let r = Rig::new(&GOLDEN, None);
    still(&r, 3);
    assert_eq!(r.leave(), FaceReply::default());
    assert!(r.fakes.injector.strokes().is_empty());
}

#[test]
fn a_stale_table_stops_the_burst() {
    // After a dead key the next key goes as a scan code, so the app's own layout must match the one shown.
    let r = Rig::new(&["fr-fr", "en-us"], None);
    r.tap("c-0x001A").unwrap();
    r.fakes.source.set_foreground(Some(id("en-us")));
    r.fakes.layouts.switch(id("en-us"));
    let e = "c-0x0012";
    let (delay, _) = times();
    r.press(e).unwrap();
    assert_eq!(r.tick_in(delay), STATE, "it sends nothing and redraws");
    assert_eq!(r.fakes.layouts.refreshes(), 1);
    still(&r, 3);
    assert_eq!(r.let_go(e), Ok(FaceReply::default()));
    assert_eq!(r.fakes.injector.strokes(), [Stroke::Tap(Scan::new(0x1A))]);
}

#[test]
fn a_failed_send_stops_the_burst_with_one_notice() {
    let r = Rig::new(&GOLDEN, None);
    let (_, every) = times();
    hold(&r, A, 0);
    let refused = InjectError::Refused { sent: 0, total: 1 };
    r.fakes.injector.set_error(Some(refused));
    let warned = FaceReply {
        notice: Some(keys::KEYS_NOT_SENT),
        ..STATE
    };
    assert_eq!(r.tick_in(every), warned);
    r.fakes.injector.set_error(None);
    still(&r, 3);
    assert_eq!(r.let_go(A), Ok(FaceReply::default()));
    assert_eq!(r.fakes.injector.strokes(), [text("a")]);
}

#[test]
fn a_first_repeat_sent_in_part_stops_the_burst_and_lets_shift_go() {
    let r = Rig::new(&GOLDEN, None);
    let (delay, _) = times();
    r.tap("shift").unwrap();
    r.press(A).unwrap();
    let part = InjectError::Refused { sent: 1, total: 2 };
    r.fakes.injector.set_error(Some(part));
    assert_eq!(r.tick_in(delay).notice, Some(keys::KEYS_NOT_SENT));
    r.fakes.injector.set_error(None);
    assert_eq!(r.state()["keys"]["shift"]["on"], false, "part of it went");
    still(&r, 3);
    assert_eq!(r.let_go(A), Ok(FaceReply::default()));
}

#[test]
fn a_stale_first_repeat_keeps_shift_for_the_next_key() {
    let r = Rig::new(&["fr-fr", "en-us"], None);
    r.tap("c-0x001A").unwrap();
    r.tap("shift").unwrap();
    r.fakes.source.set_foreground(Some(id("en-us")));
    r.fakes.layouts.switch(id("en-us"));
    let (delay, _) = times();
    r.press("c-0x0012").unwrap();
    assert_eq!(r.tick_in(delay), STATE, "it sends nothing and redraws");
    still(&r, 3);
    assert_eq!(r.let_go("c-0x0012"), Ok(FaceReply::default()));
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
    assert_eq!(r.fakes.injector.strokes(), [text("A")]);
}

#[test]
fn opening_the_list_stops_the_burst() {
    // A second input, such as a reader's Invoke on the language key, opens the list while an arrow repeats.
    let r = Rig::new(&GOLDEN, Some(("lang_key", "list".into())));
    hold(&r, "down", 0);
    let typed = r.fakes.injector.strokes();
    assert_eq!(typed.len(), 1, "the first repeat");
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
            [text("A"), text("A")],
            "the hold goes on"
        );
        assert_eq!(r.leave(), STATE, "its own end redraws");
    }
}

/// Keys a run holds: a letter, Backspace, a latch and a key that never repeats.
const KEYS: [&str; 4] = [A, "bksp", "shift", "esc"];

/// One pointer or timer input on the face.
#[derive(Clone, Debug)]
enum Input {
    /// The pointer goes down on a key of `KEYS`.
    Press(usize),
    /// The hold timer ticks this many ms later.
    Tick(u64),
    /// The pointer lets go of a key of `KEYS`.
    LetGo(usize),
    /// The pointer leaves the key.
    Leave,
}

/// After a let-go or leave of the press `held` made: once that press typed, no latch is left down by its burst.
fn latches_went(r: &Rig, held: Option<(usize, usize)>) -> Result<(), TestCaseError> {
    let Some((k, sent)) = held else {
        return Ok(());
    };
    let typed = r.fakes.injector.strokes().len() > sent;
    let shift = r.state()["keys"]["shift"]["on"].clone();
    prop_assert!(
        KEYS[k] == "shift" || !typed || shift == false,
        "Shift outlived the burst"
    );
    Ok(())
}

fn input() -> impl Strategy<Value = Input> {
    let (delay, _) = times();
    prop_oneof![
        (0..KEYS.len()).prop_map(Input::Press),
        (0..delay).prop_map(Input::Tick),
        (0..delay).prop_map(Input::Tick),
        (0..KEYS.len()).prop_map(Input::LetGo),
        Just(Input::Leave),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn a_burst_never_outlives_the_press(shift in any::<bool>(), inputs in vec(input(), 0..40)) {
        let r = Rig::new(&GOLDEN, None);
        if shift {
            r.tap("shift").unwrap();
        }
        // The key held and how many strokes had gone out when it went down.
        let mut held: Option<(usize, usize)> = None;
        for i in inputs {
            let sent = r.fakes.injector.strokes().len();
            match i {
                Input::Press(k) => held = r.press(KEYS[k]).is_ok().then_some((k, sent)),
                Input::Tick(ms) => {
                    r.tick_in(ms);
                    let now = r.fakes.injector.strokes().len();
                    prop_assert!(held.is_some() || now == sent, "a tick typed with nothing held");
                }
                Input::LetGo(k) => {
                    r.let_go(KEYS[k]).unwrap();
                    latches_went(&r, held.take())?;
                }
                Input::Leave => {
                    r.leave();
                    latches_went(&r, held.take())?;
                }
            }
        }
    }
}
