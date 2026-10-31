//! The clipboard panel through the hub: a view wakes QML once per read and a closed panel drops it, the ui's rows go
//! out as the panel opens and as the settings change, a pad page turns the open panel, pictures serve only the kept
//! round, Open folder shows the folder the app handed over, a row of unknown size says so, and the JSON holds
//! only what QML draws.

mod acts;
mod shape;
mod size;
mod view;

use crate::Rig;
use kx_module_api::ports::{
    ClipAct, ClipKind, ClipPage, ClipPanel, ClipRow, ClipView, Date, Filter, Pages, Pixels,
    Preview, Room, ScrollPage, When,
};
use kx_module_api::{Bus, Redacted, Subscription};
use kx_platform::ui::RevealError;
use kx_test_support::host::{FakeCore, FakeHost};
use kx_ui::{MANIFEST, UiSettings, Wake};
use serde_json::Value;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// The longest a test waits for the file manager's thread.
const WAIT: Duration = Duration::from_secs(5);
/// The history folder the app hands over in these tests.
const FOLDER: &str = r"D:\data\KeyXtend\clipboard";
/// A picture's side in these tests.
const SIDE: u32 = 2;

/// The acts the hub sent on the bus, oldest first.
#[derive(Default)]
pub struct Acts(Mutex<Vec<ClipAct>>);

impl Acts {
    /// The acts sent since the last call.
    pub fn take(&self) -> Vec<ClipAct> {
        std::mem::take(&mut *self.0.lock().unwrap())
    }
}

/// A bus the hub of `rig` hears, the acts it sends there, and the subscriptions that keep both; the wakes are cleared.
pub fn heard(rig: &Rig) -> (Bus, Arc<Acts>, Vec<Subscription>) {
    let bus = Bus::new(Arc::new(FakeCore::default()));
    let acts = Arc::new(Acts::default());
    let seen = Arc::clone(&acts);
    let mut subs = vec![
        bus.subscribe(move |a: &ClipAct| seen.0.lock().unwrap().push(*a))
            .unwrap(),
    ];
    subs.extend(rig.hub.listen(&bus).unwrap());
    rig.wakes();
    (bus, acts, subs)
}

/// The panel opens and shows `page`; the wakes and acts so far are cleared.
pub fn shown(rig: &Rig, bus: &Bus, acts: &Acts, page: ClipPage) {
    bus.publish(&ClipPanel { open: true }).unwrap();
    bus.publish(&ClipView::new(page)).unwrap();
    rig.wakes();
    acts.take();
}

/// `side` square pixels of one colour.
pub fn pixels(side: u32, rgba: [u8; 4]) -> Pixels {
    let n = usize::try_from(side * side).unwrap();
    Pixels::new(side, side, rgba.repeat(n), side).unwrap()
}

/// Text row `id` titled `title`, copied in Notepad at 16:32 today, 12 bytes, with no thumbnail.
pub fn row(id: u64, title: &str) -> ClipRow {
    ClipRow {
        id,
        kind: ClipKind::Text,
        title: Redacted::new(title.to_owned()),
        rtl: false,
        arabic: false,
        app: Some(Redacted::new("Notepad".to_owned())),
        when: When {
            days_ago: 0,
            date: Date::new(2026, 10, 31).unwrap(),
            hour: 16,
            minute: 32,
        },
        bytes: Some(12),
        tick: None,
        pinned: false,
        expires: false,
        thumb: None,
    }
}

/// Page `round` of `rows`, the first highlighted, nothing ticked, with 1 MB of 1 GB used.
pub fn page(round: u32, rows: Vec<ClipRow>) -> ClipPage {
    ClipPage {
        round,
        highlight: (!rows.is_empty()).then_some(0),
        unpinned: u16::try_from(rows.len()).unwrap(),
        rows,
        filter: Filter::All,
        search: Redacted::new(String::new()),
        pages: Pages { at: 0, count: 1 },
        ticked: 0,
        unpin: false,
        preview: None,
        confirm: None,
        room: Room {
            used: 1 << 20,
            free: 1 << 30,
            drive: "D:".to_owned(),
        },
        cap: 1 << 30,
        note: None,
        kept_days: 7,
        empty: false,
    }
}

