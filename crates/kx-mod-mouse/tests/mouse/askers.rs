//! Presses per asker (ADR-0041 decision 6): each asker's wish stands apart, so presses and releases are told until
//! every asker and the panel stop, and each release is told with `ours` and no point.

use crate::live::{Live, WAIT, hear, on, synced};
use crate::rig::{A, Heard, Rig, T0, T1, T2, down, ours, up, up_ours, wants};
use kx_mod_mouse::Msg;
use kx_module_api::ports::PressAsker::{self, Predict, Selection};
use kx_module_api::ports::{AssistChanged, PointerPressed, PointerReleased};
use kx_module_api::{Bus, Mono};

/// A press told on the bus.
fn pressed(ours: bool) -> Heard {
    Heard::Pressed(PointerPressed { ours })
}

/// A release told on the bus.
fn released(ours: bool) -> Heard {
    Heard::Released(PointerReleased { ours })
}

/// `asker`'s wish, as the actor gets it.
fn wish(asker: PressAsker, on: bool) -> Msg {
    Msg::Presses(wants(asker, on))
}

/// A started rig with what it said so far taken.
fn started() -> Rig {
    let mut rig = Rig::new();
    rig.begin();
    rig.take();
    rig
}

/// Whether a click outside at `now` is told, its press then its release, as both reach the app.
fn told(rig: &mut Rig, now: Mono) -> bool {
    assert!(!rig.feed(down(A), now), "the press reaches the app");
    assert!(!rig.feed(up(A), now), "the release reaches the app");
    rig.take() == [pressed(false), released(false)]
}

#[test]
fn two_askers_keep_presses_told_until_both_stop_in_either_order() {
    for (first, second) in [(Predict, Selection), (Selection, Predict)] {
        let mut rig = started();
        rig.send(wish(first, true));
        rig.send(wish(second, true));
        rig.send(wish(first, false));
        rig.settle(T0);
        assert!(rig.hook.running(), "{second:?} still wants them");
        assert!(told(&mut rig, T1), "{second:?}");
        rig.send(wish(second, false));
        rig.settle(T2);
        assert!(!rig.hook.running(), "no asker wants them now");
        assert_eq!(rig.hook.starts(), 1, "{first:?} then {second:?}");
    }
}

#[test]
fn a_repeated_wish_changes_nothing_and_one_askers_off_never_cancels_another() {
    let mut rig = started();
    rig.send(wish(Selection, true));
    rig.send(wish(Selection, true));
    rig.send(wish(Predict, false));
    rig.send(wish(Predict, false));
    rig.settle(T0);
    assert_eq!(rig.hook.starts(), 1);
    assert!(
        told(&mut rig, T1),
        "prediction never asked, the selection does"
    );
    rig.send(wish(Selection, false));
    rig.settle(T2);
    assert!(!rig.hook.running(), "one off ends a wish told twice");
}

#[test]
fn each_release_while_wanted_is_told_with_ours_and_reaches_the_app() {
    let mut rig = started();
    rig.send(wish(Selection, true));
    rig.settle(T0);
    for (press, lift) in [(down(A), up(A)), (ours(A), up_ours(A))] {
        assert!(!rig.feed(press, T1));
        assert!(!rig.feed(lift, T1), "the release passes");
    }
    let want = [
        pressed(false),
        released(false),
        pressed(true),
        released(true),
    ];
    assert_eq!(rig.take(), want);
    assert!(rig.hook.take_sent().is_empty(), "nothing of ours is sent");
}

#[test]
fn the_pad_alone_tells_no_release_and_the_panels_wish_tells_both() {
    let mut rig = started();
    rig.send(Msg::Scroll(true));
    rig.settle(T0);
    assert!(!rig.feed(down(A), T0));
    assert!(!rig.feed(up(A), T0));
    assert_eq!(rig.take(), [pressed(false)], "scrolling tells only presses");
    rig.send(Msg::PanelPresses(true));
    rig.send(Msg::Scroll(false));
    rig.settle(T1);
    assert!(
        told(&mut rig, T2),
        "the panel wants presses, so releases are told"
    );
}

/// The real relay: each asker's wish reaches the actor on its own, and a release reaches the bus.
#[test]
fn each_askers_wish_is_heard_on_the_bus_and_a_release_is_told_there() {
    let listen = |bus: &Bus| (hear::<AssistChanged>(bus), hear::<PointerReleased>(bus));
    let (mut live, ((_e1, states), (_e2, releases))) = Live::start(|_| {}, listen);
    assert_eq!(states.recv_timeout(WAIT), Ok(on(false)));
    for (asker, on) in [(Selection, true), (Predict, true), (Selection, false)] {
        live.bus.publish(&wants(asker, on)).unwrap();
    }
    synced(&live, &states);
    live.hook.settle(T0);
    synced(&live, &states);
    assert!(live.hook.running(), "prediction still wants them");
    assert!(!live.hook.feed(up(A), T1), "the release passes");
    let away = PointerReleased { ours: false };
    assert_eq!(releases.recv_timeout(WAIT), Ok(away));
    live.bus.publish(&wants(Predict, false)).unwrap();
    synced(&live, &states);
    live.hook.settle(T2);
    synced(&live, &states);
    assert!(!live.hook.running(), "no asker wants them now");
    live.stop();
}
