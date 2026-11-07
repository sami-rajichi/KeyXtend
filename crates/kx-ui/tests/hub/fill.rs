//! Quick-fill through the hub: a view wakes the panel's object once per read and an older one is ignored, a closed
//! panel drops its view, a new waker hears an unread wake, and a pad page turns the open panel.

mod acts;
mod backup;
mod gate;
mod sheets;
mod start;
mod view;

use crate::Rig;
use kx_module_api::ports::{
    EntryKind, EntryRow, FillAct, FillClick, FillFilter, FillNames, FillPage, FillPanel, FillView,
    Hint, Pages, RowButton, ScrollPage, Sheet,
};
use kx_module_api::{Bus, Redacted, Secret, Subscription, Zeroizing};
use kx_test_support::host::FakeCore;
use kx_ui::Wake;
use serde_json::Value;
use std::sync::{Arc, Mutex};

/// What the hub sent on the bus, oldest first, and how often it woke the panel's object.
#[derive(Default)]
pub struct Sent {
    acts: Mutex<Vec<FillAct>>,
    names: Mutex<Vec<FillNames>>,
    woken: Mutex<usize>,
}

impl Sent {
    /// The clicks sent since the last call.
    pub fn acts(&self) -> Vec<FillAct> {
        std::mem::take(&mut *self.acts.lock().unwrap())
    }

    /// The label names sent since the last call.
    pub fn names(&self) -> Vec<FillNames> {
        std::mem::take(&mut *self.names.lock().unwrap())
    }

    /// The wakes since the last call.
    pub fn woken(&self) -> usize {
        std::mem::take(&mut *self.woken.lock().unwrap())
    }

