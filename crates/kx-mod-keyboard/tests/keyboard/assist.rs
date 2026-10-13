//! Mouse assist from the keyboard's side, over the bus: Right-click and Grab light from `AssistChanged`, dim while the
//! mouse module is not live and ask it to flip a mode; a bus change redraws at the next relabel.

#![allow(
    clippy::unwrap_used,
    reason = "these fixtures run only in tests, where a failed setup should fail the test"
)]

mod latches;

use crate::host::{Fakes, GOLDEN};
use crate::rig::{Rig, STATE};
use kx_mod_keyboard::view::{GRAB_TIP, LATER_TIP, MOUSE_OFF_TIP, RCLICK_TIP};
use kx_module_api::ports::{
    AssistAsk, AssistChanged, AssistToggle, FaceAction, FaceReply, GrabState, LatchesChanged, Scan,
    Toggle,
};
use kx_module_api::{Bus, BusCore, Event, Subscription};
use kx_test_support::host::FakeCore;
use serde_json::Value;
use std::sync::{Arc, Mutex, PoisonError};

/// What the keyboard said on the bus.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Said {
    /// It asked the mouse module for its state.
    Ask,
    /// It asked the mouse module to flip a mode.
    Toggle(Toggle),
    /// It told its latched set.
    Latches(Vec<Scan>),
}

/// The module on a bus the test listens to and talks on.
pub struct Bench {
    /// The started module.
    pub rig: Rig,
    /// The bus, with no limit.
    pub bus: Bus,
    said: Arc<Mutex<Vec<Said>>>,
    _ears: Vec<Subscription>,
}

/// Records each `E` on `bus` into `said` as `as_said` makes it.
fn ear<E: Event>(
    bus: &Bus,
    said: &Arc<Mutex<Vec<Said>>>,
    as_said: impl Fn(&E) -> Said + Send + Sync + 'static,
) -> Subscription {
    let said = Arc::clone(said);
    let add = move |e: &E| said.lock().unwrap().push(as_said(e));
    bus.subscribe(add).unwrap()
}

impl Bench {
    /// The module on the golden layouts with `change` over the defaults, heard from before it starts.
    pub fn new(change: Option<(&str, toml::Value)>) -> Self {
        let core: Arc<dyn BusCore> = Arc::new(FakeCore::default());
        let bus = Bus::new(Arc::clone(&core));
        let said = Arc::default();
        let ears = vec![
            ear(&bus, &said, |_: &AssistAsk| Said::Ask),
            ear(&bus, &said, |e: &AssistToggle| Said::Toggle(e.which)),
            ear(&bus, &said, |e: &LatchesChanged| {
                Said::Latches(e.held.clone())
            }),
        ];
        let rig = Rig::on_bus(Fakes::golden(&GOLDEN), change, core);
        Self {
            rig,
            bus,
            said,
            _ears: ears,
        }
    }

    /// The module started with `change`, with what it said at start already taken.
    pub fn quiet(change: Option<(&str, toml::Value)>) -> Self {
        let b = Self::new(change);
        b.said();
        b
    }

    /// What the keyboard said since the last look.
    pub fn said(&self) -> Vec<Said> {
        let mut said = self.said.lock().unwrap_or_else(PoisonError::into_inner);
        std::mem::take(&mut *said)
    }

    /// Publishes `event` as another module would.
    pub fn send<E: Event>(&self, event: &E) {
        self.bus.publish(event).unwrap();
    }

    /// The mouse module says it runs, or not, with Right-click `rclick` and Grab at `grab`.
    pub fn mouse(&self, live: bool, rclick: bool, grab: GrabState) {
        self.send(&AssistChanged { live, rclick, grab });
    }

    /// Key `id`'s state.
    pub fn key(&self, id: &str) -> Value {
        self.rig.state()["keys"][id].clone()
    }
}

/// The scan codes `codes`, as a latched set.
pub fn scans(codes: &[u16]) -> Vec<Scan> {
    codes.iter().map(|&c| Scan::new(c)).collect()
}

/// True when key state `k` shows the dim paint with the mouse-off tip.
fn off(k: &Value) -> bool {
    k["paint"]["fill"] == "dim" && k["tip"] == MOUSE_OFF_TIP && k["on"] == false
}

#[test]
fn the_board_makes_both_keys_checkable_and_never_dims_them_and_the_other_side_keys_come_later() {
    let b = Bench::quiet(None);
    let board = b.rig.board();
    let keys = board["keys"].as_array().unwrap();
    for k in keys.iter().filter(|k| k["side"] == true) {
        let assist = k["id"] == "rclick" || k["id"] == "grab";
        assert_eq!(k["checkable"], assist, "{}", k["id"]);
        assert_eq!(k["dim"], !assist, "{}", k["id"]);
        assert_eq!(k["repeat"], false, "{}", k["id"]);
        let tip = if assist {
            Value::Null
        } else {
            LATER_TIP.into()
        };
        assert_eq!(k["tip"], tip, "{}", k["id"]);
    }
}