/// The panel as QML reads it.
pub fn clip(rig: &Rig) -> Value {
    serde_json::from_str(&rig.hub.clip_json()).unwrap()
}

#[test]
fn one_view_wakes_once_and_a_second_waits_for_the_read() {
    let rig = Rig::new();
    let (bus, _acts, _subs) = heard(&rig);
    bus.publish(&ClipPanel { open: true }).unwrap();
    bus.publish(&ClipView::new(page(1, vec![row(7, "alpha")])))
        .unwrap();
    assert_eq!(rig.wakes(), [Wake::Clip]);
    bus.publish(&ClipView::new(page(2, vec![row(7, "beta")])))
        .unwrap();
    assert!(rig.wakes().is_empty(), "a wake waits for the read");
    let read = clip(&rig);
    assert_eq!(read["round"], 2, "the newest view shows");
    bus.publish(&ClipView::new(page(3, vec![row(7, "gamma")])))
        .unwrap();
    assert_eq!(rig.wakes(), [Wake::Clip], "read, so the next view wakes");
}

#[test]
fn a_closed_panel_drops_its_view_and_wakes_once() {
    let rig = Rig::new();
    let (bus, acts, _subs) = heard(&rig);
    shown(&rig, &bus, &acts, page(4, vec![row(7, "alpha")]));
    assert_eq!(clip(&rig)["round"], 4);
    bus.publish(&ClipPanel { open: false }).unwrap();
    assert_eq!(rig.wakes(), [Wake::Clip]);
    assert_eq!(rig.hub.clip_json(), "null", "nothing to draw");
    assert!(
        rig.hub.clip_picture("4/7").is_none(),
        "its pictures went too"
    );
    bus.publish(&ClipPanel { open: false }).unwrap();
    assert!(rig.wakes().is_empty(), "closed already");
}

#[test]
fn a_view_heard_while_closed_is_not_kept() {
    let rig = Rig::new();
    let (bus, _acts, _subs) = heard(&rig);
    bus.publish(&ClipView::new(page(1, vec![row(7, "late")])))
        .unwrap();
    assert!(rig.wakes().is_empty());
    bus.publish(&ClipPanel { open: true }).unwrap();
    assert!(rig.wakes().is_empty(), "no view yet");
    assert_eq!(rig.hub.clip_json(), "null");
}

#[test]
fn a_new_waker_hears_the_wake_qml_never_read() {
    let rig = Rig::new();
    let (bus, acts, _subs) = heard(&rig);
    shown(&rig, &bus, &acts, page(1, vec![row(7, "alpha")]));
    bus.publish(&ClipView::new(page(2, vec![row(7, "alpha")])))
        .unwrap();
    rig.wakes();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let log = Arc::clone(&seen);
    rig.hub
        .set_waker(Box::new(move |w| log.lock().unwrap().push(w)));
    assert!(seen.lock().unwrap().contains(&Wake::Clip));
}

#[test]
fn the_ui_rows_go_out_as_the_panel_opens_and_as_the_settings_change() {
    let (rig, mut host) = crate::words::speaking("en", 4);
    let (bus, acts, _subs) = heard(&rig);
    bus.publish(&ClipPanel { open: true }).unwrap();
    assert_eq!(acts.take(), [ClipAct::Rows(4)]);
    let mut settings = UiSettings::from_defaults().unwrap();
    settings.page_rows = 6;
    rig.hub.attach(
        settings,
        Arc::new(crate::FakeFace::default()),
        host.handle(),
    );
    assert_eq!(acts.take(), [ClipAct::Rows(6)]);
    bus.publish(&ClipPanel { open: false }).unwrap();
    let settings = UiSettings::from_defaults().unwrap();
    let mut other = FakeHost::new(&MANIFEST);
    rig.hub.attach(
        settings,
        Arc::new(crate::FakeFace::default()),
        other.handle(),
    );
    assert!(acts.take().is_empty(), "closed, so no rows");
}

