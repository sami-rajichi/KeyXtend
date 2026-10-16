//! The actor over the fake hook: the hook runs only while needed, the modes follow spec §5.1, and the engine's notes
//! reach the bus.

use crate::rig::{A, B, C, Heard, RCLICK, Rig, T0, T1, T2, down, due, moved, shown, up};
use kx_mod_mouse::{MAX_LATCHES, MouseSettings, Msg};
use kx_module_api::ports::GrabState::{Armed, Carrying, Off};
use kx_module_api::ports::{
    AssistNote, Button, PointerAct, PointerEvent, PointerMoved, Scan, Toggle,
};

/// Left Shift.
const SHIFT: Scan = Scan::new(0x2A);
/// The first of the made-up modifier codes that fill every latch.
const MOD_BASE: u16 = 0x70;

#[test]
fn the_hook_starts_only_when_a_mode_needs_it_and_stops_once_idle() {
    let mut rig = Rig::new();
    rig.begin();
    rig.send(Msg::Latches(Vec::new()));
    assert_eq!(rig.hook.starts(), 0, "nothing needs the hook yet");
    rig.toggle(Toggle::RightClick);
    assert!(rig.hook.running());
    rig.settle(T0);
    rig.toggle(Toggle::RightClick);
    assert!(rig.hook.running(), "the command waits for the engine");
    assert_eq!(
        rig.hook.wakes(),
        2,
        "one wake at the start and one for the command"
    );
    rig.settle(T0);
    assert!(!rig.hook.running());
    assert_eq!((rig.hook.starts(), rig.hook.stops()), (1, 1));
    let told = [
        shown(false, Off),
        Heard::AskLatches,
        Heard::AskScroll,
        shown(true, Off),
        shown(false, Off),
    ];
    assert_eq!(rig.take(), told);
    assert_eq!(rig.hook.config(), Some(rig.settings.hook_config()));
}

#[test]
fn a_right_click_toggle_is_saved_and_a_restart_brings_it_back() {
    let mut rig = Rig::new();
    rig.begin();
    rig.toggle(Toggle::RightClick);
    rig.settle(T0);
    let saved: MouseSettings = rig.host.sink.read_as().unwrap();
    assert!(saved.rclick);
    assert_eq!(rig.host.sink.writes().len(), 1);
    let mut again = Rig::with(|host| host.put(RCLICK, saved.rclick.into()));
    again.begin();
    assert!(again.hook.running(), "the hook starts at once");
    let told = [shown(true, Off), Heard::AskLatches, Heard::AskScroll];
    assert_eq!(again.take(), told);
    assert!(again.host.sink.writes().is_empty());
}

#[test]
fn grab_picks_up_at_once_drops_on_the_next_click_and_right_click_comes_back() {
    let mut rig = Rig::saved_rclick();
    rig.begin();
    rig.take();
    rig.toggle(Toggle::Grab);
    rig.settle(T0);
    assert!(
        !rig.feed(down(A), T1),
        "the press goes through as the pick-up"
    );
    assert!(rig.feed(up(A), T1), "its release waits for the drop");
    assert!(rig.feed(down(B), T2), "the drop click");
    let told = [
        shown(false, Armed),
        Heard::Note(AssistNote::Picked { at: A }),
        shown(false, Carrying),
        Heard::Note(AssistNote::Dropped { at: B }),
        shown(true, Off),
    ];
    assert_eq!(rig.take(), told);
    assert!(rig.hook.sent().contains(&PointerAct::Up(Button::Left, B)));
    assert!(
        rig.host.sink.writes().is_empty(),
        "Grab never changes the saved Right-click"
    );
}

#[test]
fn grab_alone_stops_the_hook_once_the_drop_click_is_released() {
    let mut rig = Rig::new();
    rig.begin();
    rig.toggle(Toggle::Grab);
    rig.settle(T0);
    let _ = rig.feed(down(A), T1);
    let _ = rig.feed(up(A), T1);
    let _ = rig.feed(down(B), T2);
    assert!(
        rig.hook.running(),
        "the drop click's release is still to come"
    );
    let _ = rig.feed(up(B), T2);
    assert!(!rig.hook.running());
    assert_eq!(rig.hook.stops(), 1);
}

#[test]
fn latches_start_the_hook_and_go_around_the_next_click() {
    let mut rig = Rig::new();
    rig.begin();
    rig.take();
    rig.send(Msg::Latches(vec![SHIFT]));
    assert!(rig.hook.running(), "a latch needs the hook with no mode on");
    assert!(rig.feed(down(A), T1));
    assert!(rig.feed(up(A), T1));
    let around = [
        PointerAct::KeyDown(SHIFT),
        PointerAct::Down(Button::Left, A),
        PointerAct::Up(Button::Left, A),
        PointerAct::KeyUp(SHIFT),
    ];
    assert_eq!(rig.hook.take_sent(), around);
    assert_eq!(rig.take(), [Heard::Used]);
    rig.send(Msg::Latches(Vec::new()));
    rig.settle(T1);
    assert!(
        !rig.hook.running(),
        "nothing needs it once the latches clear"
    );
}

