//! The chip lane through the hub: each `Suggested` wakes the top bar and shows in `words_json`, the hub asks for the
//! lane as it starts listening, and no log line holds a chip's text.

mod asks;
mod view;

use crate::logs::lines_of;
use crate::{FakeFace, Rig};
use kx_module_api::ports::{ChipKind, ChipPicked, Lane, ListPage, Suggested, WordsAsk};
use kx_module_api::{Bus, Capability, Event, EventError, Redacted, Subscription};
use kx_test_support::host::{FakeCore, FakeHost};
use kx_ui::{MANIFEST, UiSettings, Wake};
use serde_json::Value;
use std::sync::{Arc, Mutex};

/// What the hub sent on the bus, oldest first.
#[derive(Default)]
pub struct Sent {
    picks: Mutex<Vec<ChipPicked>>,
    asks: Mutex<Vec<WordsAsk>>,
}

impl Sent {
    /// The chip taps sent since the last call.
    pub fn picks(&self) -> Vec<ChipPicked> {
        std::mem::take(&mut *self.picks.lock().unwrap())
    }

    /// The asks sent since the last call.
    pub fn asks(&self) -> Vec<WordsAsk> {
        std::mem::take(&mut *self.asks.lock().unwrap())
    }
}

/// A bus `rig`'s hub hears, what the hub sends there, and the subscriptions that keep both; the wakes are cleared.
pub fn heard(rig: &Rig) -> (Bus, Arc<Sent>, Vec<Subscription>) {
    let bus = Bus::new(Arc::new(FakeCore::default()));
    let sent = Arc::new(Sent::default());
    let (picks, asks) = (Arc::clone(&sent), Arc::clone(&sent));
    let mut subs = vec![
        bus.subscribe(move |e: &ChipPicked| picks.picks.lock().unwrap().push(*e))
            .unwrap(),
        bus.subscribe(move |e: &WordsAsk| asks.asks.lock().unwrap().push(*e))
            .unwrap(),
    ];
    subs.extend(rig.hub.listen(&bus).unwrap());
    rig.wakes();
    (bus, sent, subs)
}

pub use crate::suggest::{chip, lane};

/// The lane as the top bar reads it.
pub fn words(rig: &Rig) -> Value {
    serde_json::from_str(&rig.hub.words_json()).unwrap()
}

/// A rig whose keyboard shows a layout in language `iso` with `[ui] page_rows` at `rows`, and its host.
pub fn speaking(iso: &str, rows: usize) -> (Rig, FakeHost) {
    let rig = Rig::new();
    let face = Arc::new(FakeFace::default());
    face.set_lang(iso);
    let mut host = FakeHost::new(&MANIFEST);
    let mut settings = UiSettings::from_defaults().unwrap();
    settings.page_rows = rows;
    rig.hub.attach(settings, face, host.handle());
    rig.wakes();
    (rig, host)
}

#[test]
fn a_lane_wakes_the_top_bar_once_and_two_wake_twice_with_the_newest_shown() {
    let rig = Rig::new();
    let (bus, _sent, _subs) = heard(&rig);
    let word = |w: &str| Lane::Chips(vec![chip(w, ChipKind::Word)]);
    bus.publish(&lane(3, word("hello"))).unwrap();
    assert_eq!(rig.wakes(), [Wake::Words]);
    assert_eq!(words(&rig)["round"], 3);
    bus.publish(&lane(4, word("help"))).unwrap();
    bus.publish(&lane(5, word("helmet"))).unwrap();
    assert_eq!(rig.wakes(), [Wake::Words, Wake::Words]);
    let shown = words(&rig);
    assert_eq!(shown["round"], 5);
    assert_eq!(shown["chips"][0]["text"], "helmet");
}

#[test]
fn listening_asks_for_the_lane_again_so_any_start_order_works() {
    let rig = Rig::new();
    let (_bus, sent, _subs) = heard(&rig);
    assert_eq!(sent.asks(), [WordsAsk::Again]);
    assert!(rig.wakes().is_empty(), "an ask wakes nothing");
}

#[test]
fn before_any_lane_prediction_counts_as_stopped_with_no_menu_to_offer() {
    let rig = Rig::new();
    let shown = words(&rig);
    assert_eq!(shown["kind"], "stopped");
    assert_eq!(shown["chips"], Value::Array(Vec::new()));
    assert_eq!(
        (&shown["on"], &shown["learn"]),
        (&true.into(), &true.into())
    );
    assert_eq!(
        (&shown["forgetting"], &shown["list"]),
        (&false.into(), &Value::Null)
    );
}

