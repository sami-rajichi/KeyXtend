//! The selection strip through the hub: a view wakes QML once per read and `Hidden` drops it, a ticked view stays until
//! QML's fade ends or the next view comes, and a view with an unknown screen or absurd boxes shows nothing.

mod acts;
mod board;
mod gate;
mod place;
mod view;

use crate::Rig;
use kx_module_api::ports::{
    BASE_DPI, Board, BoardPlace, ChipAct, ChipsView, Legend, Rect, Screen, SelChip, StripSpot,
};
use kx_module_api::{Bus, Subscription};
use kx_test_support::host::FakeCore;
use kx_ui::Wake;
use kx_ui::i18n::keys::sel_chip_id;
use serde_json::Value;
use std::sync::{Arc, Mutex};

/// The work area every strip lands on.
pub const AREA: Rect = Rect {
    x: 0,
    y: 0,
    w: 1920,
    h: 1040,
};

/// What the hub sent on the bus, oldest first.
#[derive(Default)]
pub struct Sent {
    acts: Mutex<Vec<ChipAct>>,
    places: Mutex<Vec<Board>>,
}

impl Sent {
    /// The clicks sent since the last call.
    pub fn acts(&self) -> Vec<ChipAct> {
        std::mem::take(&mut *self.acts.lock().unwrap())
    }

    /// The keyboard's places told since the last call.
    pub fn places(&self) -> Vec<Board> {
        std::mem::take(&mut *self.places.lock().unwrap())
    }
}

/// A bus the hub of `rig` hears, what it sends there, and the subscriptions that keep both; wakes and sends are cleared.
pub fn heard(rig: &Rig) -> (Bus, Arc<Sent>, Vec<Subscription>) {
    let bus = Bus::new(Arc::new(FakeCore::default()));
    let sent = Arc::new(Sent::default());
    let (acts, places) = (Arc::clone(&sent), Arc::clone(&sent));
    let mut subs = vec![
        bus.subscribe(move |a: &ChipAct| acts.acts.lock().unwrap().push(*a))
            .unwrap(),
        bus.subscribe(move |p: &BoardPlace| places.places.lock().unwrap().push(p.place))
            .unwrap(),
    ];
    subs.extend(rig.hub.listen(&bus).unwrap());
    rig.wakes();
    sent.acts();
    sent.places();
    (bus, sent, subs)
}

/// A screen with work area `AREA` at `dpi`.
pub fn screen(dpi: u32) -> Screen {
    Screen {
        rect: AREA,
        dpi,
        primary: true,
    }
}

/// A line box at (`x`, `y`), 200 by 20.
pub fn line(x: i32, y: i32) -> Rect {
    Rect {
        x,
        y,
        w: 200,
        h: 20,
    }
}

/// The strip by one line at (`x`, `y`) on a screen at `dpi`.
pub fn spot(x: i32, y: i32, dpi: u32) -> StripSpot {
    StripSpot {
        first: line(x, y),
        last: line(x, y),
        screen: screen(dpi),
    }
}

/// Strip `round` of every chip by `spot`, with `acted`'s tick if any, showing `legend`.
pub fn strip(round: u32, spot: StripSpot, acted: Option<SelChip>, legend: Legend) -> ChipsView {
    ChipsView::Shown {
        round,
        chips: SelChip::ALL.to_vec(),
        spot,
        acted,
        legend,
    }
}

/// The line the usual strip sits by, at (100, 300) at 100 %.
pub fn usual() -> StripSpot {
    spot(100, 300, BASE_DPI)
}

/// Strip `round` by the usual line, showing both legends, nothing clicked.
pub fn view(round: u32) -> ChipsView {
    strip(round, usual(), None, Legend::Both)
}

/// Strip `round` by the usual line with `chip`'s tick.
pub fn ticked(round: u32, chip: SelChip) -> ChipsView {
    strip(round, usual(), Some(chip), Legend::Both)
}

/// The strip as QML reads it.
pub fn sel(rig: &Rig) -> Value {
    serde_json::from_str(&rig.hub.sel_json()).unwrap()
}

