//! The Power key from the keyboard's side, over the bus (spec §8.5): it asks the power module for the dialog while that
//! module is live, and is dimmed with a tip, doing nothing, while it is not.

#![allow(
    clippy::unwrap_used,
    reason = "these fixtures run only in tests, where a failed setup should fail the test"
)]

use crate::assist::{Bench, ear};
use crate::rig::STATE;
use kx_mod_keyboard::view::POWER_OFF_TIP;
use kx_module_api::ports::{FaceAction, FaceReply, PowerAsk, PowerAsked, PowerChanged};
use kx_module_api::{Bus, Subscription};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex, PoisonError};

/// The Power key.
const KEY: &str = "power";

/// What the keyboard said to the power module.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Heard {
    /// It asked for the module's state.
    Ask,
    /// The Power key was clicked.
    Asked,
}

/// The module on a bus the test listens to, with what it said to the power module.
struct Power {
    b: Bench,
    heard: Arc<Mutex<Vec<Heard>>>,
}

impl Power {
    /// The module with `power`'s handlers on the bus before it starts, as a power module that already runs.
    fn after(power: impl FnOnce(&Bus) -> Vec<Subscription>) -> Self {
        let heard = Arc::default();
        let h = Arc::clone(&heard);
        let b = Bench::after(None, move |bus| {
            let mut ears = vec![
                ear(bus, &h, |_: &PowerAsk| Heard::Ask),
                ear(bus, &h, |_: &PowerAsked| Heard::Asked),
            ];
            ears.extend(power(bus));
            ears
        });
        Self { b, heard }
    }

    /// The module started, with what it said at start already taken.
    fn quiet() -> Self {
        let p = Self::after(|_| Vec::new());
        p.heard();
        p
    }

    /// What the keyboard said to the power module since the last look.
    fn heard(&self) -> Vec<Heard> {
        let mut heard = self.heard.lock().unwrap_or_else(PoisonError::into_inner);
        std::mem::take(&mut *heard)
    }

    /// The power module says it runs, or not.
    fn live(&self, live: bool) {
        self.b.send(&PowerChanged { live });
    }

    /// The Power key's dim flag, tip and fill.
    fn key(&self) -> Value {
        let k = self.b.key(KEY);
        json!({ "dim": k["dim"], "tip": k["tip"], "fill": k["paint"]["fill"], "on": k["on"] })
    }
}

/// The Power key while the power module is not running.
fn off() -> Value {
    json!({ "dim": true, "tip": POWER_OFF_TIP, "fill": "dim", "on": false })
}

/// The Power key while the power module runs.
fn on() -> Value {
    json!({ "dim": false, "tip": null, "fill": "key_act", "on": false })
}

#[test]
fn start_and_restart_ask_the_power_module_for_its_state() {
    let mut p = Power::after(|_| Vec::new());
    assert_eq!(p.heard(), [Heard::Ask]);
    p.b.rig.restart(Some(("lang_key", "cycle".into())));
    assert_eq!(p.heard(), [Heard::Ask]);
}

/// The keyboard subscribes before it asks, so a power module that answers at once is heard.
#[test]
fn a_power_module_already_running_lights_the_key_at_start() {
    let p = Power::after(|bus| {
        let reply = bus.clone();
        let answer = move |_: &PowerAsk| reply.publish(&PowerChanged { live: true }).unwrap();
        vec![bus.subscribe(answer).unwrap()]
    });
    assert_eq!(p.key(), on());
}

/// Plan edge case 12: the power module is off.
#[test]
fn before_the_power_module_speaks_the_key_is_dim_says_so_and_does_nothing() {
    let p = Power::quiet();
    assert_eq!(p.key(), off());
    assert_eq!(p.b.rig.tap(KEY), Ok(FaceReply::default()));
    assert!(p.heard().is_empty());
    assert!(p.b.rig.fakes.injector.strokes().is_empty());
}

#[test]
fn a_live_power_module_hears_one_ask_per_click() {
    let p = Power::quiet();
    p.live(true);
    assert_eq!(p.key(), on());
    for _ in 0..2 {
        assert_eq!(p.b.rig.tap(KEY), Ok(FaceReply::default()));
        assert_eq!(p.heard(), [Heard::Asked]);
    }
}

#[test]
fn a_power_module_that_stops_dims_the_key_again_and_its_clicks_do_nothing() {
    let p = Power::quiet();
    p.live(true);
    p.b.send(&PowerChanged::OFF);
    assert_eq!(p.key(), off());
    p.b.rig.tap(KEY).unwrap();
    assert!(p.heard().is_empty());
}

#[test]
fn a_power_click_sends_nothing_to_the_app_and_keeps_the_latches() {
    let p = Power::quiet();
    p.live(true);
    p.b.rig.tap("shift").unwrap();
    p.b.rig.tap(KEY).unwrap();
    assert_eq!(p.b.key("shift")["on"], true);
    assert!(p.b.rig.fakes.injector.strokes().is_empty());
    assert!(p.b.rig.fakes.shortcuts.sent().is_empty());
}

#[test]
fn a_power_change_redraws_at_each_relabel_until_the_ui_reads_the_state() {
    let p = Power::quiet();
    let relabel = || p.b.rig.act(FaceAction::Relabel).unwrap();
    assert_eq!(relabel(), FaceReply::default());
    p.live(true);
    assert_eq!(relabel(), STATE, "the key wakes up");
    assert_eq!(relabel(), STATE, "until the UI reads the state");
    p.b.rig.state();
    assert_eq!(relabel(), FaceReply::default());
    p.live(true);
    assert_eq!(relabel(), FaceReply::default(), "the same state again");
}

/// The power module may have stopped while the keyboard was down, so its key dims until its answer lights it again.
#[test]
fn a_keyboard_restart_dims_the_power_key_until_the_power_module_answers() {
    let mut p = Power::quiet();
    p.live(true);
    p.b.rig.restart(Some(("lang_key", "cycle".into())));
    assert_eq!(p.key(), off(), "no answer yet");
    p.live(true);
    assert_eq!(p.key(), on());
}
