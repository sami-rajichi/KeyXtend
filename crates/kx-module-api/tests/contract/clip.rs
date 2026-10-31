//! The clipboard history's events and ports (ADR-0034, ADR-0035): the panel's state, content and search travel only
//! with `ReadClipboard`, the paste only with `WriteClipboard`, the shelf needs `SecretStore`, and the open toggle, the
//! chord flags, pictures and calendar are open.

use kx_module_api::Capability::{InjectInput, ReadClipboard, SecretStore, WriteClipboard};
use kx_module_api::ports::{
    BlobName, Calendar, CalendarKey, ClipAct, ClipConfig, ClipError, ClipKey, ClipNote, ClipOpen,
    ClipPage, ClipPanel, ClipPart, ClipRun, ClipView, ClipWait, Clipboard, ClipboardKey,
    ClipboardWriter, ClipboardWriterKey, Date, FileFact, Filter, HistoryAsk, HistoryWanted,
    LocalTime, Pages, PanelKey, PasteAsk, Pasted, PictureAsk, PictureConfig, PictureError,
    PictureRun, Pictures, PicturesKey, Pixels, Room, SealedError, SealedShelf, SealedShelfKey,
};
use kx_module_api::{
    Bus, Capability, Event, EventError, Host, Manifest, ModuleCx, ModuleId, Redacted, ServiceError,
    ServiceKey,
};
use kx_test_support::host::FakeHost;
use std::path::PathBuf;
use std::sync::Arc;

/// A clipboard module declaring the three gates, and `InjectInput` to show that another grant opens none.
static CLIP: Manifest = Manifest {
    id: ModuleId::new("clipboard"),
    version: env!("CARGO_PKG_VERSION"),
    requires: &[
        ClipboardKey::ID,
        ClipboardWriterKey::ID,
        SealedShelfKey::ID,
        PicturesKey::ID,
        CalendarKey::ID,
    ],
    provides: &[],
    capabilities: &[ReadClipboard, WriteClipboard, SecretStore, InjectInput],
    settings: None,
};

/// A platform whose clipboard is always busy, whose shelf is locked and whose worker is gone.
struct Stuck;

/// A watch that hears nothing.
struct Deaf;

impl ClipRun for Deaf {}

impl Clipboard for Stuck {
    fn watch(
        &self,
        _: ClipConfig,
        tell: Box<dyn Fn(ClipNote) + Send>,
    ) -> Result<Box<dyn ClipRun>, ClipError> {
        tell(ClipNote::Busy);
        Ok(Box::new(Deaf))
    }

    fn files(&self, paths: &Redacted<Vec<PathBuf>>) -> Vec<FileFact> {
        paths.expose().iter().map(|_| FileFact::Gone).collect()
    }
}

impl ClipboardWriter for Stuck {
    fn write(&self, _: &[ClipPart], _: ClipWait) -> Result<(), ClipError> {
        Err(ClipError::Busy)
    }
}

impl SealedShelf for Stuck {
    fn load(&self, _: &BlobName, _: &BlobName) -> Result<Option<Vec<u8>>, SealedError> {
        Err(SealedError::Locked)
    }

    fn save(&self, _: &BlobName, _: &BlobName, _: &[u8]) -> Result<(), SealedError> {
        Err(SealedError::Locked)
    }

    fn remove(&self, _: &BlobName, _: &BlobName) -> Result<(), SealedError> {
        Err(SealedError::Locked)
    }

    fn drop_folder(&self, _: &BlobName) -> Result<(), SealedError> {
        Err(SealedError::Locked)
    }

    fn folders(&self) -> Result<Vec<BlobName>, SealedError> {
        Err(SealedError::Locked)
    }

    fn names(&self, _: &BlobName) -> Result<Vec<BlobName>, SealedError> {
        Err(SealedError::Locked)
    }

    fn room(&self) -> Result<Room, SealedError> {
        Err(SealedError::Locked)
    }
}