    /// Forgets everything so far.
    pub fn clear(&self) {
        self.acts();
        self.names();
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
    /// The vault says its panel opens or closes.
    pub fn panel(&self, open: bool) {
        self.bus.publish(&FillPanel { open }).unwrap();
    }

    /// The vault draws `page`.
    pub fn view(&self, page: FillPage) {
        self.bus.publish(&FillView::new(page)).unwrap();
    }

    /// The panel opens and shows `page`; what was sent and woken so far is cleared.
    pub fn shown(&self, rig: &Rig, page: FillPage) {
        self.panel(true);
        self.view(page);
        rig.wakes();
        self.sent.clear();
    }
}

/// A bus the hub of `rig` hears, with the panel's object's waker set; the wakes are cleared.
pub fn heard(rig: &Rig) -> Heard {
    let bus = Bus::new(Arc::new(FakeCore::default()));
    let sent = Arc::new(Sent::default());
    let (acts, names, woken) = (Arc::clone(&sent), Arc::clone(&sent), Arc::clone(&sent));
    let mut subs = vec![
        bus.subscribe(move |a: &FillAct| acts.acts.lock().unwrap().push(*a))
            .unwrap(),
        bus.subscribe(move |n: &FillNames| names.names.lock().unwrap().push(n.clone()))
            .unwrap(),
    ];
    subs.extend(rig.hub.listen(&bus).unwrap());
    rig.hub
        .set_fill_waker(Box::new(move || *woken.woken.lock().unwrap() += 1));
    rig.wakes();
    sent.clear();
    Heard {
        bus,
        sent,
        _subs: subs,
    }
}

/// `text` as a secret.
pub fn secret(text: &str) -> Secret {
    Secret::new(Zeroizing::new(text.to_owned()))
}

/// `text`, redacted.
pub fn redacted(text: &str) -> Redacted<String> {
    Redacted::new(text.to_owned())
}

/// Login row `id` named `name`, with its count of fields as the hint.
pub fn login(id: u64, name: &str) -> EntryRow {
    EntryRow {
        id,
        kind: EntryKind::Login,
        name: redacted(name),
        hint: Hint::Fields(3),
        buttons: vec![RowButton::User, RowButton::Password, RowButton::Both],
    }
}

/// Page `round` showing `sheet`: All, no search, one page, the first row highlighted.
pub fn page(round: u32, sheet: Sheet) -> FillPage {
    FillPage {
        round,
        sheet,
        filter: FillFilter::All,
        search: redacted(""),
        caret: Redacted::new(0),
        pages: Pages { at: 0, count: 1 },
        highlight: Some(0),
        note: None,
        waiting: None,
        empty: false,
    }
}

/// List page `round` with one login named `Bank`.
pub fn listed(round: u32) -> FillPage {
    page(round, Sheet::List(vec![login(7, "Bank")]))
}

/// The panel as its object reads it.
pub fn fill(rig: &Rig) -> Value {
    serde_json::from_str(&rig.hub.fill_json()).unwrap()
}

#[test]
fn one_view_wakes_once_and_a_second_waits_for_the_read() {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard.panel(true);
    heard.view(listed(1));
    assert_eq!(heard.sent.woken(), 1);
    heard.view(listed(2));
    assert_eq!(heard.sent.woken(), 0, "a wake waits for the read");
    assert_eq!(fill(&rig)["round"], 2, "the newest view shows");
    heard.view(listed(3));
    assert_eq!(heard.sent.woken(), 1, "read, so the next view wakes");
    assert!(rig.wakes().is_empty(), "the Ui object hears none of it");
}

#[test]
fn an_older_view_is_ignored_and_a_reopened_panel_takes_any_round() {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard.shown(&rig, listed(5));
    assert_eq!(fill(&rig)["round"], 5);
    heard.view(listed(4));
    assert_eq!(heard.sent.woken(), 0, "older, so nothing to draw");
    assert_eq!(fill(&rig)["round"], 5);
    heard.view(listed(5));
    assert_eq!(heard.sent.woken(), 1, "the same round draws again");
    heard.panel(false);
    heard.panel(true);
    heard.view(listed(1));
    assert_eq!(fill(&rig)["round"], 1, "a new open counts again");
}

/// The vault answers `FillAsk` with `FillPanel { open: true }` while open, which changes nothing.
#[test]
fn an_open_sent_again_keeps_the_view_and_its_floor() {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard.shown(&rig, listed(5));
    heard.panel(true);
    assert_eq!(fill(&rig)["round"], 5, "the view stays");
    heard.view(listed(4));
    assert_eq!(heard.sent.woken(), 0, "still older");
    assert_eq!(fill(&rig)["round"], 5);
}

#[test]
fn a_closed_panel_drops_its_view_and_wakes_once() {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard.shown(&rig, listed(4));
    assert_eq!(fill(&rig)["round"], 4);
    heard.panel(false);
    assert_eq!(heard.sent.woken(), 1);
    assert_eq!(rig.hub.fill_json(), "null", "nothing to draw");
    heard.panel(false);
    assert_eq!(heard.sent.woken(), 0, "closed already");
}

#[test]
fn a_view_heard_while_closed_is_not_kept() {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard.view(listed(1));
    assert_eq!(heard.sent.woken(), 0);
    heard.panel(true);
    assert_eq!(heard.sent.woken(), 0, "no view yet");
    assert_eq!(rig.hub.fill_json(), "null");
}

#[test]
fn a_new_waker_hears_the_wake_the_panel_never_read() {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard.shown(&rig, listed(1));
    heard.view(listed(2));
    let seen = Arc::new(Mutex::new(0));
    let log = Arc::clone(&seen);
    rig.hub
        .set_fill_waker(Box::new(move || *log.lock().unwrap() += 1));
    assert_eq!(
        *seen.lock().unwrap(),
        1,
        "the unread view wakes the new one"
    );
    heard.view(listed(3));
    assert_eq!(*seen.lock().unwrap(), 1, "still unread");
    assert_eq!(heard.sent.woken(), 0, "the last waker hears nothing more");
}

#[test]
fn a_pad_page_turns_the_open_panel_and_our_list_otherwise() {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard.bus.publish(&ScrollPage { back: true }).unwrap();
    assert_eq!(rig.wakes(), [Wake::Page { back: true }]);
    assert!(heard.sent.acts().is_empty());
    heard.shown(&rig, listed(3));
    heard.bus.publish(&ScrollPage { back: false }).unwrap();
    let turn = FillAct {
        round: 3,
        click: FillClick::Page { back: false },
    };
    assert_eq!(heard.sent.acts(), [turn]);
    assert!(rig.wakes().is_empty(), "the list under it stays");
}
