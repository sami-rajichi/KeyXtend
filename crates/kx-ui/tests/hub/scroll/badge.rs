//! The badge's scroll part: the way and the speed while scrolling, beside the mode part, following the pointer only
//! while the mouse module runs, as it sends the moves.

use super::{DOWN, STILL};
use crate::Rig;
use crate::assist::{RCLICK, heard, woke};
use kx_module_api::ports::{AssistChanged, AssistNote, Dir, Point, PointerMoved, ScrollChanged};
use kx_ui::Wake;
use serde_json::{Value, json};

/// Where the pointer is.
const AT: Point = Point { x: 4321, y: 8765 };
/// The mouse module running with every mode off.
const MOUSE: AssistChanged = AssistChanged {
    live: true,
    ..AssistChanged::OFF
};

/// The overlay's state as QML reads it.
fn badge(rig: &Rig) -> Value {
    serde_json::from_str(&rig.hub.assist_json()).unwrap()
}

/// The pointer's point as the state holds it.
fn point() -> Value {
    json!({ "x": AT.x, "y": AT.y })
}

#[test]
fn the_badge_holds_a_mode_part_and_a_scroll_part() {
    let (rig, bus, _subs) = heard();
    bus.publish(&RCLICK).unwrap();
    bus.publish(&ScrollChanged { fast: true, ..DOWN }).unwrap();
    let s = badge(&rig);
    let part = json!({ "dir": "down", "fast": true });
    assert_eq!((&s["mode"], &s["scroll"]), (&json!("rclick"), &part));
    bus.publish(&STILL).unwrap();
    let s = badge(&rig);
    assert_eq!((&s["mode"], &s["scroll"]), (&json!("rclick"), &Value::Null));
}

#[test]
fn a_scroll_change_wakes_the_overlay_once_per_read() {
    let (rig, bus, _subs) = heard();
    bus.publish(&MOUSE).unwrap();
    let _ = badge(&rig);
    rig.wakes();
    bus.publish(&DOWN).unwrap();
    bus.publish(&ScrollChanged { fast: true, ..DOWN }).unwrap();
    assert_eq!(woke(&rig, Wake::Assist), 1, "one wake in flight");
    assert_eq!(
        badge(&rig)["scroll"],
        json!({ "dir": "down", "fast": true })
    );
    bus.publish(&ScrollChanged { fast: true, ..DOWN }).unwrap();
    assert_eq!(woke(&rig, Wake::Assist), 0, "no change, no wake");
    let left = ScrollChanged {
        dir: Some(Dir::Left),
        ..DOWN
    };
    bus.publish(&left).unwrap();
    assert_eq!(woke(&rig, Wake::Assist), 1);
}

/// With no mode on, the badge shows only the scroll part, and a hold shows no ring.
#[test]
fn while_scrolling_alone_the_badge_follows_the_pointer_with_no_ring() {
    let (rig, bus, _subs) = heard();
    bus.publish(&MOUSE).unwrap();
    bus.publish(&DOWN).unwrap();
    bus.publish(&PointerMoved { at: AT }).unwrap();
    let s = badge(&rig);
    assert_eq!((&s["mode"], &s["point"]), (&Value::Null, &point()));
    let hold = AssistNote::HoldStart {
        at: AT,
        ring_after_ms: 150,
        hold_ms: 1_500,
    };
    bus.publish(&hold).unwrap();
    bus.publish(&AssistNote::Fired { at: AT }).unwrap();
    let s = badge(&rig);
    assert_eq!((&s["ring"], &s["burst"]), (&Value::Null, &Value::Null));
    bus.publish(&STILL).unwrap();
    let s = badge(&rig);
    assert_eq!(
        (&s["scroll"], &s["point"]),
        (&Value::Null, &Value::Null),
        "idle"
    );
}

/// A mode turned off mid-scroll takes its ring and burst, while the badge keeps the pointer for the scroll part.
#[test]
fn a_mode_off_while_scrolling_clears_its_ring_and_keeps_the_point() {
    let (rig, bus, _subs) = heard();
    bus.publish(&RCLICK).unwrap();
    bus.publish(&DOWN).unwrap();
    bus.publish(&PointerMoved { at: AT }).unwrap();
    let hold = AssistNote::HoldStart {
        at: AT,
        ring_after_ms: 150,
        hold_ms: 1_500,
    };
    bus.publish(&hold).unwrap();
    assert_ne!(badge(&rig)["ring"], Value::Null);
    bus.publish(&MOUSE).unwrap();
    let s = badge(&rig);
    assert_eq!((&s["mode"], &s["ring"]), (&Value::Null, &Value::Null));
    assert_eq!(s["point"], point());
    assert_eq!(s["scroll"], json!({ "dir": "down", "fast": false }));
}

/// The pad still scrolls with the mouse module off, but no moves come, so no badge shows (edge case 14).
#[test]
fn with_the_mouse_module_off_the_pad_scrolls_with_no_badge() {
    let (rig, bus, _subs) = heard();
    bus.publish(&DOWN).unwrap();
    bus.publish(&PointerMoved { at: AT }).unwrap();
    let s = badge(&rig);
    assert_eq!((&s["scroll"], &s["point"]), (&Value::Null, &Value::Null));
    bus.publish(&MOUSE).unwrap();
    bus.publish(&PointerMoved { at: AT }).unwrap();
    let s = badge(&rig);
    assert_eq!(
        s["scroll"],
        json!({ "dir": "down", "fast": false }),
        "it came back"
    );
    assert_eq!(s["point"], point());
    bus.publish(&AssistChanged::OFF).unwrap();
    let s = badge(&rig);
    assert_eq!((&s["scroll"], &s["point"]), (&Value::Null, &Value::Null));
}
