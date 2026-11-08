//! The selection chips' events and ports (ADR-0041): the chips' events travel only with `Selection`, the selection
//! watch needs `ReadFocusedText`, the browser link needs `OpenLink`, and the release and a named press request are open.

use kx_module_api::Capability::{InjectInput, OpenLink, ReadFocusedText, Selection};
use kx_module_api::ports::{
    ChipAct, ChipsView, Edit, EditAsk, EditDone, Legend, Link, LinkError, PointerReleased,
    PressAsker, PressesWanted, Rect, Screen, SelChip, SelConfig, SelError, SelKind, SelRun,
    SelSpot, SelectionWatch, SelectionWatchKey, StripSpot, WebLink, WebLinkKey,
};
use kx_module_api::{
    Bus, Capability, Event, EventError, Host, Manifest, ModuleCx, ModuleId, Redacted, ServiceError,
    ServiceKey,
};
use kx_test_support::host::FakeHost;
use std::sync::{Arc, Mutex};

/// A selection module declaring the three gates, and `InjectInput` to show that another grant opens none.
static SELECT: Manifest = Manifest {
    id: ModuleId::new("selection"),
    version: env!("CARGO_PKG_VERSION"),
    requires: &[SelectionWatchKey::ID, WebLinkKey::ID],
    provides: &[],
    capabilities: &[Selection, OpenLink, ReadFocusedText, InjectInput],
    settings: None,
};

/// What the one box holds selected ("hello").
const TEXT: &str = "مرحبا";
/// The one box's id.
const BOX: u64 = 9;
/// Google's translate page, as the selection module builds it.
const LINK: &str = "https://translate.google.com/?sl=auto&tl=en&text=hi&op=translate";

/// The one box's line, as its app tells it.
const LINE: Rect = Rect {
    x: 100,
    y: 100,
    w: 200,
    h: 20,
};

/// The screen that holds it.
const SCREEN: Screen = Screen {
    rect: Rect {
        x: 0,
        y: 0,
        w: 1920,
        h: 1040,
    },
    dpi: 96,
    primary: true,
};

/// A watch on one editable box holding a selection, which tells one change at once.
struct OneBox;

/// The run over that box.
struct OneRun;

/// A browser that keeps each link it was handed.
#[derive(Default)]
struct Browser(Mutex<Vec<String>>);

impl SelectionWatch for OneBox {
    fn watch(&self, _: SelConfig, tell: Box<dyn Fn() + Send>) -> Result<Box<dyn SelRun>, SelError> {
        tell();
        Ok(Box::new(OneRun))
    }
}

impl SelRun for OneRun {
    fn look(&mut self) -> Result<SelSpot, SelError> {
        Ok(SelSpot {
            box_id: BOX,
            kind: SelKind::Editable,
            selected: true,
            first: Some(LINE),
            last: Some(LINE),
            caret: None,
            screen: SCREEN,
        })
    }

    fn still(&mut self, box_id: u64) -> bool {
        box_id == BOX
    }

    fn read(&mut self, box_id: u64, max_chars: u16) -> Result<Redacted<String>, SelError> {
        if box_id != BOX {
            return Err(SelError::Closed);
        }
        let text: String = TEXT.chars().take(usize::from(max_chars)).collect();
        Ok(Redacted::new(text))
    }

    fn clip_holds(&self) -> bool {
        true
    }
}

impl WebLink for Browser {
    fn open(&self, link: &Link) -> Result<(), LinkError> {
        let mut opened = self.0.lock().map_err(|_| LinkError::Unavailable)?;
        opened.push(link.expose().to_owned());
        Ok(())
    }
}

/// Puts `api` into `host` under `K`'s id and gate.
fn put<K: ServiceKey>(host: &mut FakeHost, api: Arc<K::Api>) {
    host.services.insert(K::ID, (K::CAPABILITY, Box::new(api)));
}

/// A host granted `granted`, holding both ports as the platform would provide them.
fn host(granted: &[Capability]) -> (FakeHost, Arc<Browser>) {
    let browser = Arc::new(Browser::default());
    let mut host = FakeHost::new(&SELECT);
    host.granted.extend(granted);
    put::<SelectionWatchKey>(&mut host, Arc::new(OneBox));
    put::<WebLinkKey>(&mut host, Arc::clone(&browser) as Arc<dyn WebLink>);
    (host, browser)
}

/// The strip by the one line.
fn shown() -> ChipsView {
    ChipsView::Shown {
        round: 1,
        chips: SelChip::ALL.to_vec(),
        spot: StripSpot {
            first: LINE,
            last: LINE,
            screen: SCREEN,
        },
        acted: None,
        legend: Legend::Icons,
    }
}

