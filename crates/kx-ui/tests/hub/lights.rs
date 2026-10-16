//! The keyboard's wake from the bus: a mouse mode change, a used latch or a scroll change wakes it once, only while a
//! face is attached, and only after the face's own handler ran, so the window reads the new state.

use crate::assist::{RCLICK, heard, woke};
use crate::scroll::DOWN;
use crate::{FakeFace, Rig};
use kx_module_api::ports::{AssistChanged, LatchesUsed, ScrollChanged};
use kx_module_api::{Bus, Event, Subscription};
use kx_test_support::host::FakeHost;
use kx_ui::{MANIFEST, Wake};
use std::sync::{Arc, Mutex};

/// Attaches a fresh face to `rig`, as the `ui` module's start does; the host holds its settings.
fn attach(rig: &Rig) -> FakeHost {
    let mut host = FakeHost::new(&MANIFEST);
    rig.attach(Arc::new(FakeFace::default()), &mut host);
    host
}

/// Sends a mode change, a used latch and a scroll change on `bus`.
fn all(bus: &Bus) {
    bus.publish(&RCLICK).unwrap();
    bus.publish(&LatchesUsed).unwrap();
    bus.publish(&DOWN).unwrap();
}

/// A stand-in for the face's handler of `E`, subscribed now as the keyboard's start does: each run notes whether the
/// keyboard was woken already.
fn face_on<E: Event>(rig: &Rig, bus: &Bus, early: &Arc<Mutex<Vec<bool>>>) -> Subscription {
    let (wakes, early) = (Arc::clone(&rig.wakes), Arc::clone(early));
    let note = move |_: &E| {
        let woken = wakes.lock().unwrap().contains(&Wake::Keys);
        early.lock().unwrap().push(woken);
    };
    bus.subscribe(note).unwrap()
}

#[test]
fn the_keyboard_wakes_only_after_the_faces_own_handler_ran() {
    let (rig, bus, _subs) = heard();
    let early = Arc::new(Mutex::new(Vec::new()));
    let _face = [
        face_on::<AssistChanged>(&rig, &bus, &early),
        face_on::<LatchesUsed>(&rig, &bus, &early),
        face_on::<ScrollChanged>(&rig, &bus, &early),
    ];
    let _host = attach(&rig);
    rig.wakes();
    bus.publish(&RCLICK).unwrap();
    assert_eq!(woke(&rig, Wake::Keys), 1);
    bus.publish(&LatchesUsed).unwrap();
    assert_eq!(woke(&rig, Wake::Keys), 1);
    bus.publish(&DOWN).unwrap();
    assert_eq!(woke(&rig, Wake::Keys), 1, "the pad lights at once");
    assert_eq!(*early.lock().unwrap(), [false; 3], "the face ran first");
}

#[test]
fn the_keyboard_wakes_once_per_change_and_only_while_a_face_is_attached() {
    let (rig, bus, _subs) = heard();
    all(&bus);
    assert_eq!(woke(&rig, Wake::Keys), 0, "no face yet");
    let _host = attach(&rig);
    let _again = attach(&rig);
    all(&bus);
    assert_eq!(
        woke(&rig, Wake::Keys),
        3,
        "one each, after a second attach too"
    );
    rig.hub.detach();
    all(&bus);
    assert_eq!(woke(&rig, Wake::Keys), 0, "the face went");
}
