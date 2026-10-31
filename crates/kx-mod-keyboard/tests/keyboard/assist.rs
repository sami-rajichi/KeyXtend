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
pub fn ear<E: Event, T: Send + 'static>(
    bus: &Bus,
    said: &Arc<Mutex<Vec<T>>>,
    as_said: impl Fn(&E) -> T + Send + Sync + 'static,
) -> Subscription {
    let said = Arc::clone(said);
    let add = move |e: &E| said.lock().unwrap().push(as_said(e));
    bus.subscribe(add).unwrap()
}

impl Bench {
    /// The module on the golden layouts with `change` over the defaults, heard from before it starts.
    pub fn new(change: Option<(&str, toml::Value)>) -> Self {
        Self::after(change, |_| Vec::new())
    }

    /// The same, with `mouse`'s handlers on the bus before the module starts, as a mouse module that already runs.
    pub fn after(
        change: Option<(&str, toml::Value)>,
        mouse: impl FnOnce(&Bus) -> Vec<Subscription>,
    ) -> Self {
        let core: Arc<dyn BusCore> = Arc::new(FakeCore::default());
        let bus = Bus::new(Arc::clone(&core));
        let said = Arc::default();
        let mut ears = vec![
            ear(&bus, &said, |_: &AssistAsk| Said::Ask),
            ear(&bus, &said, |e: &AssistToggle| Said::Toggle(e.which)),
            ear(&bus, &said, |e: &LatchesChanged| {
                Said::Latches(e.held.clone())
            }),
        ];
        ears.extend(mouse(&bus));
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

/// True when key state `k` does nothing and shows the dim paint with the mouse-off tip.
fn off(k: &Value) -> bool {
    k["dim"] == true && k["paint"]["fill"] == "dim" && k["tip"] == MOUSE_OFF_TIP && k["on"] == false
}

#[test]
fn the_board_makes_both_keys_checkable_and_never_dims_them_and_the_later_side_keys_come_later() {
    let b = Bench::quiet(None);
    let board = b.rig.board();
    let keys = board["keys"].as_array().unwrap();
    for k in keys.iter().filter(|k| k["side"] == true) {
        let assist = k["id"] == "rclick" || k["id"] == "grab";
        let live = assist
            || ["shortcuts", "clip", "power"]
                .iter()
                .any(|id| k["id"] == *id);
        assert_eq!(k["checkable"], live && k["id"] != "power", "{}", k["id"]);
        assert_eq!(k["dim"], !live, "{}", k["id"]);
        assert_eq!(k["repeat"], false, "{}", k["id"]);
        let tip = if live { Value::Null } else { LATER_TIP.into() };
        assert_eq!(k["tip"], tip, "{}", k["id"]);
    }
}

#[test]
fn start_and_restart_tell_the_latches_and_ask_the_mouse_state() {
    let none = Said::Latches(Vec::new());
    let mut b = Bench::new(None);
    assert_eq!(b.said(), [none.clone(), Said::Ask]);
    b.rig.restart(Some(("lang_key", "cycle".into())));
    assert_eq!(b.said(), [none, Said::Ask]);
}

/// Plan edge case 18: the keyboard subscribes before it asks, so a mouse module that answers at once is heard.
#[test]
fn a_mouse_module_already_running_lights_the_keys_at_start() {
    let b = Bench::after(None, |bus| {
        let reply = bus.clone();
        let live = AssistChanged {
            live: true,
            rclick: true,
            grab: GrabState::Off,
        };
        let answer = move |_: &AssistAsk| reply.publish(&live).unwrap();
        vec![bus.subscribe(answer).unwrap()]
    });
    for id in ["rclick", "grab"] {
        let k = b.key(id);
        assert!(k["dim"] == false && k["tip"] != MOUSE_OFF_TIP, "{id}: {k}");
    }
    assert_eq!(b.key("rclick")["on"], true);
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
fn a_bus_change_redraws_at_each_relabel_until_the_ui_reads_the_state() {
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

/// The mouse module may have stopped while the keyboard was down, so its keys dim until its answer lights them again.
#[test]
fn a_keyboard_restart_dims_the_mouse_keys_until_the_mouse_module_answers() {
    let mut b = Bench::quiet(None);
    b.mouse(true, true, GrabState::Off);
    b.rig.restart(Some(("lang_key", "cycle".into())));
    assert!(
        off(&b.key("rclick")) && off(&b.key("grab")),
        "no answer yet"
    );
    b.mouse(true, true, GrabState::Off);
    assert_eq!(b.key("rclick")["on"], true);
}
