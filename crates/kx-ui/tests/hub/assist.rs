//! The pointer overlay's state through the hub: the mouse modes, the pointer, the hold ring and its burst as one JSON,
//! with at most one wake in flight until QML reads it, the keyboard woken on each mode or latch change, and no point in
//! any log.

use crate::Rig;
use crate::app::keeper;
use crate::logs::lines_of;
use kx_module_api::ports::{
    AssistChanged, AssistNote, GrabState, LatchesUsed, Point, PointerMoved,
};
use kx_module_api::{Bus, Subscription};
use kx_platform::ui::SysLook;
use kx_test_support::host::FakeCore;
use kx_ui::{Hub, Wake};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

/// Where a hold began; odd numbers, so a log line that held one would show it.
const AT: Point = Point { x: 4321, y: 8765 };
/// Where the pointer went next.
const TO: Point = Point { x: 4329, y: 8771 };
/// Right-click on, with the mouse module running.
const RCLICK: AssistChanged = AssistChanged {
    live: true,
    rclick: true,
    grab: GrabState::Off,
};

/// A rig whose hub hears a fresh bus, the bus, and the subscriptions that keep it hearing.
fn heard() -> (Rig, Bus, Vec<Subscription>) {
    let rig = Rig::new();
    let bus = Bus::new(Arc::new(FakeCore::default()));
    let subs = rig.hub.listen(&bus).unwrap();
    (rig, bus, subs)
}

/// The same, with Right-click on and every wake so far read and forgotten.
fn rclick_on() -> (Rig, Bus, Vec<Subscription>) {
    let (rig, bus, subs) = heard();
    bus.publish(&RCLICK).unwrap();
    let _ = state(&rig);
    rig.wakes();
    (rig, bus, subs)
}

/// The overlay's state as QML reads it, which lets the next change wake again.
fn state(rig: &Rig) -> Value {
    serde_json::from_str(&rig.hub.assist_json()).unwrap()
}

/// How many times the rig woke for `what` since the last look; it forgets every wake.
fn woke(rig: &Rig, what: Wake) -> usize {
    rig.wakes().iter().filter(|w| **w == what).count()
}

/// A hold at `AT` whose ring shows after `ring_after_ms` and which fires at `hold_ms`.
fn hold(ring_after_ms: u32, hold_ms: u32) -> AssistNote {
    AssistNote::HoldStart {
        at: AT,
        ring_after_ms,
        hold_ms,
    }
}

#[test]
fn the_state_starts_off_with_nothing_shown_and_the_usual_pointer() {
    let rig = Rig::new();
    let want = json!({
        "live": false, "rclick": false, "grab": "off", "point": null, "ring": null, "burst": null,
        "pointerPx": SysLook::POINTER_PX, "seq": 0
    });
    assert_eq!(state(&rig), want);
}

#[test]
fn a_mode_change_shows_in_the_state_and_wakes_once() {
    let (rig, bus, _subs) = heard();
    bus.publish(&RCLICK).unwrap();
    assert_eq!(woke(&rig, Wake::Assist), 1);
    let s = state(&rig);
    assert_eq!(
        (&s["live"], &s["rclick"], &s["grab"]),
        (&json!(true), &json!(true), &json!("off"))
    );
    for (grab, name) in [
        (GrabState::Armed, "armed"),
        (GrabState::Carrying, "carrying"),
    ] {
        bus.publish(&AssistChanged { grab, ..RCLICK }).unwrap();
        assert_eq!(woke(&rig, Wake::Assist), 1, "{name}");
        assert_eq!(state(&rig)["grab"], name);
    }
    let again = AssistChanged {
        grab: GrabState::Carrying,
        ..RCLICK
    };
    bus.publish(&again).unwrap();
    assert_eq!(woke(&rig, Wake::Assist), 0, "no change, no wake");
}

#[test]
fn moves_before_a_read_wake_once_and_a_read_lets_the_next_move_wake() {
    let (rig, bus, _subs) = rclick_on();
    let seq = state(&rig)["seq"].clone();
    bus.publish(&PointerMoved { at: AT }).unwrap();
    bus.publish(&PointerMoved { at: TO }).unwrap();
    assert_eq!(woke(&rig, Wake::Assist), 1, "one wake in flight");
    let s = state(&rig);
    assert_eq!(s["point"], json!({ "x": TO.x, "y": TO.y }));
    assert!(s["seq"].as_u64() > seq.as_u64(), "the seq grew");
    bus.publish(&PointerMoved { at: AT }).unwrap();
    assert_eq!(woke(&rig, Wake::Assist), 1, "the read let it wake again");
}

#[test]
fn moves_while_no_mode_is_on_are_not_kept_and_wake_nothing() {
    let (rig, bus, _subs) = heard();
    bus.publish(&PointerMoved { at: AT }).unwrap();
    assert!(rig.wakes().is_empty());
    assert_eq!(state(&rig)["point"], Value::Null);
}

#[test]
fn a_hold_rings_at_its_press_point_for_the_time_left_after_the_ring_shows() {
    let (rig, bus, _subs) = rclick_on();
    bus.publish(&PointerMoved { at: TO }).unwrap();
    let _ = state(&rig);
    rig.wakes();
    for (after, fire, wave) in [(150, 1_500, 1_350), (150, 500, 350)] {
        bus.publish(&hold(after, fire)).unwrap();
        assert_eq!(woke(&rig, Wake::Assist), 1);
        let s = state(&rig);
        let want =
            json!({ "x": AT.x, "y": AT.y, "afterMs": after, "waveMs": wave, "seq": s["seq"] });
        assert_eq!(s["ring"], want, "{after}/{fire}");
    }
}