#[test]
fn one_view_wakes_once_and_a_second_waits_for_the_read() {
    let rig = Rig::new();
    let (bus, _sent, _subs) = heard(&rig);
    bus.publish(&view(1)).unwrap();
    assert_eq!(rig.wakes(), [Wake::Sel]);
    bus.publish(&view(2)).unwrap();
    assert!(rig.wakes().is_empty(), "a wake waits for the read");
    assert_eq!(sel(&rig)["round"], 2, "the newest view shows");
    bus.publish(&view(3)).unwrap();
    assert_eq!(rig.wakes(), [Wake::Sel], "read, so the next view wakes");
}

#[test]
fn hidden_drops_the_view_and_wakes_once() {
    let rig = Rig::new();
    let (bus, _sent, _subs) = heard(&rig);
    bus.publish(&ChipsView::Hidden).unwrap();
    assert!(rig.wakes().is_empty(), "nothing showed");
    bus.publish(&view(4)).unwrap();
    assert_eq!(sel(&rig)["round"], 4);
    rig.wakes();
    bus.publish(&ChipsView::Hidden).unwrap();
    assert_eq!(rig.wakes(), [Wake::Sel]);
    assert_eq!(rig.hub.sel_json(), "null", "nothing to draw");
    bus.publish(&ChipsView::Hidden).unwrap();
    assert!(rig.wakes().is_empty(), "hidden already");
}

#[test]
fn a_ticked_view_stays_until_its_fade_ends_or_the_next_view_comes() {
    let rig = Rig::new();
    let (bus, _sent, _subs) = heard(&rig);
    bus.publish(&ticked(5, SelChip::Cut)).unwrap();
    assert_eq!(sel(&rig)["acted"], sel_chip_id(SelChip::Cut));
    rig.wakes();
    for other in [4, 6, -5, i64::from(u32::MAX) + 5] {
        rig.hub.sel_faded(other);
        assert_eq!(sel(&rig)["round"], 5, "fade of {other}");
    }
    rig.hub.sel_faded(5);
    assert_eq!(rig.hub.sel_json(), "null", "the fade ended it");
    assert!(rig.wakes().is_empty(), "QML faded it itself");
    bus.publish(&ticked(6, SelChip::Copy)).unwrap();
    bus.publish(&view(7)).unwrap();
    let shown = sel(&rig);
    assert_eq!(
        (&shown["round"], &shown["acted"]),
        (&7.into(), &Value::Null)
    );
    rig.hub.sel_faded(7);
    assert_eq!(sel(&rig)["round"], 7, "only a ticked strip fades");
}

/// Spots no strip may show by: an unknown screen, a box of negative size, a last line off the work area, a flat area.
fn absurd() -> [StripSpot; 4] {
    let unknown = StripSpot {
        screen: Screen::default(),
        ..usual()
    };
    let backwards = StripSpot {
        first: Rect {
            h: -5,
            ..line(100, 300)
        },
        ..usual()
    };
    let away = StripSpot {
        last: line(AREA.w + 500, 300),
        ..usual()
    };
    let flat = StripSpot {
        screen: Screen {
            rect: Rect { h: 0, ..AREA },
            ..screen(BASE_DPI)
        },
        ..usual()
    };
    [unknown, backwards, away, flat]
}

#[test]
fn a_view_with_an_unknown_screen_or_absurd_boxes_shows_nothing() {
    let rig = Rig::new();
    let (bus, _sent, _subs) = heard(&rig);
    for bad in absurd() {
        bus.publish(&view(1)).unwrap();
        sel(&rig);
        rig.wakes();
        bus.publish(&strip(2, bad, None, Legend::Both)).unwrap();
        assert_eq!(rig.hub.sel_json(), "null", "{bad:?}");
        assert_eq!(rig.wakes(), [Wake::Sel], "the shown strip goes");
    }
    let none = ChipsView::Shown {
        round: 3,
        chips: Vec::new(),
        spot: usual(),
        acted: None,
        legend: Legend::Both,
    };
    bus.publish(&none).unwrap();
    assert_eq!(rig.hub.sel_json(), "null", "no chips");
}

#[test]
fn a_new_waker_hears_the_wake_qml_never_read() {
    let rig = Rig::new();
    let (bus, _sent, _subs) = heard(&rig);
    bus.publish(&view(1)).unwrap();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let log = Arc::clone(&seen);
    rig.hub
        .set_waker(Box::new(move |w| log.lock().unwrap().push(w)));
    assert!(seen.lock().unwrap().contains(&Wake::Sel));
}