/// Sends one of each chip event on `bus`, then subscribes to each.
fn chip_events(bus: &Bus) -> [Result<(), EventError>; 8] {
    let ask = EditAsk {
        round: 1,
        edit: Edit::Copy,
        terminal: false,
    };
    [
        bus.publish(&shown()),
        bus.publish(&ChipAct {
            round: 1,
            chip: SelChip::Copy,
        }),
        bus.publish(&ask),
        bus.publish(&EditDone { round: 1, ok: true }),
        bus.subscribe(|_: &ChipsView| {}).map(drop),
        bus.subscribe(|_: &ChipAct| {}).map(drop),
        bus.subscribe(|_: &EditAsk| {}).map(drop),
        bus.subscribe(|_: &EditDone| {}).map(drop),
    ]
}

/// The refusal of `E` for want of `Selection`.
fn refused<E: Event>() -> Result<(), EventError> {
    Err(EventError::NotGranted(E::NAME, Selection))
}

#[test]
fn without_selection_no_chip_event_travels() {
    let (host, _) = host(&[ReadFocusedText, OpenLink, InjectInput]);
    let want = [
        refused::<ChipsView>(),
        refused::<ChipAct>(),
        refused::<EditAsk>(),
        refused::<EditDone>(),
        refused::<ChipsView>(),
        refused::<ChipAct>(),
        refused::<EditAsk>(),
        refused::<EditDone>(),
    ];
    assert_eq!(chip_events(&host.bus()), want);
}

#[test]
fn with_selection_a_view_reaches_its_listener_whole() {
    let (host, _) = host(&[Selection]);
    let bus = host.bus();
    assert_eq!(chip_events(&bus), [Ok(()); 8]);
    let (tx, rx) = std::sync::mpsc::channel();
    let _sub = bus
        .subscribe(move |v: &ChipsView| tx.send(v.clone()).unwrap())
        .unwrap();
    bus.publish(&shown()).unwrap();
    bus.publish(&ChipsView::Hidden).unwrap();
    assert_eq!(
        rx.try_iter().collect::<Vec<_>>(),
        [shown(), ChipsView::Hidden]
    );
}

#[test]
fn a_release_and_a_named_press_request_pass_with_no_grants() {
    let bus = host(&[]).0.bus();
    assert_eq!(bus.publish(&PointerReleased { ours: false }), Ok(()));
    let wanted = PressesWanted {
        asker: PressAsker::Selection,
        on: true,
    };
    assert_eq!(bus.publish(&wanted), Ok(()));
}

#[test]
fn the_watch_needs_read_focused_text_and_the_link_open_link() {
    let (mut host, _) = host(&[Selection, InjectInput]);
    let cx = ModuleCx::new(&mut host);
    let watch = ServiceError::NotGranted(SelectionWatchKey::ID, ReadFocusedText);
    assert_eq!(cx.service::<SelectionWatchKey>().err(), Some(watch));
    let link = ServiceError::NotGranted(WebLinkKey::ID, OpenLink);
    assert_eq!(cx.service::<WebLinkKey>().err(), Some(link));
}

#[test]
fn a_watch_reached_through_its_gate_tells_looks_checks_and_reads() {
    let (mut host, _) = host(&[ReadFocusedText]);
    let watch = ModuleCx::new(&mut host)
        .service::<SelectionWatchKey>()
        .unwrap();
    let told = Arc::new(Mutex::new(0));
    let count = Arc::clone(&told);
    let tell = Box::new(move || *count.lock().unwrap() += 1);
    let cfg = SelConfig {
        settle_ms: 1,
        read_ms: 1,
        looks_per_s: 1,
    };
    let mut run = watch.watch(cfg, tell).unwrap();
    assert_eq!(*told.lock().unwrap(), 1);
    let spot = run.look().unwrap();
    assert_eq!(
        (spot.box_id, spot.kind, spot.last),
        (BOX, SelKind::Editable, Some(LINE))
    );
    assert!(run.still(BOX) && !run.still(BOX + 1));
    assert_eq!(run.read(BOX, 2).unwrap().expose(), "مر");
    assert_eq!(
        run.read(BOX + 1, 2).err(),
        Some(SelError::Closed),
        "another box"
    );
    assert!(run.clip_holds());
}

#[test]
fn a_link_reached_through_its_gate_goes_to_the_browser() {
    let (mut host, browser) = host(&[OpenLink]);
    let web = ModuleCx::new(&mut host).service::<WebLinkKey>().unwrap();
    let link = Link::new(LINK.to_owned()).unwrap();
    assert_eq!(web.open(&link), Ok(()));
    assert_eq!(*browser.0.lock().unwrap(), [LINK]);
}