impl PictureRun for Stuck {
    fn draw(&mut self, _: PictureAsk) -> Result<Pixels, PictureError> {
        Err(PictureError::Gone)
    }
}

impl Pictures for Stuck {
    fn open(&self, _: PictureConfig) -> Box<dyn PictureRun> {
        Box::new(Stuck)
    }
}

impl Calendar for Stuck {
    #[allow(clippy::unwrap_used, reason = "day 0, the epoch, is a date")]
    fn now(&self) -> LocalTime {
        LocalTime {
            unix_ms: 0,
            date: Date::from_day_number(0).unwrap(),
            hour: 0,
            minute: 0,
            offset_min: 0,
        }
    }
}

/// Puts `api` into `host` under `K`'s id and gate.
fn put<K: ServiceKey>(host: &mut FakeHost, api: Arc<K::Api>) {
    host.services.insert(K::ID, (K::CAPABILITY, Box::new(api)));
}

/// A host granted `granted`, holding the five ports as the platform would provide them.
fn host(granted: &[Capability]) -> FakeHost {
    let mut host = FakeHost::new(&CLIP);
    host.granted.extend(granted);
    put::<ClipboardKey>(&mut host, Arc::new(Stuck));
    put::<ClipboardWriterKey>(&mut host, Arc::new(Stuck));
    put::<SealedShelfKey>(&mut host, Arc::new(Stuck));
    put::<PicturesKey>(&mut host, Arc::new(Stuck));
    put::<CalendarKey>(&mut host, Arc::new(Stuck));
    host
}

/// Sends each gated event but the view, subscribes to each, then sends the open `ClipOpen`.
fn gated(bus: &Bus) -> [Result<(), EventError>; 12] {
    let key = ClipKey(PanelKey::Text(Redacted::default()));
    [
        bus.publish(&ClipPanel { open: true }),
        bus.publish(&key),
        bus.publish(&ClipAct::Filter(Filter::Images)),
        bus.publish(&PasteAsk { round: 1 }),
        bus.publish(&Pasted { round: 1, ok: true }),
        bus.subscribe(|_: &ClipPanel| {}).map(drop),
        bus.subscribe(|_: &ClipKey| {}).map(drop),
        bus.subscribe(|_: &ClipView| {}).map(drop),
        bus.subscribe(|_: &ClipAct| {}).map(drop),
        bus.subscribe(|_: &PasteAsk| {}).map(drop),
        bus.subscribe(|_: &Pasted| {}).map(drop),
        bus.publish(&ClipOpen),
    ]
}

/// The search line of `page` ("hello").
const SEARCH: &str = "مرحبا";

/// An empty history's page, searched for `SEARCH`.
fn page() -> ClipPage {
    ClipPage {
        round: 1,
        rows: Vec::new(),
        highlight: None,
        filter: Filter::All,
        search: Redacted::new(SEARCH.to_owned()),
        pages: Pages::default(),
        ticked: 0,
        unpin: false,
        preview: None,
        confirm: None,
        unpinned: 0,
        room: Some(Room {
            used: 0,
            free: 1,
            drive: "D:".to_owned(),
        }),
        cap: 1,
        note: None,
        kept_days: 7,
        empty: true,
    }
}

/// The refusal of `E` for want of `cap`.
fn refused<E: Event>(cap: Capability) -> Result<(), EventError> {
    Err(EventError::NotGranted(E::NAME, cap))
}

#[test]
fn without_the_clipboard_gates_no_content_or_paste_travels() {
    let bus = host(&[InjectInput, SecretStore]).bus();
    let (read, write) = (ReadClipboard, WriteClipboard);
    let want = [
        refused::<ClipPanel>(read),
        refused::<ClipKey>(read),
        refused::<ClipAct>(read),
        refused::<PasteAsk>(write),
        refused::<Pasted>(write),
        refused::<ClipPanel>(read),
        refused::<ClipKey>(read),
        refused::<ClipView>(read),
        refused::<ClipAct>(read),
        refused::<PasteAsk>(write),
        refused::<Pasted>(write),
        Ok(()),
    ];
    assert_eq!(gated(&bus), want);
}