#[test]
fn an_ask_publishes_the_state_again_so_either_start_order_ends_right() {
    let mut rig = Rig::new();
    rig.begin();
    let first = [shown(false, Off), Heard::AskLatches, Heard::AskScroll];
    assert_eq!(
        rig.take(),
        first,
        "a keyboard that started first hears the state"
    );
    rig.toggle(Toggle::RightClick);
    rig.settle(T0);
    rig.take();
    rig.send(Msg::Ask);
    assert_eq!(
        rig.take(),
        [shown(true, Off)],
        "a keyboard that starts later asks"
    );
}

#[test]
fn two_moves_before_a_publish_give_one_pointer_moved_at_the_latest_point() {
    let mut rig = Rig::saved_rclick();
    rig.begin();
    rig.take();
    let _ = rig.hook.feed(moved(A), T1);
    let _ = rig.hook.feed(moved(B), T1);
    rig.pump();
    assert_eq!(rig.take(), [Heard::Moved(PointerMoved { at: B })]);
    let _ = rig.feed(moved(C), T2);
    assert_eq!(rig.take(), [Heard::Moved(PointerMoved { at: C })]);
}

#[test]
fn hold_notes_carry_the_ring_and_hold_times_of_the_settings() {
    let mut rig = Rig::saved_rclick();
    rig.begin();
    rig.take();
    let s = rig.settings.clone();
    let _ = rig.feed(down(A), T0);
    rig.settle(due(&s, T0));
    let _ = rig.feed(up(A), due(&s, T0));
    let (t, hold_ms, ring_after_ms) = (due(&s, T1), s.hold_ms, s.ring_after_ms);
    let _ = rig.feed(down(B), t);
    let _ = rig.feed(up(B), t);
    let start = |at| AssistNote::HoldStart {
        at,
        ring_after_ms,
        hold_ms,
    };
    let told = [
        start(A),
        AssistNote::Fired { at: A },
        start(B),
        AssistNote::HoldEnd,
    ];
    assert_eq!(rig.take(), told.map(Heard::Note));
}

#[test]
fn esc_during_a_carry_is_a_cancelled_note() {
    let mut rig = Rig::new();
    rig.begin();
    rig.toggle(Toggle::Grab);
    rig.settle(T0);
    let _ = rig.feed(down(A), T1);
    rig.take();
    assert!(!rig.feed(PointerEvent::Esc, T2), "the Esc reaches the app");
    let told = [Heard::Note(AssistNote::Cancelled), shown(false, Off)];
    assert_eq!(rig.take(), told);
}

#[test]
fn a_hold_that_came_due_fires_before_a_queued_command() {
    let mut rig = Rig::saved_rclick();
    rig.begin();
    let _ = rig.feed(down(A), T0);
    rig.take();
    rig.actor.handle(Msg::Toggle(Toggle::Grab));
    rig.settle(due(&rig.settings.clone(), T0));
    let click = [
        PointerAct::Down(Button::Right, A),
        PointerAct::Up(Button::Right, A),
    ];
    assert_eq!(
        rig.hook.take_sent(),
        click,
        "a right-click, not the press let through"
    );
    let told = [
        Heard::Note(AssistNote::Fired { at: A }),
        shown(false, Armed),
    ];
    assert_eq!(rig.take(), told);
}

#[test]
fn a_settle_that_fires_a_hold_and_takes_a_burst_of_toggles_stays_within_its_room() {
    let mut rig = Rig::saved_rclick();
    rig.begin();
    let every: Vec<Scan> = (MOD_BASE..).take(MAX_LATCHES).map(Scan::new).collect();
    rig.send(Msg::Latches(every));
    rig.settle(T0);
    let _ = rig.feed(down(A), T0);
    for which in [Toggle::Grab, Toggle::Grab, Toggle::RightClick, Toggle::Grab] {
        rig.actor.handle(Msg::Toggle(which));
    }
    // The fake panics when one call pushes past the engine's `max_acts`.
    rig.settle(due(&rig.settings.clone(), T0));
    assert!(
        rig.hook
            .sent()
            .contains(&PointerAct::Down(Button::Right, A))
    );
}

#[test]
fn an_idle_report_older_than_a_queued_command_keeps_the_hook() {
    let mut rig = Rig::saved_rclick();
    rig.begin();
    rig.actor.handle(Msg::Toggle(Toggle::RightClick));
    rig.hook.settle(T0);
    // The engine went idle, but the user turned Right-click on again before the actor read that.
    rig.actor.handle(Msg::Toggle(Toggle::RightClick));
    rig.pump();
    assert!(
        rig.hook.running(),
        "the second toggle still waits in the queue"
    );
    rig.settle(T1);
    assert!(rig.hook.running());
    assert_eq!(rig.take().last(), Some(&shown(true, Off)));
}