#[test]
fn rows_past_what_a_byte_holds_are_clamped() {
    let (rig, _host) = crate::words::speaking("en", 300);
    let (bus, acts, _subs) = heard(&rig);
    bus.publish(&ClipPanel { open: true }).unwrap();
    assert_eq!(acts.take(), [ClipAct::Rows(u8::MAX)]);
}

#[test]
fn a_pad_page_turns_the_open_panel_and_our_list_otherwise() {
    let rig = Rig::new();
    let (bus, acts, _subs) = heard(&rig);
    bus.publish(&ScrollPage { back: true }).unwrap();
    assert_eq!(rig.wakes(), [Wake::Page { back: true }]);
    assert!(acts.take().is_empty());
    shown(&rig, &bus, &acts, page(1, vec![row(7, "alpha")]));
    bus.publish(&ScrollPage { back: false }).unwrap();
    assert_eq!(acts.take(), [ClipAct::Page { back: false }]);
    assert!(rig.wakes().is_empty(), "the list under it stays");
}

#[test]
fn pictures_come_from_the_kept_round_only() {
    let rig = Rig::new();
    let (bus, acts, _subs) = heard(&rig);
    let mut thumbed = row(7, "picture");
    thumbed.kind = ClipKind::Image;
    thumbed.thumb = Some(pixels(SIDE, [255, 0, 0, 255]));
    let mut kept = page(5, vec![thumbed, row(8, "words")]);
    kept.preview = Some(Preview::Image(Some(pixels(SIDE, [0, 0, 255, 255]))));
    shown(&rig, &bus, &acts, kept);
    let thumb = rig.hub.clip_picture("5/7").expect("the thumbnail");
    assert_eq!(thumb.rgba().expose()[..4], [255, 0, 0, 255]);
    let large = rig.hub.clip_picture("5/preview").expect("the preview");
    assert_eq!(large.rgba().expose()[..4], [0, 0, 255, 255]);
    for gone in ["4/7", "6/7", "5/8", "5/9", "5", "", "x/7", "5/7/1", "-5/7"] {
        assert!(rig.hub.clip_picture(gone).is_none(), "{gone}");
    }
}

#[test]
fn open_folder_shows_the_folder_the_app_handed_over() {
    let rig = Rig::new();
    let folder = PathBuf::from(FOLDER);
    assert!(!rig.hub.open_folder(), "none handed over");
    assert!(rig.reveal.shown().is_empty());
    rig.hub.set_clip_folder(folder.clone());
    rig.hub.set_clip_folder(PathBuf::from(r"C:\other"));
    rig.hub.open_folder();
    assert!(rig.reveal.wait_shown(1, WAIT));
    assert_eq!(rig.reveal.shown(), [folder], "the first one stays");
}

/// The refusal's warning is logged on the reveal's own thread, which the log capture does not hear.
#[test]
fn a_refused_folder_ends_its_reveal_without_a_panic() {
    let rig = Rig::new();
    rig.reveal.set_error(Some(RevealError::Refused(5)));
    rig.hub.set_clip_folder(PathBuf::from(FOLDER));
    rig.hub.open_folder();
    assert!(rig.reveal.wait_left(1, WAIT));
    assert_eq!(rig.reveal.shown(), [PathBuf::from(FOLDER)]);
}

#[test]
fn a_double_tap_opens_one_window() {
    let rig = Rig::new();
    rig.hub.set_clip_folder(PathBuf::from(FOLDER));
    rig.reveal.hold(true);
    assert!(rig.hub.open_folder(), "the first tap starts a reveal");
    assert!(rig.reveal.wait_shown(1, WAIT));
    assert!(!rig.hub.open_folder(), "one is in flight");
    assert!(!rig.hub.open_folder());
    rig.reveal.hold(false);
    assert!(rig.reveal.wait_left(1, WAIT));
    assert_eq!(
        rig.reveal.shown().len(),
        1,
        "the taps meanwhile were ignored"
    );
}
