//! The scroll pad through the hub: the scroll state and the box to outline as one JSON with one wake per read, the
//! hub's ask at start, the pad's actions, the guard's scroll calls, and no box or point in any log.

mod badge;
mod pages;
mod props;

use crate::Rig;
use crate::app::keeper;
use crate::assist::{heard, woke};
use crate::logs::{lines_of, warnings_of};
use kx_module_api::ports::{
    AssistChanged, Dir, FaceAction, PointerMoved, Rect, ScrollAimed, ScrollAsk, ScrollChanged,
    ScrollPeek, ScrollStop, ScrollTap,
};
use kx_module_api::{Bus, Event, Subscription};
use kx_platform::ui::UiError;
use kx_platform_fake::GuardCall;
use kx_test_support::host::FakeCore;
use kx_ui::{Hub, Wake};
use serde_json::{Value, json};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

/// The box to outline; odd numbers, so a log line that held one would show it.
pub const BOX: Rect = Rect {
    x: 1357,
    y: 2469,
    w: 975,
    h: 531,
};
/// The scroll module running and still.
pub const STILL: ScrollChanged = ScrollChanged {
    live: true,
    dir: None,
    fast: false,
};
/// Scrolling down, slow.
pub const DOWN: ScrollChanged = ScrollChanged {
    dir: Some(Dir::Down),
    ..STILL
};

/// The scroll state as QML reads it, which lets the next change wake again.
pub fn scroll(rig: &Rig) -> Value {
    serde_json::from_str(&rig.hub.scroll_json()).unwrap()
}

/// The JSON for the scroll module off, with no outline.
fn off() -> Value {
    json!({ "live": false, "dir": null, "fast": false, "area": null })
}

/// Counts every `E` on `bus` until the subscription drops.
fn count<E: Event>(bus: &Bus) -> (Arc<AtomicUsize>, Subscription) {
    let n = Arc::new(AtomicUsize::new(0));
    let seen = Arc::clone(&n);
    let sub = bus.subscribe(move |_: &E| {
        seen.fetch_add(1, Ordering::SeqCst);
    });
    (n, sub.unwrap())
}

#[test]
fn the_scroll_state_starts_off_with_no_outline() {
    assert_eq!(scroll(&Rig::new()), off());
}

/// The scroll module may start before the UI listens, or after, so the hub asks once.
#[test]
fn listening_asks_the_scroll_module_for_its_state_once() {
    let rig = Rig::new();
    let bus = Bus::new(Arc::new(FakeCore::default()));
    let (asks, _ear) = count::<ScrollAsk>(&bus);
    let _subs = rig.hub.listen(&bus).unwrap();
    assert_eq!(asks.load(Ordering::SeqCst), 1);
}

#[test]
fn a_scroll_change_and_its_box_reach_qml_with_one_wake() {
    let (rig, bus, _subs) = heard();
    bus.publish(&DOWN).unwrap();
    bus.publish(&ScrollAimed { area: Some(BOX) }).unwrap();
    assert_eq!(woke(&rig, Wake::Scroll), 1, "one wake until QML reads");
    let want = json!({ "live": true, "dir": "down", "fast": false, "area": BOX });
    assert_eq!(scroll(&rig), want);
    let fast = ScrollChanged { fast: true, ..DOWN };
    bus.publish(&fast).unwrap();
    assert_eq!(woke(&rig, Wake::Scroll), 1, "the read let it wake again");
    assert_eq!(scroll(&rig)["fast"], true);
    bus.publish(&fast).unwrap();
    bus.publish(&ScrollAimed { area: Some(BOX) }).unwrap();
    assert_eq!(woke(&rig, Wake::Scroll), 0, "no change, no wake");
}

/// A peek outlines while still, and a quit, restart or stop leaves no outline behind.
#[test]
fn the_outline_shows_only_while_the_module_runs_and_goes_when_it_stops() {
    let (rig, bus, _subs) = heard();
    bus.publish(&ScrollAimed { area: Some(BOX) }).unwrap();
    assert_eq!(woke(&rig, Wake::Scroll), 0, "not live yet");
    assert_eq!(scroll(&rig), off());
    bus.publish(&STILL).unwrap();
    bus.publish(&ScrollAimed { area: Some(BOX) }).unwrap();
    assert_eq!(scroll(&rig)["area"], json!(BOX));
    bus.publish(&ScrollAimed { area: None }).unwrap();
    assert_eq!(scroll(&rig)["area"], Value::Null);
    bus.publish(&ScrollAimed { area: Some(BOX) }).unwrap();
    bus.publish(&ScrollChanged::OFF).unwrap();
    assert_eq!(scroll(&rig), off(), "a stopped module leaves no outline");
}

