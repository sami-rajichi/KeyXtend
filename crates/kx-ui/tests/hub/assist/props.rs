//! Properties of the overlay state over random mouse events, scroll changes, pointer sizes and QML reads: idle shows
//! nothing, a ring needs a mode, the scroll part shows while the mouse module runs, the seq never falls, and each
//! change after a read sends one `Wake::Assist`, never two before the next read.

use super::{heard, woke};
use crate::Rig;
use kx_module_api::Bus;
use kx_module_api::ports::{
    AssistChanged, AssistNote, Dir, GrabState, Point, PointerMoved, ScrollChanged,
};
use kx_platform::ui::SysLook;
use kx_ui::Wake;
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::sample::select;
use serde_json::{Value, json};

/// Random runs per property.
const CASES: u32 = 256;
/// Most steps in one run.
const MAX_STEPS: usize = 40;
/// Farthest a point goes on each axis, in pixels; small, so points repeat.
const REACH: i32 = 4;
/// Longest ring wait and hold, in ms; the hold may be shorter than the wait.
const MAX_MS: u32 = 2_000;

/// One random step.
#[derive(Clone, Debug)]
enum Step {
    /// The mouse module's state.
    Changed(AssistChanged),
    /// The scroll module's state.
    Scrolled(ScrollChanged),
    /// The pointer moved.
    Moved(Point),
    /// A hold's note.
    Note(AssistNote),
    /// The system look, with this pointer size.
    Sys(u32),
    /// QML reads the state.
    Read,
}

fn point() -> impl Strategy<Value = Point> {
    (0..=REACH, 0..=REACH).prop_map(|(x, y)| Point { x, y })
}

fn note() -> impl Strategy<Value = AssistNote> {
    prop_oneof![
        (point(), 0..=MAX_MS, 0..=MAX_MS).prop_map(|(at, ring_after_ms, hold_ms)| {
            AssistNote::HoldStart {
                at,
                ring_after_ms,
                hold_ms,
            }
        }),
        point().prop_map(|at| AssistNote::Fired { at }),
        Just(AssistNote::HoldEnd),
        Just(AssistNote::Cancelled),
        point().prop_map(|at| AssistNote::Picked { at }),
        point().prop_map(|at| AssistNote::Dropped { at }),
    ]
}

fn scrolled() -> impl Strategy<Value = ScrollChanged> {
    let dir = prop_oneof![Just(None), select(Dir::ALL.to_vec()).prop_map(Some)];
    (any::<bool>(), dir, any::<bool>()).prop_map(|(live, dir, fast)| ScrollChanged {
        live,
        dir,
        fast,
    })
}

fn step() -> impl Strategy<Value = Step> {
    let grab = select(vec![GrabState::Off, GrabState::Armed, GrabState::Carrying]);
    let px = SysLook::POINTER_PX;
    prop_oneof![
        2 => (any::<bool>(), any::<bool>(), grab).prop_map(|(live, rclick, grab)| {
            Step::Changed(AssistChanged { live, rclick, grab })
        }),
        2 => scrolled().prop_map(Step::Scrolled),
        3 => point().prop_map(Step::Moved),
        3 => note().prop_map(Step::Note),
        1 => select(vec![px, px * 2]).prop_map(Step::Sys),
        3 => Just(Step::Read),
    ]
}

/// The mode the hub must name for state `s`: Grab before Right-click, none while the module is not live.
fn mode_of(s: &Value) -> Value {
    match (s["live"] == true, s["grab"] != "off", s["rclick"] == true) {
        (true, true, _) => json!("grab"),
        (true, false, true) => json!("rclick"),
        _ => Value::Null,
    }
}

/// The scroll part the hub must show for state `s` after scroll change `last`: the way and speed while scrolling and
/// the mouse module runs, else none.
fn scroll_of(s: &Value, last: Option<ScrollChanged>) -> Value {
    match last {
        Some(ScrollChanged {
            live: true,
            dir: Some(dir),
            fast,
        }) if s["live"] == true => json!({ "dir": dir, "fast": fast }),
        _ => Value::Null,
    }
}

/// Plays `step` on the rig.
fn play(rig: &Rig, bus: &Bus, step: &Step) {
    match step {
        Step::Changed(now) => bus.publish(now).unwrap(),
        Step::Scrolled(now) => bus.publish(now).unwrap(),
        Step::Moved(at) => bus.publish(&PointerMoved { at: *at }).unwrap(),
        Step::Note(note) => bus.publish(note).unwrap(),
        Step::Sys(pointer_px) => rig.hub.set_sys(SysLook {
            pointer_px: *pointer_px,
            ..SysLook::default()
        }),
        Step::Read => {}
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn idle_shows_nothing_the_seq_grows_and_one_wake_waits_per_read(
        steps in vec(step(), 1..=MAX_STEPS),
    ) {
        let (rig, bus, _subs) = heard();
        let mut last: Value = serde_json::from_str(&rig.hub.assist_json()).unwrap();
        rig.wakes();
        let (mut woken, mut scrolled) = (0, None);
        let end = [Step::Read];
        for step in steps.iter().chain(&end) {
            play(&rig, &bus, step);
            if let Step::Scrolled(now) = step {
                scrolled = Some(*now);
            }
            woken += woke(&rig, Wake::Assist);
            prop_assert!(woken <= 1, "two wakes before a read: {steps:?}");
            if !matches!(step, Step::Read) {
                continue;
            }
            let s: Value = serde_json::from_str(&rig.hub.assist_json()).unwrap();
            prop_assert_eq!(&s["mode"], &mode_of(&s));
            prop_assert_eq!(&s["scroll"], &scroll_of(&s, scrolled));
            if s["mode"].is_null() {
                let gone = ["ring", "burst"].map(|k| s[k].is_null());
                prop_assert_eq!(gone, [true; 2], "a ring needs a mode");
            }
            if s["mode"].is_null() && s["scroll"].is_null() {
                prop_assert!(s["point"].is_null(), "idle shows nothing");
            }
            prop_assert!(s["seq"].as_u64() >= last["seq"].as_u64(), "the seq fell");
            let changed = s != last;
            prop_assert_eq!(woken, usize::from(changed), "one wake per changed read");
            (last, woken) = (s, 0);
        }
    }
}