#[test]
fn a_pick_sends_the_round_qml_drew_so_an_old_lane_reaches_the_module_as_old() {
    let rig = Rig::new();
    let (bus, sent, _subs) = heard(&rig);
    rig.hub.pick_chip(7, 2);
    assert_eq!(sent.picks(), [ChipPicked { round: 7, index: 2 }]);
    bus.publish(&lane(9, Lane::Empty)).unwrap();
    rig.hub.pick_chip(7, 2);
    assert_eq!(
        sent.picks(),
        [ChipPicked { round: 7, index: 2 }],
        "untouched"
    );
}

#[test]
fn a_pick_outside_a_round_or_a_lane_place_sends_nothing() {
    let rig = Rig::new();
    let (_bus, sent, _subs) = heard(&rig);
    let past = i64::from(u32::MAX) + 1;
    for (round, index) in [(-1, 0), (past, 0), (7, -1), (7, 256)] {
        rig.hub.pick_chip(round, index);
    }
    assert!(sent.picks().is_empty());
    rig.hub.pick_chip(i64::from(u32::MAX), 255);
    let last = ChipPicked {
        round: u32::MAX,
        index: u8::MAX,
    };
    assert_eq!(sent.picks(), [last], "the ends still go");
}

#[test]
fn nothing_is_sent_before_the_hub_listens() {
    let rig = Rig::new();
    let bus = Bus::new(Arc::new(FakeCore::default()));
    let sent = Arc::new(Sent::default());
    let keep = Arc::clone(&sent);
    let _sub = bus.subscribe(move |e: &ChipPicked| keep.picks.lock().unwrap().push(*e));
    rig.hub.pick_chip(1, 0);
    assert!(sent.picks().is_empty());
}

#[test]
fn without_read_focused_text_the_hub_neither_hears_nor_sends_the_lane() {
    let gate = Capability::ReadFocusedText;
    assert!(MANIFEST.capabilities.contains(&gate));
    let narrowed: Vec<Capability> = MANIFEST
        .capabilities
        .iter()
        .copied()
        .filter(|c| *c != gate)
        .collect();
    let bus = Bus::new(Arc::new(FakeCore::default()));
    let pick = ChipPicked { round: 1, index: 0 };
    let refused = EventError::NotGranted(ChipPicked::NAME, gate);
    assert_eq!(
        bus.restricted(&narrowed).publish(&pick).err(),
        Some(refused)
    );
    assert!(bus.restricted(MANIFEST.capabilities).publish(&pick).is_ok());
    let rig = Rig::new();
    let refused = EventError::NotGranted(Suggested::NAME, gate);
    let heard = rig.hub.listen(&bus.restricted(&narrowed));
    assert_eq!(heard.err(), Some(refused));
}

#[test]
fn no_log_line_or_wake_holds_a_chip_or_an_email() {
    let secrets = ["zanzibar", "قرطبة", "kept@secret.example"];
    let lines = lines_of(|| {
        let rig = Rig::new();
        let (bus, _sent, _subs) = heard(&rig);
        let chips = vec![
            chip(secrets[0], ChipKind::Word),
            chip(secrets[1], ChipKind::Word),
            chip(secrets[2], ChipKind::Email),
        ];
        let mut shown = lane(2, Lane::Chips(chips));
        shown.list = Some(ListPage {
            items: vec![Redacted::new(secrets[2].to_owned())],
            page: 0,
            pages: 1,
        });
        bus.publish(&shown).unwrap();
        let _ = rig.hub.words_json();
        rig.hub.pick_chip(2, 1);
        rig.hub.pick_chip(-2, 1);
        rig.hub.words_ask("mystery", 0);
        let woke = format!("{:?}", rig.wakes());
        assert!(!secrets.iter().any(|s| woke.contains(s)), "{woke}");
    });
    assert_eq!(lines.len(), 2, "both refused taps warn: {lines:?}");
    let leaked: Vec<&String> = lines
        .iter()
        .filter(|l| secrets.iter().any(|s| l.contains(s)))
        .collect();
    assert!(leaked.is_empty(), "{leaked:?}");
}
