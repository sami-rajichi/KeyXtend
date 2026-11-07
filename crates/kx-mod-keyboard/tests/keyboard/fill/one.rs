//! One panel at a time (spec §3.4): whichever opens, from its key, the history chord or the word lane's chip, the
//! keyboard closes the other with a close that never opens it, so a late or repeated word cannot leave both open.

use super::{Heard, Quick};
use crate::assist::scans;
use crate::rig::STATE;
use kx_module_api::ports::{
    ClipAct, ClipOpen, ClipPanel, FillAsk, FillClose, FillLive, FillOpen, FillPanel, HistoryAsk,
    HistoryWanted, Scan,
};
use kx_module_api::{Bus, Event, Subscription};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::SeqCst;

/// A panel's module as the tests run it: its open toggles, its close closes an open panel, its ask tells, and it
/// says each change on the bus.
#[derive(Clone)]
pub struct Owner {
    open: Arc<AtomicBool>,
    bus: Bus,
}

impl Owner {
    fn new(bus: &Bus) -> Self {
        let (open, bus) = (Arc::default(), bus.clone());
        Self { open, bus }
    }

    /// True while its panel is open.
    pub fn is_open(&self) -> bool {
        self.open.load(SeqCst)
    }

    /// Opens or closes its panel and says so, as its own close button does.
    pub fn set<E: Event>(&self, open: bool, said: fn(bool) -> E) {
        self.open.store(open, SeqCst);
        self.bus.publish(&said(open)).unwrap();
    }

    /// Calls `then` with this owner on each `E`.
    fn on<E: Event>(&self, then: impl Fn(&Self, &E) + Send + Sync + 'static) -> Subscription {
        let me = self.clone();
        self.bus.subscribe(move |e: &E| then(&me, e)).unwrap()
    }
}

/// The clipboard panel's word.
pub fn clip_said(open: bool) -> ClipPanel {
    ClipPanel { open }
}

/// Quick-fill's word.
pub fn fill_said(open: bool) -> FillPanel {
    FillPanel { open }
}

/// The clipboard module's panel handlers; it wants the history chord.
fn clipboard(c: &Owner) -> Vec<Subscription> {
    vec![
        c.on(|c, _: &ClipOpen| c.set(!c.is_open(), clip_said)),
        c.on(|c, e: &ClipAct| {
            if *e == ClipAct::Close && c.is_open() {
                c.set(false, clip_said);
            }
        }),
        c.on(|c, _: &HistoryAsk| {
            c.bus.publish(&HistoryWanted { on: true }).unwrap();
            c.set(c.is_open(), clip_said);
        }),
    ]
}

/// The vault module's panel handlers; its ask also says it runs.
fn vault(v: &Owner) -> Vec<Subscription> {
    vec![
        v.on(|v, _: &FillOpen| v.set(!v.is_open(), fill_said)),
        v.on(|v, _: &FillClose| {
            if v.is_open() {
                v.set(false, fill_said);
            }
        }),
        v.on(|v, _: &FillAsk| {
            v.bus.publish(&FillLive { live: true }).unwrap();
            v.set(v.is_open(), fill_said);
        }),
    ]
}

/// The keyboard between a clipboard module and a vault module that run.
pub struct Two {
    pub q: Quick,
    pub clip: Owner,
    pub fill: Owner,
}

impl Two {
    pub fn new() -> Self {
        let owners = Arc::new(std::sync::Mutex::new(None));
        let o = Arc::clone(&owners);
        let q = Quick::after(None, move |bus| {
            let (clip, fill) = (Owner::new(bus), Owner::new(bus));
            let mut ears = clipboard(&clip);
            ears.extend(vault(&fill));
            *o.lock().unwrap() = Some((clip, fill));
            ears
        });
        let (clip, fill) = owners.lock().unwrap().take().unwrap();
        q.heard();
        Self { q, clip, fill }
    }

    /// Which panels are open: the clipboard's, then Quick-fill's, checking never both are.
    pub fn one_at_most(&self) -> (bool, bool) {
        let open = (self.clip.is_open(), self.fill.is_open());
        assert!(!(open.0 && open.1), "never both open");
        open
    }

    /// The same, checking the keys light as the modules say.
    pub fn shown(&self) -> (bool, bool) {
        let open = self.one_at_most();
        assert_eq!(
            (self.q.on("clip"), self.q.on("fill")),
            open,
            "the keys follow"
        );
        open
    }
}

#[test]
fn the_fill_key_closes_the_clipboard_panel_and_the_clip_key_closes_quick_fill() {
    let t = Two::new();
    t.q.b.rig.tap("clip").unwrap();
    assert_eq!(t.shown(), (true, false));
    assert_eq!(t.q.b.rig.tap("fill"), Ok(STATE));
    assert_eq!(t.shown(), (false, true));
    t.q.b.rig.tap("clip").unwrap();
    assert_eq!(t.shown(), (true, false));
    t.q.b.rig.tap("clip").unwrap();
    assert_eq!(t.shown(), (false, false), "a second click closes it");
}

#[test]
fn the_lane_chip_and_the_history_chord_close_the_other_too() {
    let t = Two::new();
    t.q.b.rig.tap("clip").unwrap();
    t.q.b.send(&FillOpen);
    assert_eq!(t.shown(), (false, true), "the ui's chip");
    let chord = (scans(&[0xE05B]), Scan::new(0x2F));
    t.q.b.rig.fakes.shortcuts.set_history(Some(chord));
    t.q.b.rig.tap("win").unwrap();
    t.q.b.rig.press("c-0x002F").unwrap();
    assert_eq!(t.shown(), (true, false), "the history chord");
    assert!(t.q.strokes().is_empty());
}

#[test]
fn a_late_or_repeated_open_closes_the_other_and_never_reopens_anything() {
    let t = Two::new();
    let both = || (t.clip.is_open(), t.fill.is_open());
    t.q.b.send(&clip_said(true));
    assert_eq!(t.q.heard(), [Heard::Close], "a close, never a toggle");
    assert_eq!(both(), (false, false), "the late word opened nothing");
    t.q.b.send(&fill_said(true));
    assert_eq!(t.q.heard(), [Heard::ClipClose]);
    assert_eq!(both(), (false, false));
    t.q.b.send(&fill_said(false));
    t.q.b.rig.tap("fill").unwrap();
    t.q.heard();
    for _ in 0..2 {
        t.q.b.send(&fill_said(true));
        assert_eq!(t.shown(), (false, true), "a repeat keeps it");
    }
    t.q.b.send(&clip_said(false));
    assert_eq!(
        t.shown(),
        (false, true),
        "the other's close changes nothing"
    );
    let toggles = t.q.heard().into_iter().filter(|h| *h == Heard::Open);
    assert_eq!(toggles.count(), 0, "only a click toggles");
}

#[test]
fn a_restart_hears_both_modules_again_and_keeps_one_open() {
    let mut t = Two::new();
    t.q.b.rig.tap("fill").unwrap();
    t.q.b.rig.restart(Some(("lang_key", "cycle".into())));
    assert_eq!(t.shown(), (false, true));
    t.q.b.rig.tap("c-0x0010").unwrap();
    assert!(t.q.strokes().is_empty(), "its keys still go to it");
}
