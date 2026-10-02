//! Tests of the hook thread's state: the hold timer and the wait behind our own input.

use spike_core::hold::Button;
use windows::Win32::Foundation::POINT;
use windows::Win32::UI::Input::KeyboardAndMouse::{VIRTUAL_KEY, VK_ESCAPE, VK_RETURN};
use windows::Win32::UI::WindowsAndMessaging::{KBDLLHOOKSTRUCT_FLAGS, WM_KEYDOWN, WM_MOUSEMOVE};

use super::*;
use crate::hookio::SIM_TAG;

/// How long a hold must last, in ms; the tests never reach it.
const HOLD_MS: u64 = 1_000;
/// Moves up to this many px count as still.
const STILL_PX: i32 = 4;
/// The echo wait, in ms: far longer than a test runs.
const REPLY_MS: u64 = 60_000;
/// Most hook timings a test host keeps.
const SAMPLES: usize = 2;

/// A host on the simulated user's input with nothing queued, and the live values it shares.
fn shared() -> (Host, Arc<Mutex<Live>>) {
    let setup = Setup {
        hold_ms: HOLD_MS,
        still_px: STILL_PX,
        source: Source::Simulated,
        own: Vec::new(),
        reply_ms: REPLY_MS,
        samples: SAMPLES,
    };
    let live = Arc::default();
    (Host::new(setup, 0, Arc::clone(&live)), live)
}

/// A host with nothing queued and the echo wait `REPLY_MS`.
fn host() -> Host {
    shared().0
}

/// The live values, read once.
fn live_of(live: &Arc<Mutex<Live>>) -> Live {
    *live.lock().expect("no test holds the lock")
}

/// The simulated user's mouse message at `p`. Its time is 0: the engine is off, so how long it waited does not matter.
fn sim_mouse(p: Pt) -> MSLLHOOKSTRUCT {
    MSLLHOOKSTRUCT {
        pt: POINT { x: p.x, y: p.y },
        flags: LLMHF_INJECTED,
        dwExtraInfo: SIM_TAG,
        ..MSLLHOOKSTRUCT::default()
    }
}

/// A key message for `vk` with `flags`, tagged as the simulated user's.
fn sim_key(vk: VIRTUAL_KEY, flags: KBDLLHOOKSTRUCT_FLAGS) -> KBDLLHOOKSTRUCT {
    KBDLLHOOKSTRUCT {
        vkCode: u32::from(vk.0),
        flags,
        dwExtraInfo: SIM_TAG,
        ..KBDLLHOOKSTRUCT::default()
    }
}

#[test]
fn timer_ms_rounds_up_and_never_overflows() {
    assert_eq!(timer_ms(1_500_000, 0), 1501);
    assert_eq!(timer_ms(1_500_999, 0), 1501);
    assert_eq!(timer_ms(0, 10), 1);
    assert_eq!(timer_ms(i64::MAX, i64::MIN), u32::MAX);
}

#[test]
fn input_we_sent_holds_later_input_back() {
    let mut host = host();
    assert!(!host.behind());
    host.sent(now_us());
    assert!(host.behind());
}

#[test]
fn a_failed_injection_ends_the_wait_and_keeps_the_error() {
    let mut host = host();
    host.sent(now_us());
    host.failed("injection failed".to_string());
    assert!(!host.behind());
    assert_eq!(host.into_report().errors, ["injection failed"]);
}

#[test]
fn input_that_never_comes_back_stops_the_wait_and_is_reported() {
    let mut host = host();
    host.sent(now_us() - host.echo_wait_us - 1);
    assert!(!host.behind());
    assert_eq!(host.into_report().errors.len(), 1);
}

#[test]
fn every_event_is_counted_but_only_the_set_number_of_timings_is_kept() {
    let (mut host, live) = shared();
    for _ in 0..=SAMPLES {
        host.note(None, 1);
    }
    assert_eq!(live_of(&live).seen, SAMPLES + 1);
    let report = host.into_report();
    assert_eq!((report.seen, report.hook_us.len()), (SAMPLES + 1, SAMPLES));
}

#[test]
fn only_a_point_outside_our_windows_is_kept_as_the_last_one() {
    let (mut host, live) = shared();
    let (outside, ours) = (Pt { x: 1, y: 2 }, Pt { x: 3, y: 4 });
    host.note(Some((outside, false)), 1);
    host.note(Some((ours, true)), 1);
    assert_eq!(live_of(&live).last_outside, Some(outside));
}

#[test]
fn input_let_through_waits_behind_ours_in_order() {
    let mut host = host();
    let (a, b) = (Pt { x: 1, y: 1 }, Pt { x: 2, y: 2 });
    assert!(
        !host.queue(Event::Move(a), now_us()),
        "nothing of ours is out, so it passes"
    );
    assert!(host.acts.is_empty());
    host.sent(now_us());
    assert!(host.queue(Event::Move(a), now_us()));
    assert!(host.queue(Event::Down(Button::Left, b, false), now_us()));
    assert_eq!(host.acts, [Act::Move(a), Act::Down(Button::Left, b)]);
}

#[test]
fn our_input_coming_back_ends_the_wait_and_is_not_counted() {
    let mut host = host();
    host.sent(now_us());
    let echo = MSLLHOOKSTRUCT {
        dwExtraInfo: TAG,
        ..sim_mouse(Pt::default())
    };
    assert!(!host.on_mouse(WM_MOUSEMOVE, &echo));
    assert!(!host.behind());
    assert_eq!(host.into_report().seen, 0);
}

#[test]
fn only_the_simulated_users_mouse_reaches_the_engine() {
    let (mut host, live) = shared();
    let p = Pt { x: 5, y: 6 };
    let real = MSLLHOOKSTRUCT {
        flags: 0,
        dwExtraInfo: 0,
        ..sim_mouse(p)
    };
    assert!(!host.on_mouse(WM_MOUSEMOVE, &real));
    assert_eq!(
        live_of(&live).seen,
        0,
        "a hand on the mouse cannot disturb a run"
    );
    assert!(!host.on_mouse(WM_MOUSEMOVE, &sim_mouse(p)));
    let now = live_of(&live);
    assert_eq!((now.seen, now.last_outside), (1, Some(p)));
}

#[test]
fn a_simulated_move_while_ours_is_out_is_held_back() {
    let mut host = host();
    let p = Pt { x: 7, y: 8 };
    host.sent(now_us());
    assert!(host.on_mouse(WM_MOUSEMOVE, &sim_mouse(p)));
    assert_eq!(host.acts, [Act::Move(p)]);
}

#[test]
fn only_an_esc_from_the_simulated_user_reaches_the_engine() {
    let mut host = host();
    host.on_key(WM_KEYDOWN, &sim_key(VK_RETURN, LLKHF_INJECTED));
    host.on_key(WM_KEYDOWN, &sim_key(VK_ESCAPE, KBDLLHOOKSTRUCT_FLAGS(0)));
    assert_eq!(
        host.report.seen, 0,
        "Enter, and an Esc typed by hand, are left alone"
    );
    host.on_key(WM_KEYDOWN, &sim_key(VK_ESCAPE, LLKHF_INJECTED));
    assert_eq!(host.into_report().seen, 1);
}