#[test]
fn the_module_asks_the_mouse_module_for_its_state_at_start_and_after_a_restart() {
    let mut b = Bench::new(None);
    assert_eq!(b.said(), [Said::Ask]);
    b.rig.restart(Some(("lang_key", "cycle".into())));
    assert_eq!(b.said(), [Said::Ask]);
}

#[test]
fn before_the_mouse_module_speaks_both_keys_are_dim_say_so_and_do_nothing() {
    let b = Bench::quiet(None);
    for id in ["rclick", "grab"] {
        assert!(off(&b.key(id)), "{id}: {}", b.key(id));
        assert_eq!(b.rig.tap(id), Ok(FaceReply::default()), "{id}");
    }
    assert!(b.said().is_empty());
    assert!(b.rig.fakes.injector.strokes().is_empty());
}

#[test]
fn a_live_mouse_module_lights_its_modes_and_a_tap_asks_it_to_flip_one() {
    let b = Bench::quiet(None);
    b.mouse(true, true, GrabState::Off);
    let rclick = b.key("rclick");
    assert_eq!(
        (&rclick["on"], &rclick["tip"]),
        (&true.into(), &RCLICK_TIP.into())
    );
    assert_eq!(rclick["paint"]["fill"], "on");
    let grab = b.key("grab");
    assert_eq!(
        (&grab["on"], &grab["tip"]),
        (&false.into(), &GRAB_TIP.into())
    );
    assert_eq!(grab["paint"]["fill"], "key_act");
    assert_eq!(b.rig.tap("grab"), Ok(FaceReply::default()));
    assert_eq!(b.said(), [Said::Toggle(Toggle::Grab)]);
    assert_eq!(b.rig.tap("rclick"), Ok(FaceReply::default()));
    assert_eq!(b.said(), [Said::Toggle(Toggle::RightClick)]);
    assert!(b.rig.fakes.injector.strokes().is_empty());
}

#[test]
fn grab_lights_while_armed_or_carrying() {
    let b = Bench::quiet(None);
    for (grab, on) in [
        (GrabState::Armed, true),
        (GrabState::Carrying, true),
        (GrabState::Off, false),
    ] {
        b.mouse(true, false, grab);
        assert_eq!(b.key("grab")["on"], on, "{grab:?}");
        assert_eq!(b.key("rclick")["on"], false, "{grab:?}");
    }
}

#[test]
fn a_mouse_module_that_stops_dims_both_keys_again_and_their_taps_do_nothing() {
    let b = Bench::quiet(None);
    b.mouse(true, true, GrabState::Armed);
    b.mouse(false, true, GrabState::Off);
    for id in ["rclick", "grab"] {
        assert!(off(&b.key(id)), "{id}: {}", b.key(id));
        assert_eq!(b.rig.tap(id), Ok(FaceReply::default()), "{id}");
    }
    assert!(b.said().is_empty());
}

#[test]
fn a_bus_change_redraws_the_state_at_the_next_relabel_once() {
    let b = Bench::quiet(None);
    let relabel = || b.rig.act(FaceAction::Relabel).unwrap();
    assert_eq!(relabel(), FaceReply::default());
    b.mouse(true, false, GrabState::Off);
    assert_eq!(relabel(), STATE, "the keys wake up");
    assert_eq!(relabel(), STATE, "until the UI reads the state");
    b.rig.state();
    assert_eq!(relabel(), FaceReply::default());
    b.mouse(true, false, GrabState::Off);
    assert_eq!(relabel(), FaceReply::default(), "the same state again");
}

#[test]
fn a_mouse_module_that_answers_at_once_lights_the_key_with_no_deadlock() {
    let b = Bench::quiet(None);
    b.mouse(true, false, GrabState::Off);
    let bus = b.bus.clone();
    let answer = move |e: &AssistToggle| {
        let rclick = e.which == Toggle::RightClick;
        let grab = GrabState::Off;
        bus.publish(&AssistChanged {
            live: true,
            rclick,
            grab,
        })
        .unwrap();
    };
    let _mouse = b.bus.subscribe(answer).unwrap();
    b.rig.tap("rclick").unwrap();
    assert_eq!(b.key("rclick")["on"], true);
}

#[test]
fn a_tap_on_right_click_sends_nothing_to_the_app_and_keeps_the_latches() {
    let b = Bench::quiet(None);
    b.mouse(true, false, GrabState::Off);
    b.rig.tap("shift").unwrap();
    b.said();
    b.rig.tap("rclick").unwrap();
    assert_eq!(b.said(), [Said::Toggle(Toggle::RightClick)], "no latches");
    assert!(b.rig.fakes.injector.strokes().is_empty());
    assert_eq!(b.key("shift")["on"], true);
}

#[test]
fn a_settings_change_keeps_the_mouse_state() {
    let mut b = Bench::quiet(None);
    b.mouse(true, true, GrabState::Off);
    b.rig.restart(Some(("lang_key", "cycle".into())));
    assert_eq!(b.key("rclick")["on"], true);
}
