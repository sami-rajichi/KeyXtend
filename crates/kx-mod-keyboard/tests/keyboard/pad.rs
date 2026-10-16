//! The scroll pad from the keyboard's side, over the bus: live while the scroll module says so, else dimmed with a tip;
//! its clicks and hover go out, and the module's state comes back.

#![allow(
    clippy::unwrap_used,
    reason = "these fixtures run only in tests, where a failed setup should fail the test"
)]

mod peeks;

use crate::assist::{Bench, ear};
use crate::rig::STATE;
use kx_mod_keyboard::view::{DPAD_NAME, SCROLL_OFF_TIP};
use kx_module_api::ports::{
    Dir, FaceAction, FaceReply, ScrollAsk, ScrollChanged, ScrollPeek, ScrollStop, ScrollTap,
};
use kx_module_api::{Bus, Subscription};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex, PoisonError};

/// What the keyboard said to the scroll module.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Heard {
    /// It asked for the module's state.
    Ask,
    /// An arrow was clicked.
    Tap(Dir),
    /// Stop was clicked.
    Stop,
    /// The pointer went onto the pad or off it.
    Peek(bool),
}

/// The module on a bus the test listens to, with what it said to the scroll module.
struct Pad {
    b: Bench,
    heard: Arc<Mutex<Vec<Heard>>>,
}

impl Pad {
    /// The module with `change` over the defaults, heard from before it starts.
    fn new(change: Option<(&str, toml::Value)>) -> Self {
        Self::after(change, |_| Vec::new())
    }

    /// The same, with `scroll`'s handlers on the bus before the module starts, as a scroll module that already runs.
    fn after(
        change: Option<(&str, toml::Value)>,
        scroll: impl FnOnce(&Bus) -> Vec<Subscription>,
    ) -> Self {
        let heard = Arc::default();
        let h = Arc::clone(&heard);
        let b = Bench::after(change, move |bus| {
            let mut ears = vec![
                ear(bus, &h, |_: &ScrollAsk| Heard::Ask),
                ear(bus, &h, |e: &ScrollTap| Heard::Tap(e.dir)),
                ear(bus, &h, |_: &ScrollStop| Heard::Stop),
                ear(bus, &h, |e: &ScrollPeek| Heard::Peek(e.on)),
            ];
            ears.extend(scroll(bus));
            ears
        });
        Self { b, heard }
    }

    /// The module started with `change`, with what it said at start already taken.
    fn quiet(change: Option<(&str, toml::Value)>) -> Self {
        let p = Self::new(change);
        p.heard();
        p
    }

    /// What the keyboard said to the scroll module since the last look.
    fn heard(&self) -> Vec<Heard> {
        let mut heard = self.heard.lock().unwrap_or_else(PoisonError::into_inner);
        std::mem::take(&mut *heard)
    }

    /// The scroll module says it runs, or not, scrolling towards `dir` (none while still), fast or slow.
    fn scroll(&self, live: bool, dir: Option<Dir>, fast: bool) {
        self.b.send(&ScrollChanged { live, dir, fast });
    }

    /// A click on an arrow, or on Stop when `dir` is none.
    fn click(&self, dir: Option<Dir>) -> FaceReply {
        self.b.rig.act(FaceAction::Pad { dir }).unwrap()
    }

    /// The pointer goes onto the pad or off it.
    fn hover(&self, on: bool) -> FaceReply {
        self.b.rig.act(FaceAction::PadHover { on }).unwrap()
    }

    /// The pad's state.
    fn dpad(&self) -> Value {
        self.b.rig.state()["dpad"].clone()
    }

    /// The relabel timer runs.
    fn relabel(&self) -> FaceReply {
        self.b.rig.act(FaceAction::Relabel).unwrap()
    }
}

/// The pad while the scroll module is not running.
fn off() -> Value {
    json!({ "dim": true, "tip": SCROLL_OFF_TIP, "dir": null, "fast": false })
}

/// The pad of a running scroll module, scrolling towards `dir` (none while still), fast or slow.
fn live(dir: Option<&str>, fast: bool) -> Value {
    json!({ "dim": false, "tip": null, "dir": dir, "fast": fast })
}

#[test]
fn the_board_draws_the_pad_by_its_box_and_name_and_leaves_the_rest_to_the_state() {
    let p = Pad::quiet(None);
    let pad = p.b.rig.board()["dpad"].clone();
    let mut fields: Vec<&str> = pad
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    fields.sort_unstable();
    assert_eq!(fields, ["h", "name", "w", "x", "y"]);
    assert_eq!(pad["name"], DPAD_NAME);
}

#[test]
fn start_and_restart_ask_the_scroll_module_for_its_state() {
    let mut p = Pad::new(None);
    assert_eq!(p.heard(), [Heard::Ask]);
    p.b.rig.restart(Some(("lang_key", "cycle".into())));
    assert_eq!(p.heard(), [Heard::Ask]);
}

/// Plan edge case 15: the keyboard subscribes before it asks, so a scroll module that answers at once is heard.
#[test]
fn a_scroll_module_already_running_lights_the_pad_at_start() {
    let p = Pad::after(None, |bus| {
        let reply = bus.clone();
        let on = ScrollChanged {
            live: true,
            dir: Some(Dir::Up),
            fast: false,
        };
        let answer = move |_: &ScrollAsk| reply.publish(&on).unwrap();
        vec![bus.subscribe(answer).unwrap()]
    });
    assert_eq!(p.dpad(), live(Some("up"), false));
}

