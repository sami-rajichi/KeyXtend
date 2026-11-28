//! The clipboard history's events and ports (ADR-0034, ADR-0035): the panel's state, content and search travel only
//! with `ReadClipboard`, the paste only with `WriteClipboard`, the shelf needs `SecretStore`, and the open toggle, the
//! chord flags, pictures and calendar are open.

use kx_module_api::Capability::{InjectInput, ReadClipboard, SecretStore, WriteClipboard};
use kx_module_api::ports::{
    CalendarKey, ClipAct, ClipConfig, ClipError, ClipKey, ClipNote, ClipOpen, ClipPage, ClipPanel,
    ClipView, ClipWait, ClipboardKey, ClipboardWriterKey, FileFact, Filter, HistoryAsk,
    HistoryWanted, Pages, PanelKey, PasteAsk, Pasted, PicturesKey, Room, SealedError,
    SealedShelfKey,
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

mod kept;
mod stuck;

use stuck::Stuck;

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
        caret: Redacted::new(2),
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
    assert_eq!(*heard.0.caret.expose(), 2, "where its caret is");
}

#[test]
fn the_delete_key_and_a_placed_caret_need_read_clipboard() {
    let closed = host(&[WriteClipboard, InjectInput]).bus();
    let key = ClipKey(PanelKey::Delete);
    let caret = ClipAct::Caret { at: 1.into() };
    assert_eq!(closed.publish(&key), refused::<ClipKey>(ReadClipboard));
    assert_eq!(closed.publish(&caret), refused::<ClipAct>(ReadClipboard));
    let open = host(&[ReadClipboard]).bus();
    assert_eq!(open.publish(&key), Ok(()));
    assert_eq!(open.publish(&caret), Ok(()));
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
    assert_eq!(clipboard.text(wait, 1), Err(ClipError::Busy));
    assert_eq!(clipboard.unmarked_text(wait, 1), Err(ClipError::Busy));
    let writer = cx.service::<ClipboardWriterKey>().unwrap();
    assert_eq!(writer.write(&[], wait), Err(ClipError::Busy));
    assert_eq!(writer.write_lent(&[], wait), Err(ClipError::Busy));
    let shelf = cx.service::<SealedShelfKey>().unwrap();
    assert_eq!(shelf.room(), Err(SealedError::Locked));
}
