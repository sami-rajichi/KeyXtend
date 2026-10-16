//! The actor while the pad scrolls (ADR-0029 decision 5): the hook stays on, the badge follows the pointer and each
//! press is told, so a press outside our windows can stop the scroll; the modes work as before.

use crate::logs::lines_of;
use crate::rig::{A, B, DPI, Heard, Rig, T0, T1, T2, down, due, moved, shown, told, up};
use kx_mod_mouse::Msg;
use kx_module_api::keys;
use kx_module_api::ports::GrabState::Off;
use kx_module_api::ports::{
    AssistNote, Button, HookError, Point, PointerEvent, PointerMoved, PointerPressed, Scan, Toggle,
};
use tracing::Level;

/// Left Shift.
const SHIFT: Scan = Scan::new(0x2A);
/// The OS error code a refused hook carries.
const CODE: i32 = 1428;

/// A press told on the bus.
fn pressed(ours: bool) -> Heard {
    Heard::Pressed(PointerPressed { ours })
}

/// A left press on one of our windows at `at`.
fn ours(at: Point) -> PointerEvent {
    PointerEvent::Down {
        button: Button::Left,
        at,
        ours: true,
        dpi: DPI,
    }
}

/// A started rig that scrolls, with the command settled and what it said so far taken.
fn scrolling(rig: Rig) -> Rig {
    let mut rig = rig;
    rig.begin();
    rig.send(Msg::Scroll(true));
    rig.settle(T0);
    rig.take();
    rig
}

#[test]
fn scrolling_alone_starts_the_hook_passes_every_event_and_its_end_stops_it() {
    let mut rig = Rig::new();
    rig.begin();
    rig.take();
    rig.send(Msg::Scroll(true));
    assert!(rig.hook.running(), "scrolling needs the hook");
    assert!(rig.take().is_empty(), "no state change and no notice");
    assert!(!rig.feed(moved(A), T0));
    assert!(!rig.feed(down(B), T1), "the press passes");
    assert!(!rig.feed(up(B), T1));
    assert!(!rig.feed(ours(A), T2));
    assert!(rig.hook.take_sent().is_empty(), "nothing of ours is sent");
    let badge = Heard::Moved(PointerMoved { at: A });
    assert_eq!(rig.take(), [badge, pressed(false), pressed(true)]);
    rig.send(Msg::Scroll(false));
    rig.settle(T2);
    assert!(!rig.hook.running(), "nothing else needs it");
    assert_eq!(rig.hook.starts(), 1);
}

#[test]
fn with_only_a_latch_no_press_or_move_is_told() {
    let mut rig = Rig::new();
    rig.begin();
    rig.send(Msg::Latches(vec![SHIFT]));
    rig.take();
    let _ = rig.feed(moved(A), T0);
    let _ = rig.feed(down(A), T1);
    let _ = rig.feed(up(A), T1);
    assert_eq!(rig.take(), [Heard::Used]);
    rig.send(Msg::Scroll(true));
    rig.settle(T2);
    let _ = rig.feed(down(B), T2);
    assert_eq!(rig.take(), [pressed(false)], "told once scrolling");
}

#[test]
fn a_hold_while_scrolling_still_right_clicks_and_its_press_is_told_once() {
    let mut rig = scrolling(Rig::saved_rclick());
    let s = rig.settings.clone();
    assert!(rig.feed(down(A), T0), "the press is held back");
    rig.settle(due(&s, T0));
    let _ = rig.feed(up(A), due(&s, T0));
    let start = AssistNote::HoldStart {
        at: A,
        ring_after_ms: s.ring_after_ms,
        hold_ms: s.hold_ms,
    };
    let fired = AssistNote::Fired { at: A };
    let want = [pressed(false), Heard::Note(start), Heard::Note(fired)];
    assert_eq!(rig.take(), want);
    rig.send(Msg::Scroll(false));
    rig.settle(T2);
    assert!(rig.hook.running(), "Right-click still needs it");
}

#[test]
fn a_mode_turned_off_while_scrolling_keeps_the_hook_until_scrolling_ends() {
    let mut rig = scrolling(Rig::saved_rclick());
    rig.toggle(Toggle::RightClick);
    rig.settle(T0);
    assert!(rig.hook.running(), "scrolling still needs it");
    assert_eq!(rig.take(), [shown(false, Off)]);
    rig.send(Msg::Scroll(false));
    rig.settle(T1);
    assert!(!rig.hook.running());
}

#[test]
fn a_turn_or_a_speed_change_sends_the_engine_nothing() {
    let mut rig = scrolling(Rig::new());
    let wakes = rig.hook.wakes();
    rig.send(Msg::Scroll(true));
    assert_eq!(rig.hook.wakes(), wakes, "no command, no wake");
    assert_eq!(rig.hook.starts(), 1);
}

#[test]
fn a_hook_refused_for_scrolling_tells_no_one_and_the_next_change_tries_again() {
    let lines = lines_of(|| {
        let mut rig = Rig::new();
        rig.begin();
        rig.take();
        rig.hook.fail_next(HookError::Refused(CODE));
        rig.send(Msg::Scroll(true));
        assert!(!rig.hook.running());
        assert!(rig.take().is_empty(), "it scrolls without the badge");
        rig.send(Msg::Scroll(true));
        assert!(rig.hook.running(), "the next change tries again");
    });
    let warned = lines.iter().filter(|(l, _)| *l == Level::WARN).count();
    assert_eq!(warned, 1, "{lines:?}");
}

#[test]
fn a_run_started_for_a_mode_while_scrolling_tells_presses_too() {
    let mut rig = Rig::new();
    rig.begin();
    rig.hook.fail_next(HookError::Refused(CODE));
    rig.send(Msg::Scroll(true));
    rig.toggle(Toggle::Grab);
    rig.settle(T0);
    rig.take();
    let _ = rig.feed(down(A), T1);
    assert_eq!(rig.take()[0], pressed(false));
}

#[test]
fn a_given_up_engine_while_scrolling_gets_one_fresh_run_until_scrolling_changes() {
    let mut rig = scrolling(Rig::new());
    let reset = [shown(false, Off), told(keys::MOUSE_RESET)];
    for _ in 0..2 {
        rig.hook.give_up();
        rig.pump();
        assert_eq!(rig.take(), reset);
    }
    assert_eq!(rig.hook.starts(), 2, "one fresh run, never a loop");
    assert!(!rig.hook.running());
    rig.send(Msg::Scroll(false));
    rig.send(Msg::Scroll(true));
    assert!(rig.hook.running(), "a new scroll starts afresh");
    let _ = rig.feed(down(B), T0);
    assert_eq!(rig.take(), [pressed(false)]);
}

#[test]
fn a_stop_while_scrolling_lets_the_hook_go() {
    let mut rig = scrolling(Rig::new());
    assert!(!rig.send(Msg::Stop), "the actor is done");
    assert!(!rig.hook.running());
    assert_eq!(rig.hook.stops(), 1);
}