/// Plan edge case 14: the scroll module is off.
#[test]
fn before_the_scroll_module_speaks_the_pad_is_dim_says_so_and_does_nothing() {
    let p = Pad::quiet(None);
    assert_eq!(p.dpad(), off());
    for dir in Dir::ALL.map(Some).into_iter().chain([None]) {
        assert_eq!(p.click(dir), FaceReply::default(), "{dir:?}");
    }
    assert_eq!(p.hover(true), FaceReply::default());
    assert!(p.heard().is_empty());
    assert!(p.b.rig.fakes.injector.strokes().is_empty());
}

#[test]
fn a_live_pad_sends_each_arrow_as_a_tap_and_stop_as_a_stop() {
    let p = Pad::quiet(None);
    p.scroll(true, None, false);
    assert_eq!(p.dpad(), live(None, false));
    for dir in Dir::ALL {
        assert_eq!(p.click(Some(dir)), FaceReply::default(), "{dir:?}");
        assert_eq!(p.heard(), [Heard::Tap(dir)]);
    }
    assert_eq!(p.click(None), FaceReply::default());
    assert_eq!(p.heard(), [Heard::Stop]);
    assert!(p.b.rig.fakes.injector.strokes().is_empty(), "nothing typed");
}

#[test]
fn the_state_carries_the_way_and_the_speed_the_scroll_module_tells() {
    let p = Pad::quiet(None);
    p.scroll(true, Some(Dir::Down), false);
    assert_eq!(p.dpad(), live(Some("down"), false));
    p.scroll(true, Some(Dir::Down), true);
    assert_eq!(p.dpad(), live(Some("down"), true));
    p.scroll(true, Some(Dir::Left), false);
    assert_eq!(p.dpad(), live(Some("left"), false));
    p.scroll(true, None, false);
    assert_eq!(p.dpad(), live(None, false), "stopped");
}

#[test]
fn a_scroll_module_that_stops_dims_the_pad_again_and_its_clicks_do_nothing() {
    let p = Pad::quiet(None);
    p.scroll(true, Some(Dir::Up), true);
    p.b.send(&ScrollChanged::OFF);
    assert_eq!(p.dpad(), off());
    p.scroll(false, Some(Dir::Up), true);
    assert_eq!(p.dpad(), off(), "no way or speed while off");
    assert_eq!(p.click(Some(Dir::Up)), FaceReply::default());
    assert_eq!(p.click(None), FaceReply::default());
    assert!(p.heard().is_empty());
}

#[test]
fn a_scroll_change_redraws_at_each_relabel_until_the_ui_reads_the_state() {
    let p = Pad::quiet(None);
    assert_eq!(p.relabel(), FaceReply::default());
    p.scroll(true, Some(Dir::Right), false);
    assert_eq!(p.relabel(), STATE, "the pad lights");
    assert_eq!(p.relabel(), STATE, "until the UI reads the state");
    p.dpad();
    assert_eq!(p.relabel(), FaceReply::default());
    p.scroll(true, Some(Dir::Right), false);
    assert_eq!(p.relabel(), FaceReply::default(), "the same state again");
}

#[test]
fn a_scroll_module_that_answers_a_tap_at_once_lights_the_pad_with_no_deadlock() {
    let p = Pad::quiet(None);
    p.scroll(true, None, false);
    let bus = p.b.bus.clone();
    let answer = move |e: &ScrollTap| {
        let on = ScrollChanged {
            live: true,
            dir: Some(e.dir),
            fast: false,
        };
        bus.publish(&on).unwrap();
    };
    let _scroll = p.b.bus.subscribe(answer).unwrap();
    p.click(Some(Dir::Up));
    assert_eq!(p.dpad(), live(Some("up"), false));
}

/// Only the keyboard tells the scroll module where the pointer is, once per change.
#[test]
fn the_pointer_on_the_pad_asks_for_the_outline_once_per_change() {
    let p = Pad::quiet(None);
    p.scroll(true, None, false);
    assert_eq!(p.hover(true), FaceReply::default());
    assert_eq!(p.heard(), [Heard::Peek(true)]);
    p.hover(true);
    assert!(p.heard().is_empty(), "the same again says nothing");
    p.hover(false);
    assert_eq!(p.heard(), [Heard::Peek(false)]);
    p.hover(false);
    assert!(p.heard().is_empty());
}

/// Plan edge case 15: a scroll module that starts, or starts again, while the pointer rests on the pad outlines.
#[test]
fn a_scroll_module_that_comes_live_under_the_pointer_hears_it_is_on_the_pad() {
    let p = Pad::quiet(None);
    p.hover(true);
    assert!(p.heard().is_empty(), "nobody to tell while off");
    p.scroll(true, None, false);
    assert_eq!(p.heard(), [Heard::Peek(true)]);
    p.scroll(true, Some(Dir::Down), false);
    assert!(p.heard().is_empty(), "only when it comes live");
    p.b.send(&ScrollChanged::OFF);
    p.scroll(true, None, false);
    assert_eq!(p.heard(), [Heard::Peek(true)], "a restart");
    p.hover(false);
    p.b.send(&ScrollChanged::OFF);
    p.scroll(true, None, false);
    assert_eq!(p.heard(), [Heard::Peek(false)], "the pointer left first");
}

#[test]
fn a_settings_change_keeps_the_scroll_state() {
    let mut p = Pad::quiet(None);
    p.scroll(true, Some(Dir::Up), true);
    p.b.rig.restart(Some(("lang_key", "cycle".into())));
    assert_eq!(p.dpad(), live(Some("up"), true));
}
