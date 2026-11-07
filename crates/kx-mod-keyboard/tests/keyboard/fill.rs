//! Quick-fill from the keyboard's side, over the bus (spec §8.1, ADR-0038): the fill key asks for the panel, the panel's
//! keys reach it instead of the app, one panel shows at a time, and a fill types at the caret with nothing told.

#![allow(
    clippy::unwrap_used,
    reason = "these fixtures run only in tests, where a failed setup should fail the test"
)]

mod front;
mod gap;
mod gate;
mod keys;
mod late;
mod live;
mod logs;
mod one;
mod props;
mod typing;

use crate::assist::{Bench, ear};
use crate::rig::STATE;
use kx_mod_keyboard::KeyboardSettings;
use kx_module_api::ports::{
    ClipAct, ClipOpen, FaceAction, FillAsk, FillClose, FillKey, FillLive, FillOpen, FillPanel,
    FillPart, FillText, FillType, FillTyped, PanelKey, Piece, Scan, TextRoute, Typed,
};
use kx_module_api::{Bus, Redacted, Secret, Subscription, Zeroizing};
use kx_platform_fake::Stroke;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

/// The wait after a Tab the tests' vault module asks for, as its `tab_gap_ms` does by default.
pub const GAP: Duration = Duration::from_millis(150);
/// The board's Tab key.
pub const TAB: u16 = 0x0F;
/// The app in front while the tests fill, unless a test names another.
pub const APP: &str = "browser.exe";
/// The window in front while the tests aim, unless a test names another.
pub const WINDOW: u64 = 1;

/// How late a part may still go: the keyboard's default `fill_late_ms`.
pub fn late() -> Duration {
    Duration::from_millis(KeyboardSettings::from_defaults().unwrap().fill_late_ms)
}

/// `d` in whole ms.
pub fn ms(d: Duration) -> u64 {
    u64::try_from(d.as_millis()).unwrap()
}

/// The gap in whole ms.
pub fn gap_ms() -> u64 {
    ms(GAP)
}

/// What the keyboard said to the vault module, and the closes and toggles it sent the clipboard module.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Heard {
    /// It asked whether the panel is open.
    Ask,
    /// The fill key asked to open or close the panel.
    Open,
    /// It asked the panel to close, as the clipboard panel opened.
    Close,
    /// A key for the open panel.
    Key(PanelKey),
    /// Its answer to a fill.
    Typed(FillTyped),
    /// It asked the clipboard panel to close, as Quick-fill opened.
    ClipClose,
    /// The clip key or the history chord asked to open or close the clipboard panel.
    ClipOpen,
}

/// The keyboard on a bus the test listens to, with what it said to the vault module and told word prediction.
pub struct Quick {
    pub b: Bench,
    heard: Arc<Mutex<Vec<Heard>>>,
    pieces: Arc<Mutex<Vec<Piece>>>,
}

impl Quick {
    /// The keyboard with `change` over the defaults and `modules`' handlers on the bus before it starts.
    pub fn after(
        change: Option<(&str, toml::Value)>,
        modules: impl FnOnce(&Bus) -> Vec<Subscription>,
    ) -> Self {
        let (heard, pieces) = (Arc::default(), Arc::default());
        let (h, p) = (Arc::clone(&heard), Arc::clone(&pieces));
        let b = Bench::after(change, move |bus| {
            let mut ears = vec![
                ear(bus, &p, |e: &Typed| e.piece.clone()),
                ear(bus, &h, |_: &FillAsk| Heard::Ask),
                ear(bus, &h, |_: &FillOpen| Heard::Open),
                ear(bus, &h, |_: &FillClose| Heard::Close),
                ear(bus, &h, |e: &FillKey| Heard::Key(e.0.clone())),
                ear(bus, &h, |e: &FillTyped| Heard::Typed(*e)),
                ear(bus, &h, |_: &ClipOpen| Heard::ClipOpen),
            ];
            let closes = ear(bus, &h, |e: &ClipAct| {
                assert_eq!(
                    *e,
                    ClipAct::Close,
                    "the keyboard only closes the clipboard panel"
                );
                Heard::ClipClose
            });
            ears.push(closes);
            ears.extend(modules(bus));
            ears
        });
        b.rig.fakes.injector.set_target(Some(APP));
        b.rig.fakes.injector.set_window(Some(WINDOW));
        Self { b, heard, pieces }
    }

    /// The app in front is now `app`, or one the system cannot name.
    pub fn front(&self, app: Option<&str>) {
        self.b.rig.fakes.injector.set_target(app);
    }

    /// The window in front is now `window`, or one the system cannot name.
    pub fn window(&self, window: Option<u64>) {
        self.b.rig.fakes.injector.set_window(window);
    }

    /// The keyboard started with `change` and the vault module running, with what it said at start already taken.
    pub fn quiet(change: Option<(&str, toml::Value)>) -> Self {
        let q = Self::after(change, |_| Vec::new());
        q.live(true);
        q.heard();
        q.b.said();
        q
    }

    /// The pieces told to word prediction since the last look.
    pub fn pieces(&self) -> Vec<Piece> {
        let mut pieces = self.pieces.lock().unwrap_or_else(PoisonError::into_inner);
        std::mem::take(&mut *pieces)
    }

