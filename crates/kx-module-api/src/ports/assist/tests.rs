//! The assist events: unique kebab-case names, their gates, and `Debug` that hides every point and key.

use super::*;
use crate::capability::is_kebab;
use crate::ports::{LayoutChanged, PowerAsk, PowerAsked, PowerChanged};
use crate::{ModuleStateChanged, Notice, REDACTED_MARKER, SettingsChanged};
use std::collections::HashSet;

/// A point whose numbers appear nowhere else in the output.
const AT: Point = Point { x: 4321, y: 8765 };
/// The point's numbers, which no output may show.
const AT_TEXT: [&str; 2] = ["4321", "8765"];
/// Left Shift and right Ctrl.
const HELD: [Scan; 2] = [Scan::new(0x2A), Scan::new(0xE01D)];
/// Those scan codes in decimal and hex, and the type's name, which no output may show.
const HELD_TEXT: [&str; 7] = ["42", "2A", "2a", "57373", "E01D", "e01d", "Scan"];
/// The `InjectInput` gate.
const INJECT: Option<Capability> = Some(Capability::InjectInput);

/// An event's name and gate.
fn row<E: Event>() -> (&'static str, Option<Capability>) {
    (E::NAME, E::CAPABILITY)
}

/// The assist events, in the order `gates_are_inject_input_for_toggles_and_latches_only` checks.
fn assist() -> [(&'static str, Option<Capability>); 9] {
    [
        row::<AssistToggle>(),
        row::<AssistChanged>(),
        row::<AssistAsk>(),
        row::<AssistNote>(),
        row::<PointerMoved>(),
        row::<PointerPressed>(),
        row::<LatchesChanged>(),
        row::<LatchesUsed>(),
        row::<LatchesAsk>(),
    ]
}

/// Fails when `text` shows any of `secrets`.
fn hides(text: &str, secrets: &[&str]) {
    for secret in secrets {
        assert!(!text.contains(secret), "{text} shows {secret}");
    }
}

#[test]
fn names_are_kebab_case_and_unique_in_the_crate() {
    let older = [
        Notice::NAME,
        ModuleStateChanged::NAME,
        SettingsChanged::NAME,
        LayoutChanged::NAME,
        PowerAsked::NAME,
        PowerChanged::NAME,
        PowerAsk::NAME,
    ];
    let names: Vec<_> = assist().iter().map(|r| r.0).chain(older).collect();
    assert!(names.iter().all(|n| is_kebab(n)), "{names:?}");
    let unique: HashSet<_> = names.iter().collect();
    assert_eq!(unique.len(), names.len(), "{names:?}");
}

#[test]
fn gates_are_inject_input_for_toggles_and_latches_only() {
    let gates = assist().map(|r| r.1);
    let want = [INJECT, None, None, None, None, None, INJECT, INJECT, INJECT];
    assert_eq!(gates, want);
}

#[test]
fn notes_hide_every_point() {
    let notes = [
        AssistNote::HoldStart {
            at: AT,
            ring_after_ms: 150,
            hold_ms: 1500,
        },
        AssistNote::Fired { at: AT },
        AssistNote::Picked { at: AT },
        AssistNote::Dropped { at: AT },
    ];
    for note in notes {
        let text = format!("{note:?}");
        hides(&text, &AT_TEXT);
        assert!(text.contains(REDACTED_MARKER), "{text}");
    }
    let fired = format!("{:?}", AssistNote::Fired { at: AT });
    assert_eq!(fired, format!("Fired {{ at: {REDACTED_MARKER} }}"));
}

#[test]
fn a_hold_start_still_shows_its_timings() {
    let note = AssistNote::HoldStart {
        at: AT,
        ring_after_ms: 150,
        hold_ms: 1500,
    };
    let want = format!("HoldStart {{ at: {REDACTED_MARKER}, ring_after_ms: 150, hold_ms: 1500 }}");
    assert_eq!(format!("{note:?}"), want);
    assert_eq!(format!("{:?}", AssistNote::Cancelled), "Cancelled");
}

#[test]
fn a_pointer_move_hides_its_point() {
    let text = format!("{:?}", PointerMoved { at: AT });
    hides(&text, &AT_TEXT);
    assert_eq!(text, format!("PointerMoved {{ at: {REDACTED_MARKER} }}"));
}

#[test]
fn a_press_is_named_and_shows_only_whether_it_was_ours() {
    assert_eq!(PointerPressed::NAME, "pointer-pressed");
    let text = format!("{:?}", PointerPressed { ours: true });
    assert_eq!(text, "PointerPressed { ours: true }");
}

#[test]
fn latches_show_only_how_many_are_held() {
    let text = format!("{:?}", LatchesChanged { held: HELD.into() });
    hides(&text, &HELD_TEXT);
    assert_eq!(text, "LatchesChanged { count: 2 }");
}

#[test]
fn off_is_a_module_that_does_not_run_with_every_mode_off() {
    let off = AssistChanged::OFF;
    assert!(!off.live && !off.rclick);
    assert_eq!(off.grab, GrabState::Off);
}
