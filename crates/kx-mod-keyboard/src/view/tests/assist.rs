//! The mouse-assist side keys in the state: dimmed with a tip until the mouse module runs, then lit by their mode.

use super::*;
use kx_module_api::ports::GrabState;

/// The mouse module running with Right-click `rclick` and Grab at `grab`.
fn live(rclick: bool, grab: GrabState) -> AssistChanged {
    AssistChanged {
        live: true,
        rclick,
        grab,
    }
}

fn with(assist: AssistChanged) -> StateView {
    state_of(&none(), false, None, assist, ScrollChanged::OFF)
}

#[test]
fn the_board_never_dims_the_mouse_assist_keys_and_makes_them_checkable() {
    let v = board_view(&spec(), &settings(), 1.0);
    for id in ["rclick", "grab"] {
        let k = key(&v, id);
        assert!(!k.dim && k.tip.is_none() && k.side, "{id}");
        assert!(k.checkable && !k.repeat, "{id}");
    }
}

#[test]
fn until_the_mouse_module_runs_both_keys_are_dim_and_say_it_is_off() {
    let stopped = AssistChanged {
        live: false,
        ..live(true, GrabState::Carrying)
    };
    for s in [with(AssistChanged::OFF), with(stopped)] {
        for id in ["rclick", "grab"] {
            let k = &s.keys[id];
            assert_eq!((k.paint, k.tip, k.on), (DIM, Some(MOUSE_OFF_TIP), false));
            assert!(k.dim, "{id} does nothing");
        }
    }
}

#[test]
fn a_running_mouse_module_lights_each_key_by_its_mode_with_its_own_tip() {
    let s = with(live(true, GrabState::Off));
    let (rclick, grab) = (&s.keys["rclick"], &s.keys["grab"]);
    assert_eq!((rclick.on, rclick.tip), (true, Some(RCLICK_TIP)));
    assert_eq!(rclick.paint, paint(KeyKind::Act, true));
    assert_eq!((grab.on, grab.tip), (false, Some(GRAB_TIP)));
    assert_eq!(grab.paint, paint(KeyKind::Act, false));
    assert!(!rclick.dim && !grab.dim, "both act");
    for g in [GrabState::Armed, GrabState::Carrying] {
        let s = with(live(false, g));
        assert!(s.keys["grab"].on && !s.keys["rclick"].on, "{g:?}");
    }
}

#[test]
fn the_other_side_keys_stay_dim_with_no_tip_whatever_the_mouse_module_says() {
    let s = with(live(true, GrabState::Armed));
    let mic = &s.keys["mic"];
    assert_eq!((mic.paint, mic.tip, mic.dim), (DIM, None, true));
}