    /// The same with Quick-fill open, as the vault module said, and what that said already taken.
    pub fn opened(change: Option<(&str, toml::Value)>) -> Self {
        let q = Self::quiet(change);
        q.open(true);
        assert_eq!(
            q.heard(),
            [Heard::ClipClose],
            "an open Quick-fill closes the clipboard panel"
        );
        q
    }

    /// What the keyboard said since the last look.
    pub fn heard(&self) -> Vec<Heard> {
        let mut heard = self.heard.lock().unwrap_or_else(PoisonError::into_inner);
        std::mem::take(&mut *heard)
    }

    /// The vault module says it runs, or not.
    pub fn live(&self, live: bool) {
        self.b.send(&FillLive { live });
    }

    /// The vault module says its panel opened or closed.
    pub fn open(&self, open: bool) {
        self.b.send(&FillPanel { open });
    }

    /// The vault module asks for fill `round` of `parts`, with the tests' gap after each Tab.
    pub fn fill(&self, round: u32, parts: Vec<FillPart>) {
        self.b.send(&FillType {
            round,
            parts,
            tab_gap: GAP,
        });
    }

    /// What reached the app since the last look.
    pub fn strokes(&self) -> Vec<Stroke> {
        let injector = &self.b.rig.fakes.injector;
        let strokes = injector.strokes();
        injector.clear();
        strokes
    }

    /// The relabel timer runs `ms` later.
    pub fn relabel_in(&self, ms: u64) {
        self.b.rig.fakes.clock.advance(Duration::from_millis(ms));
        self.b.rig.act(FaceAction::Relabel).unwrap();
    }

    /// Whether key `id` is lit.
    pub fn on(&self, id: &str) -> bool {
        self.b.key(id)["on"] == true
    }
}

/// Fill text `t`.
pub fn line(t: &str) -> FillPart {
    let secret = Secret::new(Zeroizing::new(t.to_owned()));
    FillPart::Text(FillText::new(secret).unwrap())
}

/// A fill of a user, a Tab and a password, which waits after its Tab.
pub fn login() -> Vec<FillPart> {
    vec![line("alex"), FillPart::Tab, line("pw")]
}

/// The answer to fill `round`.
pub fn typed(round: u32, ok: bool) -> Heard {
    Heard::Typed(FillTyped { round, ok })
}

/// Text for the panel's line or search.
pub fn text(t: &str) -> Heard {
    Heard::Key(PanelKey::Text(Redacted::new(t.into())))
}

/// Text typed as Unicode.
pub fn unicode(t: &str) -> Stroke {
    Stroke::Text(Redacted::new(t.into()), TextRoute::Unicode)
}

/// The board's Tab key, as it reaches the app.
pub fn tab() -> Stroke {
    Stroke::Tap(Scan::new(TAB))
}

#[test]
fn start_and_restart_ask_the_vault_module_whether_its_panel_is_open() {
    let mut q = Quick::after(None, |_| Vec::new());
    assert_eq!(q.heard(), [Heard::Ask]);
    q.b.rig.restart(Some(("lang_key", "cycle".into())));
    assert_eq!(q.heard(), [Heard::Ask]);
}

#[test]
fn the_fill_key_asks_for_the_panel_types_nothing_and_keeps_the_latches() {
    let q = Quick::quiet(None);
    q.b.rig.tap("shift").unwrap();
    assert_eq!(q.b.rig.tap("fill"), Ok(STATE));
    assert_eq!(q.heard(), [Heard::Open]);
    assert!(q.strokes().is_empty());
    assert!(q.on("shift"), "the latches stay");
}

#[test]
fn the_fill_key_lights_while_the_module_last_said_its_panel_is_open() {
    let q = Quick::quiet(None);
    assert!(!q.on("fill"));
    q.open(true);
    let relabel = q.b.rig.act(FaceAction::Relabel);
    assert_eq!(relabel, Ok(STATE), "the next relabel redraws");
    assert!(q.on("fill") && !q.on("clip"));
    q.open(false);
    assert!(!q.on("fill"));
}

#[test]
fn opening_quick_fill_closes_the_language_list() {
    let q = Quick::quiet(Some(("lang_key", "list".into())));
    q.b.rig.tap("lang").unwrap();
    assert!(!q.b.rig.state()["list"].is_null());
    q.open(true);
    assert!(q.b.rig.state()["list"].is_null(), "one panel at a time");
}

#[test]
fn a_restart_forgets_the_open_panel_until_told_again() {
    let mut q = Quick::opened(None);
    q.b.rig.restart(Some(("lang_key", "cycle".into())));
    assert_eq!(q.heard(), [Heard::Ask]);
    assert!(!q.on("fill"));
    q.b.rig.tap("space").unwrap();
    assert_eq!(q.strokes(), [Stroke::Tap(Scan::new(0x39))]);
}

/// The keyboard subscribes before it asks, so a vault module that answers at once is heard.
#[test]
fn a_vault_module_already_running_hands_over_its_open_panel_at_start() {
    let q = Quick::after(None, |bus| {
        let reply = bus.clone();
        let answer = move |_: &FillAsk| {
            reply.publish(&FillLive { live: true }).unwrap();
            reply.publish(&FillPanel { open: true }).unwrap();
        };
        vec![bus.subscribe(answer).unwrap()]
    });
    assert_eq!(q.heard(), [Heard::Ask, Heard::ClipClose]);
    assert!(q.on("fill"));
    q.b.rig.tap("c-0x0010").unwrap();
    assert_eq!(q.heard(), [text("q")]);
    assert!(q.strokes().is_empty());
}
