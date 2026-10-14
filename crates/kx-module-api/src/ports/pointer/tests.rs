//! The pointer port's plain types: replayed events, distinct errors, and `Debug` that hides every point and key.

use super::*;
use crate::REDACTED_MARKER;
use std::collections::HashSet;

/// A point and a key whose numbers appear nowhere else in the output.
const AT: Point = Point { x: 4321, y: 8765 };
const KEY: Scan = Scan::new(0xE01D);
/// Their numbers in decimal and hex, and the key's type name, which no output may show.
const HIDDEN: [&str; 6] = ["4321", "8765", "57373", "E01D", "e01d", "Scan"];

#[test]
fn a_refusal_names_the_os_code() {
    let text = HookError::Refused(1428).to_string();
    assert!(text.contains("1428"), "{text}");
}

#[test]
fn a_passed_event_is_sent_again_as_its_own_act_and_esc_has_none() {
    let at = Point { x: -3, y: 9 };
    let down = PointerEvent::Down {
        button: Button::Right,
        at,
        ours: true,
        dpi: 144,
    };
    assert_eq!(down.act(), Some(PointerAct::Down(Button::Right, at)));
    let up = PointerEvent::Up {
        button: Button::Middle,
        at,
    };
    assert_eq!(up.act(), Some(PointerAct::Up(Button::Middle, at)));
    assert_eq!(PointerEvent::Move { at }.act(), Some(PointerAct::Move(at)));
    assert_eq!(PointerEvent::Esc.act(), None, "Esc is never dropped");
}

#[test]
fn each_error_says_something_else() {
    let errors = [HookError::Busy, HookError::Refused(0), HookError::Thread];
    let texts: HashSet<_> = errors.iter().map(ToString::to_string).collect();
    assert_eq!(texts.len(), errors.len());
}

#[test]
fn events_and_acts_hide_every_point_and_key() {
    let button = Button::Left;
    let events = [
        PointerEvent::Down {
            button,
            at: AT,
            ours: false,
            dpi: 96,
        },
        PointerEvent::Up { button, at: AT },
        PointerEvent::Move { at: AT },
    ];
    let acts = [
        PointerAct::Move(AT),
        PointerAct::Down(button, AT),
        PointerAct::Up(button, AT),
        PointerAct::KeyDown(KEY),
        PointerAct::KeyUp(KEY),
    ];
    let events = events.iter().map(|e| format!("{e:?}"));
    for text in events.chain(acts.iter().map(|a| format!("{a:?}"))) {
        assert!(text.contains(REDACTED_MARKER), "{text}");
        for hidden in HIDDEN {
            assert!(!text.contains(hidden), "{text} shows {hidden}");
        }
    }
}

#[test]
fn events_and_acts_still_show_their_button_and_flags() {
    let down = PointerEvent::Down {
        button: Button::Middle,
        at: AT,
        ours: true,
        dpi: 144,
    };
    let want = format!("Down {{ button: Middle, at: {REDACTED_MARKER}, ours: true, dpi: 144 }}");
    assert_eq!(format!("{down:?}"), want);
    let up = PointerAct::Up(Button::Right, AT);
    assert_eq!(format!("{up:?}"), format!("Up(Right, {REDACTED_MARKER})"));
    assert_eq!(format!("{:?}", PointerEvent::Esc), "Esc");
}