#[test]
fn a_hold_that_ends_or_a_cancelled_carry_clears_the_ring() {
    for end in [AssistNote::HoldEnd, AssistNote::Cancelled] {
        let (rig, bus, _subs) = rclick_on();
        bus.publish(&hold(150, 1_500)).unwrap();
        bus.publish(&end).unwrap();
        assert_eq!(state(&rig)["ring"], Value::Null, "{end:?}");
    }
}

#[test]
fn a_fired_hold_bursts_at_its_point_with_a_new_seq_and_its_ring_goes() {
    let (rig, bus, _subs) = rclick_on();
    bus.publish(&hold(150, 1_500)).unwrap();
    let mut last = state(&rig)["ring"]["seq"].as_u64();
    for _ in 0..2 {
        bus.publish(&AssistNote::Fired { at: AT }).unwrap();
        let s = state(&rig);
        assert_eq!(s["ring"], Value::Null);
        assert_eq!(s["burst"], json!({ "x": AT.x, "y": AT.y, "seq": s["seq"] }));
        assert!(s["burst"]["seq"].as_u64() > last, "a new burst");
        last = s["burst"]["seq"].as_u64();
    }
}

#[test]
fn every_mode_off_or_a_stopped_module_forgets_the_point_ring_and_burst() {
    let off = AssistChanged {
        rclick: false,
        ..RCLICK
    };
    let shown = |s: &Value| ["point", "ring", "burst"].map(|k| s[k] != Value::Null);
    for end in [off, AssistChanged::OFF] {
        let (rig, bus, _subs) = rclick_on();
        bus.publish(&PointerMoved { at: TO }).unwrap();
        bus.publish(&hold(150, 1_500)).unwrap();
        bus.publish(&AssistNote::Fired { at: AT }).unwrap();
        bus.publish(&hold(150, 1_500)).unwrap();
        assert_eq!(
            shown(&state(&rig)),
            [true; 3],
            "a burst still plays as the next hold rings"
        );
        bus.publish(&end).unwrap();
        let s = state(&rig);
        assert_eq!(shown(&s), [false; 3], "{end:?}");
        assert_eq!(s["live"], end.live);
    }
}

#[test]
fn the_pointer_size_follows_the_system_and_wakes() {
    let rig = Rig::new();
    let _ = state(&rig);
    let px = SysLook::POINTER_PX * 2;
    rig.hub.set_sys(SysLook {
        pointer_px: px,
        ..SysLook::default()
    });
    assert_eq!(rig.wakes(), [Wake::Look, Wake::Assist]);
    assert_eq!(state(&rig)["pointerPx"], px);
    rig.hub.set_sys(SysLook {
        pointer_px: px,
        dark: true,
        ..SysLook::default()
    });
    assert_eq!(rig.wakes(), [Wake::Look], "the same size wakes no overlay");
}

#[test]
fn a_mode_change_or_used_latches_wake_the_keyboard_every_time() {
    let (rig, bus, _subs) = heard();
    for _ in 0..2 {
        bus.publish(&RCLICK).unwrap();
        bus.publish(&LatchesUsed).unwrap();
    }
    bus.publish(&PointerMoved { at: AT }).unwrap();
    bus.publish(&hold(150, 1_500)).unwrap();
    assert_eq!(
        woke(&rig, Wake::Keys),
        4,
        "the keyboard reads its state at once"
    );
}

#[test]
fn a_wake_qml_never_read_goes_to_the_next_waker() {
    let rig = Rig::new();
    let (hub, _requests) = Hub::new(rig.ports()).unwrap();
    let bus = Bus::new(Arc::new(FakeCore::default()));
    let _subs = hub.listen(&bus).unwrap();
    bus.publish(&RCLICK).unwrap();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let taken = || std::mem::take(&mut *seen.lock().unwrap());
    hub.set_waker(keeper(&seen));
    assert_eq!(taken(), [Wake::Assist], "it came before the waker");
    hub.drop_waker();
    hub.set_waker(keeper(&seen));
    assert_eq!(taken(), [Wake::Assist], "the windows were made again first");
    let _ = hub.assist_json();
    hub.drop_waker();
    hub.set_waker(keeper(&seen));
    assert!(taken().is_empty(), "read already, so no wake again");
}

#[test]
fn no_log_line_holds_a_point() {
    let probe = lines_of(|| tracing::info!(n = 7, "probe"));
    assert!(probe.iter().any(|l| l.contains("n=7")), "{probe:?}");
    let lines = lines_of(|| {
        let (rig, bus, _subs) = heard();
        bus.publish(&RCLICK).unwrap();
        bus.publish(&PointerMoved { at: TO }).unwrap();
        bus.publish(&hold(150, 1_500)).unwrap();
        bus.publish(&AssistNote::Fired { at: AT }).unwrap();
        let _ = state(&rig);
        bus.publish(&AssistChanged::OFF).unwrap();
    });
    let coords = [AT.x, AT.y, TO.x, TO.y].map(|c| c.to_string());
    let leaked: Vec<&String> = lines
        .iter()
        .filter(|l| coords.iter().any(|c| l.contains(c)))
        .collect();
    assert!(leaked.is_empty(), "{leaked:?}");
}
