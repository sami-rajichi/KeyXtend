//! Voice typing through the hub: the shared rig, a bus the hub hears with what it sent and woke, and the views, panels
//! and buttons the tests build and read.

mod acts;
mod bar;
mod compare;
mod device;
mod gate;
mod history;
mod models;
mod options;
mod panel;
mod place;
mod wakes;

use crate::Rig;
use kx_module_api::ports::{
    BarPlace, DeviceLabel, DeviceName, Listening, Pages, PanelButton, VoiceAct, VoiceAsk, VoiceBar,
    VoiceHome, VoiceLang, VoiceLevel, VoiceOpen, VoicePage, VoicePanel, VoiceSheet, VoiceView,
};
use kx_module_api::{Bus, Subscription};
use kx_test_support::host::FakeCore;
use kx_ui::{I18n, VoiceWake, find};
use serde_json::Value;
use std::sync::{Arc, Mutex};

/// What the hub sent on the bus and how it woke the bar's object, oldest first.
#[derive(Default)]
pub struct Sent {
    acts: Mutex<Vec<VoiceAct>>,
    opens: Mutex<usize>,
    asks: Mutex<usize>,
    woken: Mutex<Vec<VoiceWake>>,
}

impl Sent {
    /// The clicks sent since the last call.
    pub fn acts(&self) -> Vec<VoiceAct> {
        std::mem::take(&mut *self.acts.lock().unwrap())
    }

    /// How many times the panel was asked to open since the last call.
    pub fn opens(&self) -> usize {
        std::mem::take(&mut *self.opens.lock().unwrap())
    }

    /// How many times the module was asked to tell its state since the last call.
    pub fn asks(&self) -> usize {
        std::mem::take(&mut *self.asks.lock().unwrap())
    }

    /// The wakes since the last call.
    pub fn woken(&self) -> Vec<VoiceWake> {
        std::mem::take(&mut *self.woken.lock().unwrap())
    }

    /// Forgets everything so far.
    pub fn clear(&self) {
        self.acts();
        self.opens();
        self.asks();
        self.woken();
    }
}

/// A bus the hub hears, what it sends there, and the subscriptions that keep both.
pub struct Heard {
    pub bus: Bus,
    pub sent: Arc<Sent>,
    _subs: Vec<Subscription>,
}

impl Heard {
    /// The module draws `page`.
    pub fn view(&self, page: VoicePage) {
        self.bus.publish(&VoiceView::new(page)).unwrap();
    }

    /// The module tells the sound level.
    pub fn level(&self, level: u8) {
        self.bus.publish(&VoiceLevel { level }).unwrap();
    }

    /// The module draws `page`, which QML reads; what was sent and woken so far is cleared.
    pub fn shown(&self, rig: &Rig, page: VoicePage) {
        self.view(page);
        let _ = rig.hub.voice_json();
        self.sent.clear();
    }
}

/// A bus the hub of `rig` hears, with the bar's object's waker set; nothing sent or woken so far is kept.
pub fn heard(rig: &Rig) -> Heard {
    let heard = listening(rig);
    heard.sent.clear();
    heard
}

/// A bus the hub of `rig` hears, with the bar's object's waker set, keeping what the start sent.
pub fn listening(rig: &Rig) -> Heard {
    let bus = Bus::new(Arc::new(FakeCore::default()));
    let sent = Arc::new(Sent::default());
    let (acts, opens, asks) = (Arc::clone(&sent), Arc::clone(&sent), Arc::clone(&sent));
    let mut subs = vec![
        bus.subscribe(move |a: &VoiceAct| acts.acts.lock().unwrap().push(a.clone()))
            .unwrap(),
        bus.subscribe(move |_: &VoiceOpen| *opens.opens.lock().unwrap() += 1)
            .unwrap(),
        bus.subscribe(move |_: &VoiceAsk| *asks.asks.lock().unwrap() += 1)
            .unwrap(),
    ];
    subs.extend(rig.hub.listen(&bus).unwrap());
    let woken = Arc::clone(&sent);
    rig.hub
        .set_voice_waker(Box::new(move |w| woken.woken.lock().unwrap().push(w)));
    Heard {
        bus,
        sent,
        _subs: subs,
    }
}

/// The language a test dictates in.
pub fn english() -> VoiceLang {
    VoiceLang {
        name: "English".to_owned(),
        rtl: false,
        arabic: false,
    }
}

/// Arabic, as Windows names it.
pub fn arabic() -> VoiceLang {
    VoiceLang {
        name: "العربية".to_owned(),
        rtl: true,
        arabic: true,
    }
}

/// The Listening state, 1.2 s in.
pub fn listening_bar() -> VoiceBar {
    VoiceBar::Listening(Listening {
        lang: english(),
        elapsed_ms: 1200,
        nothing: false,
        wave_ms: 300,
    })
}

/// View `round` with `bar` and `panel`, above the keyboard, with sounds.
pub fn page(round: u32, bar: VoiceBar, panel: Option<VoicePanel>) -> VoicePage {
    VoicePage {
        round,
        bar,
        panel,
        place: BarPlace::Above,
        sounds: true,
    }
}

/// The panel on `sheet` for English, one page, nothing ticked, the buttons in `acts` acting.
pub fn acting(sheet: VoiceSheet, acts: &[PanelButton]) -> VoicePanel {
    VoicePanel {
        sheet,
        lang: english(),
        pages: Pages { at: 0, count: 1 },
        ticked: 0,
        acts: acts.to_vec(),
        note: None,
    }
}

/// The panel on `sheet`, one page, nothing ticked, only Close acting.
pub fn panel(sheet: VoiceSheet) -> VoicePanel {
    acting(sheet, &[PanelButton::Close])
}

/// The panel QML reads for `panel` in round 1 with no bar.
pub fn panel_json(rig: &Rig, heard: &Heard, panel: VoicePanel) -> Value {
    heard.view(page(1, VoiceBar::Hidden, Some(panel)));
    json(rig)["panel"].clone()
}

/// The `[kind, id, label, on, tone]` of each button in `buttons`, a JSON list.
pub fn buttons(buttons: &Value) -> Vec<[String; 5]> {
    let field = |b: &Value, k: &str| match &b[k] {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    };
    let all = buttons.as_array().cloned().unwrap_or_default();
    all.iter()
        .map(|b| ["kind", "id", "label", "on", "tone"].map(|k| field(b, k)))
        .collect()
}

/// A button as `buttons` lists it.
pub fn button(kind: &str, id: &str, label: &str, on: bool, tone: &str) -> [String; 5] {
    [kind, id, label, &on.to_string(), tone].map(str::to_owned)
}

/// The home list: no model ready, Automatic on the processor, never compared, 12 texts, defaults.
pub fn home() -> VoiceSheet {
    VoiceSheet::Home(VoiceHome {
        model: None,
        device: DeviceLabel::Auto(DeviceName::Cpu),
        compared: None,
        history: 12,
        changed: 0,
    })
}

/// The English text of `key`.
pub fn english_text(key: &str) -> String {
    I18n::new(find("en").unwrap()).text(key)
}

/// The bar and panel as QML reads them.
pub fn json(rig: &Rig) -> Value {
    serde_json::from_str(&rig.hub.voice_json()).unwrap()
}
