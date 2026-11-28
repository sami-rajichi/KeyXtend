//! Snip through the hub: the shared rig, a bus the hub hears with the clicks it sent and the wakes it gave, and the
//! pages the tests build.

mod acts;
mod aside;
mod drawn;
mod editor;
mod esc;
mod flat;
mod frame;
mod gate;
mod logs;
pub(crate) mod order;
mod ours;
mod shadows;
mod shield;
mod view;

use crate::Rig;
use kx_module_api::ports::{
    BarKey, CloseAsk, Crop, Dim, Editor, FrozenScreen, Ink, Keycap, Part, Picking, PictureEdit,
    Point, Rect, Shot, SnipAct, SnipAside, SnipClick, SnipPage, SnipSpot, SnipStage, SnipView,
    StripKey, Taken, Tool,
};
use kx_module_api::{Bus, Subscription};
use kx_test_support::host::FakeCore;
use kx_ui::SnipWake;
use serde_json::Value;
use std::sync::{Arc, Mutex};

/// What the hub sent on the bus and how it woke Snip's object, oldest first.
#[derive(Default)]
pub struct Sent {
    acts: Mutex<Vec<SnipAct>>,
    woken: Mutex<Vec<SnipWake>>,
}

impl Sent {
    /// The clicks sent since the last call.
    pub fn acts(&self) -> Vec<SnipAct> {
        std::mem::take(&mut *self.acts.lock().unwrap())
    }

    /// Only what each click sent since the last call named.
    pub fn clicks(&self) -> Vec<SnipClick> {
        self.acts().into_iter().map(|a| a.click).collect()
    }

    /// The wakes since the last call.
    pub fn woken(&self) -> Vec<SnipWake> {
        std::mem::take(&mut *self.woken.lock().unwrap())
    }
}

/// A bus the hub hears, what it sends there, and the subscriptions that keep both.
pub struct Heard {
    pub bus: Bus,
    pub sent: Arc<Sent>,
    _subs: Vec<Subscription>,
}

impl Heard {
    /// The module draws `page`, which QML reads after the Qt thread's own work; what was sent so far is forgotten.
    pub fn shown(&self, rig: &Rig, page: SnipPage) {
        self.bus.publish(&SnipView::new(page)).unwrap();
        rig.hub.snip_woken();
        let _ = rig.hub.snip_json();
        self.sent.acts();
        self.sent.woken();
    }

    /// The module steps the keyboard `aside`.
    pub fn aside(&self, aside: SnipAside) {
        self.bus.publish(&aside).unwrap();
    }
}

/// A bus the hub of `rig` hears, with Snip's object's waker set; nothing sent or woken so far is kept.
pub fn heard(rig: &Rig) -> Heard {
    let bus = Bus::new(Arc::new(FakeCore::default()));
    let sent = Arc::new(Sent::default());
    let acts = Arc::clone(&sent);
    let mut subs = vec![
        bus.subscribe(move |a: &SnipAct| acts.acts.lock().unwrap().push(a.clone()))
            .unwrap(),
    ];
    subs.extend(rig.hub.listen(&bus).unwrap());
    let woken = Arc::clone(&sent);
    rig.hub
        .set_snip_waker(Box::new(move |w| woken.woken.lock().unwrap().push(w)));
    sent.acts();
    sent.woken();
    Heard {
        bus,
        sent,
        _subs: subs,
    }
}

/// The view `snip_json` gives now.
pub fn json(rig: &Rig) -> Value {
    serde_json::from_str(&rig.hub.snip_json()).unwrap()
}

/// `w` by `h` pixels of one colour: blue 1, green 2, red 3 and an unused 0, as a freeze gives them.
pub fn shot(w: u32, h: u32) -> Shot {
    let size = usize::try_from(w * h).unwrap();
    Shot::new(w, h, [1, 2, 3, 0].repeat(size)).unwrap()
}

/// The rectangle at (`x`, `y`) of `w` by `h`.
pub fn rect(x: i32, y: i32, w: i32, h: i32) -> Rect {
    Rect { x, y, w, h }
}

/// A screen frozen at `at` and `dpi`, its pixels `shot(w, h)`.
pub fn frozen(at: Rect, dpi: u32) -> FrozenScreen {
    let (w, h) = (at.w.try_into().unwrap(), at.h.try_into().unwrap());
    FrozenScreen::new(at, dpi, shot(w, h)).unwrap()
}

/// Two frozen screens: a small one at 100 %, and one at 150 % to its right.
pub fn screens() -> Vec<FrozenScreen> {
    vec![frozen(rect(0, 0, 8, 6), 96), frozen(rect(8, 0, 12, 9), 144)]
}

/// The capture bar's keys, Region picked and Video full dimmed with no encoders.
pub fn bar() -> Vec<Keycap<BarKey>> {
    BarKey::ALL
        .into_iter()
        .map(|key| Keycap {
            key,
            on: key == BarKey::Region,
            dim: (key == BarKey::VideoFull).then_some(Dim::NoVideo),
        })
        .collect()
}

/// Round `round` picking over `frozen`.
pub fn picking(round: u32, frozen: Vec<FrozenScreen>) -> SnipPage {
    let stage = SnipStage::Pick(Picking {
        screen: frozen[0].rect(),
        keys: bar(),
        corner: Some(Point { x: 2, y: 3 }),
        within: None,
        windows: vec![rect(1, 1, 4, 3)],
    });
    page(round, frozen, stage)
}

/// Round `round` with the strip beside a part of the first screen.
pub fn strip(round: u32) -> SnipPage {
    let keys = StripKey::ALL
        .into_iter()
        .map(|key| Keycap {
            key,
            on: false,
            dim: None,
        })
        .collect();
    let stage = SnipStage::Strip(Taken {
        part: Part::new(rect(1, 1, 4, 3)).unwrap(),
        screen: rect(0, 0, 8, 6),
        spot: SnipSpot::Under,
        keys,
        length_ms: None,
        speed: None,
        done: None,
        waits: None,
        frame: None,
    });
    page(round, screens(), stage)
}

/// Round `round` with a picture's editor and nothing drawn, at `place` if the module kept one.
pub fn editing(round: u32, place: Option<Rect>) -> SnipPage {
    let body = PictureEdit {
        crop: Crop::new(rect(0, 0, 4, 3), 4, 3).unwrap(),
        drawings: Vec::new(),
        tool: Tool::Pen,
        ink: Ink::Red,
        size: 0,
        pending: None,
        frame: None,
        typing: None,
        undo: false,
        redo: false,
    };
    let editor = Editor {
        picture: shot(4, 3),
        body,
        save_dim: None,
        done: None,
        place,
        full: None,
    };
    page(round, Vec::new(), SnipStage::Edit(Box::new(editor)))
}

/// Round `round` with nothing shown: the snip ended.
pub fn idle(round: u32) -> SnipPage {
    page(round, Vec::new(), SnipStage::Idle)
}

/// Round `round` on `frozen` with `stage`, and no question, note or panel.
pub fn page(round: u32, frozen: Vec<FrozenScreen>, stage: SnipStage) -> SnipPage {
    SnipPage {
        round,
        frozen,
        stage,
        ask: None::<CloseAsk>,
        note: None,
        panel: None,
        dialog: false,
    }
}
