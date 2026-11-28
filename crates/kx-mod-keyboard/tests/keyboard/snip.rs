//! Snip from the keyboard's side, over the bus (ADR-0056): the Snip key left `later`, asks the snip module to start
//! or end a capture with `SnipStart`, keeping every latch, and lights while `SnipLit` says a capture runs; at each
//! start the keyboard asks for the light and the line with `SnipAsk`.

#![allow(
    clippy::unwrap_used,
    reason = "these fixtures run only in tests, where a failed setup should fail the test"
)]

mod beside;
mod bubble;
mod lines;

use crate::assist::{Bench, ear};
use crate::rig::STATE;
use kx_mod_keyboard::BoardSpec;
use kx_module_api::ports::{FaceAction, FaceReply, SnipAsk, SnipLit, SnipStart};
use kx_module_api::{Bus, Subscription};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex, PoisonError};

/// The Snip key.
const SNIP: &str = "snip";

/// What the snip module heard from the keyboard.
#[derive(Debug, PartialEq, Eq)]
enum Heard {
    /// A click on the Snip key.
    Start,
    /// An ask for the light and the line.
    Ask,
}

/// The module on a bus the test listens to, with what the snip module heard.
struct Snips {
    b: Bench,
    heard: Arc<Mutex<Vec<Heard>>>,
}

impl Snips {
    /// The module, heard from before it starts, with `snip`'s handlers on the bus first, as a snip module that runs.
    fn after(snip: impl FnOnce(&Bus) -> Vec<Subscription>) -> Self {
        let heard = Arc::default();
        let h = Arc::clone(&heard);
        let b = Bench::after(None, move |bus| {
            let mut ears = vec![
                ear(bus, &h, |_: &SnipStart| Heard::Start),
                ear(bus, &h, |_: &SnipAsk| Heard::Ask),
            ];
            ears.extend(snip(bus));
            ears
        });
        b.said();
        Self { b, heard }
    }

    /// The module, with what it said at start already taken.
    fn quiet() -> Self {
        let s = Self::after(|_| Vec::new());
        s.heard();
        s
    }

    /// What the snip module heard since the last look.
    fn heard(&self) -> Vec<Heard> {
        let mut heard = self.heard.lock().unwrap_or_else(PoisonError::into_inner);
        std::mem::take(&mut *heard)
    }

    /// The Snip key's light: its flags, the Dictation key's light field and its paint.
    fn light(&self) -> Value {
        let k = self.b.key(SNIP);
        let p = &k["paint"];
        let paint = [&p["fill"], &p["ink"], &p["skirt"]];
        json!({ "on": k["on"], "dim": k["dim"], "lit": k["lit"], "paint": paint })
    }
}

/// The key dark: a side key's own paint.
fn dark() -> Value {
    json!({ "on": false, "dim": false, "lit": "off", "paint": ["key_act", "legend_act", "key_mod"] })
}

/// The key lit while a capture runs: a turned-on side key's paint, never the Dictation key's red.
fn lit() -> Value {
    json!({ "on": true, "dim": false, "lit": "off", "paint": ["on", "on_ink", "on"] })
}

#[test]
fn the_snip_key_leaves_later_for_its_own_role() {
    let spec = BoardSpec::shipped().unwrap();
    let snip = spec.side.iter().find(|k| k.id == SNIP).unwrap();
    assert_eq!(serde_json::to_value(snip.role).unwrap(), "snip");
}

#[test]
fn the_board_draws_the_snip_key_live_and_checkable() {
    let s = Snips::quiet();
    let board = s.b.rig.board();
    let keys = board["keys"].as_array().unwrap();
    let k = keys.iter().find(|k| k["id"] == SNIP).unwrap();
    let got = json!([k["dim"], k["checkable"], k["tip"], k["repeat"]]);
    assert_eq!(got, json!([false, true, null, false]));
}

#[test]
fn a_click_tells_snip_start_sends_nothing_and_keeps_every_latch() {
    let s = Snips::quiet();
    s.b.rig.tap("shift").unwrap();
    s.b.said();
    assert_eq!(s.b.rig.tap(SNIP), Ok(STATE));
    assert_eq!(s.heard(), [Heard::Start]);
    assert!(s.b.rig.fakes.injector.strokes().is_empty(), "nothing typed");
    assert!(s.b.said().is_empty(), "the latched set never changed");
    assert_eq!(s.b.key("shift")["on"], true);
}

#[test]
fn snip_lit_lights_the_key_and_turns_it_dark_at_the_next_relabel() {
    let s = Snips::quiet();
    s.b.rig.state();
    s.b.send(&SnipLit::On);
    assert_eq!(s.b.rig.act(FaceAction::Relabel), Ok(STATE));
    assert_eq!(s.light(), lit());
    s.b.send(&SnipLit::On);
    let again = s.b.rig.act(FaceAction::Relabel);
    assert_eq!(again, Ok(FaceReply::default()), "no change, no redraw");
    s.b.send(&SnipLit::Off);
    assert_eq!(s.b.rig.act(FaceAction::Relabel), Ok(STATE));
    assert_eq!(s.light(), dark());
}

#[test]
fn a_new_light_is_in_the_state_at_once_and_only_on_the_snip_key() {
    let s = Snips::quiet();
    s.b.send(&SnipLit::On);
    assert_eq!(s.light(), lit(), "with no relabel");
    let state = s.b.rig.state();
    let keys = state["keys"].as_object().unwrap();
    let on: Vec<&String> = keys
        .iter()
        .filter(|(_, k)| k["on"] == true)
        .map(|(id, _)| id)
        .collect();
    assert_eq!(on, [SNIP]);
}

#[test]
fn the_key_starts_dark_and_a_restart_forgets_its_light() {
    let mut s = Snips::quiet();
    assert_eq!(s.light(), dark());
    s.b.send(&SnipLit::On);
    s.b.rig.restart(None);
    assert_eq!(s.light(), dark(), "unheard since, so dark");
}

#[test]
fn the_start_asks_the_snip_module_for_its_light_and_line() {
    let s = Snips::after(|_| Vec::new());
    assert_eq!(s.heard(), [Heard::Ask]);
}

/// The keyboard subscribes before it asks, so a snip module that answers at once is heard.
#[test]
fn a_restart_during_a_capture_asks_again_and_the_key_stays_lit() {
    let mut s = Snips::after(|bus| {
        let reply = bus.clone();
        let answer = move |_: &SnipAsk| reply.publish(&SnipLit::On).unwrap();
        vec![bus.subscribe(answer).unwrap()]
    });
    assert_eq!(s.heard(), [Heard::Ask]);
    assert_eq!(
        s.light(),
        lit(),
        "a capture ran before the keyboard started"
    );
    s.b.rig.restart(None);
    assert_eq!(s.heard(), [Heard::Ask], "asked again after forgetting");
    assert_eq!(s.light(), lit(), "lit until the capture ends");
}