#[test]
fn a_scroll_wake_qml_never_read_goes_to_the_next_waker() {
    let rig = Rig::new();
    let (hub, _requests) = Hub::new(rig.ports()).unwrap();
    let bus = Bus::new(Arc::new(FakeCore::default()));
    let _subs = hub.listen(&bus).unwrap();
    bus.publish(&STILL).unwrap();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let taken = || std::mem::take(&mut *seen.lock().unwrap());
    hub.set_waker(keeper(&seen));
    assert_eq!(taken(), [Wake::Scroll], "it came before the waker");
    hub.drop_waker();
    hub.set_waker(keeper(&seen));
    assert_eq!(taken(), [Wake::Scroll], "the windows were made again first");
    let _ = hub.scroll_json();
    hub.drop_waker();
    hub.set_waker(keeper(&seen));
    assert!(taken().is_empty(), "read already, so no wake again");
}

/// The keyboard publishes the pad's events itself, so the hub only hands the face each action once.
#[test]
fn pad_taps_and_hover_reach_the_keyboard_as_face_actions_and_the_hub_publishes_none() {
    let (rig, face, _host) = Rig::with_face();
    let (bus, _subs) = listened(&rig);
    let ears = [count::<ScrollTap>(&bus), count::<ScrollStop>(&bus)];
    let (peeks, _peek) = count::<ScrollPeek>(&bus);
    let acts = [
        FaceAction::Pad { dir: Some(Dir::Up) },
        FaceAction::Pad { dir: None },
        FaceAction::PadHover { on: true },
        FaceAction::PadHover { on: false },
    ];
    for act in acts.clone() {
        rig.hub.act(act);
    }
    assert_eq!(face.acts(), acts);
    let heard = ears
        .iter()
        .map(|(n, _)| n.load(Ordering::SeqCst))
        .sum::<usize>();
    assert_eq!((heard, peeks.load(Ordering::SeqCst)), (0, 0));
}

#[test]
fn the_keyboard_watches_entries_until_unwatched_and_each_outline_edge_lies_under_it() {
    let rig = Rig::new();
    assert!(rig.hub.watch_entries("kb"));
    assert!(rig.hub.lay_under("edge", "kb"));
    rig.hub.unwatch_entries("kb");
    let calls = rig.guard.calls();
    let want = [
        GuardCall::WatchEntries("kb".into()),
        GuardCall::LayUnder("edge".into(), "kb".into()),
        GuardCall::UnwatchEntries("kb".into()),
    ];
    assert!(calls.ends_with(&want), "{calls:?}");
    rig.guard.set_error(Some(UiError::NoWindow));
    let quiet = warnings_of(|| {
        assert!(!rig.hub.watch_entries("kb"), "not shown yet");
        assert!(!rig.hub.lay_under("edge", "kb"));
        rig.hub.unwatch_entries("kb");
    });
    assert!(quiet.is_empty(), "{quiet:?}");
    rig.guard.set_error(Some(UiError::Refused(2)));
    let warned = warnings_of(|| {
        assert!(!rig.hub.watch_entries("kb"));
        assert!(!rig.hub.lay_under("edge", "kb"));
    });
    assert_eq!(warned.len(), 2, "{warned:?}");
}

#[test]
fn no_log_line_holds_a_box_or_a_point() {
    let at = PointerMoved {
        at: kx_module_api::ports::Point { x: 4321, y: 8765 },
    };
    let lines = lines_of(|| {
        let (rig, bus, _subs) = heard();
        bus.publish(&AssistChanged {
            live: true,
            ..AssistChanged::OFF
        })
        .unwrap();
        bus.publish(&DOWN).unwrap();
        bus.publish(&ScrollAimed { area: Some(BOX) }).unwrap();
        bus.publish(&at).unwrap();
        let _ = (scroll(&rig), rig.hub.assist_json());
        bus.publish(&ScrollChanged::OFF).unwrap();
    });
    let coords = [BOX.x, BOX.y, BOX.w, BOX.h, at.at.x, at.at.y].map(|c| c.to_string());
    let leaked: Vec<&String> = lines
        .iter()
        .filter(|l| coords.iter().any(|c| l.contains(c)))
        .collect();
    assert!(leaked.is_empty(), "{leaked:?}");
}

/// A fresh bus `rig`'s hub hears, and the subscriptions that keep it hearing.
pub fn listened(rig: &Rig) -> (Bus, Vec<Subscription>) {
    let bus = Bus::new(Arc::new(FakeCore::default()));
    let subs = rig.hub.listen(&bus).unwrap();
    rig.wakes();
    (bus, subs)
}