#[test]
fn read_clipboard_alone_opens_content_but_not_the_paste() {
    let bus = host(&[ReadClipboard]).bus();
    let want = [
        Ok(()),
        Ok(()),
        Ok(()),
        refused::<PasteAsk>(WriteClipboard),
        refused::<Pasted>(WriteClipboard),
        Ok(()),
        Ok(()),
        Ok(()),
        Ok(()),
        refused::<PasteAsk>(WriteClipboard),
        refused::<Pasted>(WriteClipboard),
        Ok(()),
    ];
    assert_eq!(gated(&bus), want);
}

#[test]
fn a_view_reaches_its_listener_only_with_read_clipboard() {
    let view = ClipView::new(page());
    let closed = host(&[WriteClipboard, SecretStore]).bus();
    assert_eq!(closed.publish(&view), refused::<ClipView>(ReadClipboard));
    let bus = host(&[ReadClipboard]).bus();
    let (tx, rx) = std::sync::mpsc::channel();
    let _sub = bus
        .subscribe(move |v: &ClipView| tx.send(v.clone()).unwrap())
        .unwrap();
    assert_eq!(bus.publish(&view), Ok(()));
    let heard = rx.try_recv().unwrap();
    assert!(
        Arc::ptr_eq(&heard.0, &view.0),
        "the listener shares the page"
    );
    assert_eq!(heard.0.search.expose(), SEARCH);
}

#[test]
fn the_open_toggle_and_the_chord_flags_pass_with_no_grants() {
    let bus = host(&[]).bus();
    assert_eq!(bus.publish(&ClipOpen), Ok(()));
    assert_eq!(bus.publish(&HistoryWanted { on: true }), Ok(()));
    assert_eq!(bus.publish(&HistoryAsk), Ok(()));
}

#[test]
fn each_port_needs_its_gate_and_pictures_and_the_calendar_none() {
    let mut host = host(&[InjectInput]);
    let cx = ModuleCx::new(&mut host);
    let read = ServiceError::NotGranted(ClipboardKey::ID, ReadClipboard);
    assert_eq!(cx.service::<ClipboardKey>().err(), Some(read));
    let write = ServiceError::NotGranted(ClipboardWriterKey::ID, WriteClipboard);
    assert_eq!(cx.service::<ClipboardWriterKey>().err(), Some(write));
    let shelf = ServiceError::NotGranted(SealedShelfKey::ID, SecretStore);
    assert_eq!(cx.service::<SealedShelfKey>().err(), Some(shelf));
    assert!(cx.service::<PicturesKey>().is_ok());
    assert_eq!(cx.service::<CalendarKey>().unwrap().now().unix_ms, 0);
}

#[test]
fn ports_reached_through_their_gates_answer() {
    let mut host = host(&[ReadClipboard, WriteClipboard, SecretStore]);
    let cx = ModuleCx::new(&mut host);
    let wait = ClipWait {
        open_ms: 1,
        poll_ms: 1,
    };
    let cfg = ClipConfig { max_bytes: 1, wait };
    let (tx, rx) = std::sync::mpsc::channel();
    let tell = Box::new(move |n: ClipNote| tx.send(n).unwrap());
    let clipboard = cx.service::<ClipboardKey>().unwrap();
    let _run = clipboard.watch(cfg, tell).unwrap();
    assert_eq!(rx.try_recv(), Ok(ClipNote::Busy));
    let gone = clipboard.files(&Redacted::new(vec![PathBuf::from("a.txt")]));
    assert_eq!(gone, [FileFact::Gone]);
    let writer = cx.service::<ClipboardWriterKey>().unwrap();
    assert_eq!(writer.write(&[], wait), Err(ClipError::Busy));
    let shelf = cx.service::<SealedShelfKey>().unwrap();
    assert_eq!(shelf.room(), Err(SealedError::Locked));
}
